//! tdm-dash co-processor firmware (Raspberry Pi Pico / Pico 2).
//!
//! The application logic is `tdm_core::McuCore`, the same code the desktop
//! simulator runs; this file only connects it to the hardware:
//!
//! | Function                                      | Pin       |
//! |-----------------------------------------------|-----------|
//! | Wheel sensor (opto, active low)               | GP2       |
//! | Ignition pickup (H11L1, active low)           | GP3       |
//! | FRAM MB85RC256V, I2C0 SDA / SCL               | GP4 / GP5 |
//! | Neutral, high beam, turn L, turn R, lights    | GP6–GP10  |
//! | Buttons MODE, SET (to ground)                 | GP11, GP12|
//! | Backlight PWM to the TPS61169                 | GP15      |
//! | ADC: battery, coolant sender, light sensor    | GP26–GP28 |
//!
//! Edge timestamps are taken on a high-priority interrupt executor and
//! handed over through a channel, so a slow FRAM write never delays them.

#![no_std]
#![no_main]

use defmt::{info, warn};
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_rp::adc::{self, Adc, Channel as AdcChannel};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::i2c::{self, I2c};
use embassy_rp::interrupt;
use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::peripherals::{I2C0, USB};
use embassy_rp::pwm::{self, Pwm};
use embassy_rp::usb::{self, Driver};
use embassy_rp::watchdog::Watchdog;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Ticker, Timer, with_timeout};
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::{Builder, UsbDevice};
use static_cell::StaticCell;
use tdm_core::McuCore;
use tdm_core::nvm::{Nvm, RECORD_LEN};
use tdm_proto::{Dir, FrameDecoder, HostMsg, MAX_FRAME, McuMsg, input};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[cfg(feature = "rp2350")]
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: embassy_rp::block::ImageDef = embassy_rp::block::ImageDef::secure_exe();

/// Supply below ≈ 7 V for 2 ms means the key was turned or the battery
/// dropped out: save the odometer while the hold-up capacitor lasts.
const POWER_FAIL_RAW: u16 = 1569; // 7.0 V through 100 k / 22 k at 3.3 V ref
const FRAM_ADDR: u8 = 0x50;

#[derive(Clone, Copy, defmt::Format)]
enum Src {
    Wheel,
    Ignition,
}

static EDGES: Channel<CriticalSectionRawMutex, (Src, u64), 64> = Channel::new();
static FROM_HOST: Channel<CriticalSectionRawMutex, HostMsg, 8> = Channel::new();
static TO_HOST: Channel<CriticalSectionRawMutex, McuMsg, 8> = Channel::new();
static HOST_LOST: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn SWI_IRQ_1() {
    // SAFETY: called only from this interrupt, as the executor requires.
    unsafe { EXECUTOR_HIGH.on_interrupt() }
}

// ---------------------------------------------------------------- FRAM

struct Fram {
    i2c: I2c<'static, I2C0, i2c::Blocking>,
}

impl Nvm for Fram {
    type Error = i2c::Error;

    fn read(&mut self, addr: u16, buf: &mut [u8]) -> Result<(), i2c::Error> {
        self.i2c
            .blocking_write_read(FRAM_ADDR, &addr.to_be_bytes(), buf)
    }

    fn write(&mut self, addr: u16, data: &[u8]) -> Result<(), i2c::Error> {
        let mut b = [0u8; 2 + RECORD_LEN];
        let n = data.len().min(RECORD_LEN);
        b[..2].copy_from_slice(&addr.to_be_bytes());
        b[2..2 + n].copy_from_slice(&data[..n]);
        self.i2c.blocking_write(FRAM_ADDR, &b[..2 + n])
    }
}

// ---------------------------------------------------------------- tasks

#[embassy_executor::task(pool_size = 2)]
async fn edge_task(mut pin: Input<'static>, src: Src) {
    loop {
        pin.wait_for_falling_edge().await;
        let t = Instant::now().as_micros();
        if EDGES.try_send((src, t)).is_err() {
            warn!("edge queue full ({})", src);
        }
    }
}

/// Moving average of the last 16 ms of ADC samples (4 per ms → 64 samples).
struct Avg {
    buf: [u32; 16],
    i: usize,
    sum: u32,
}

impl Avg {
    const fn new() -> Self {
        Self {
            buf: [0; 16],
            i: 0,
            sum: 0,
        }
    }
    fn push(&mut self, four_samples: u32) -> u16 {
        self.sum = self.sum - self.buf[self.i] + four_samples;
        self.buf[self.i] = four_samples;
        self.i = (self.i + 1) % 16;
        (self.sum / 64) as u16
    }
}

struct Io {
    inputs: [Input<'static>; 5],
    btn_mode: Input<'static>,
    btn_set: Input<'static>,
    adc: Adc<'static, adc::Blocking>,
    ch: [AdcChannel<'static>; 3],
    pwm: Pwm<'static>,
    pwm_cfg: pwm::Config,
    wd: Watchdog,
}

#[embassy_executor::task]
async fn control(mut core: McuCore<Fram>, mut io: Io) {
    let mut ticker = Ticker::every(Duration::from_millis(1));
    let mut avg = [Avg::new(), Avg::new(), Avg::new()];
    let mut low_ms = 0u8;
    let mut power_failed = false;
    let mut last_duty = u16::MAX;
    loop {
        ticker.next().await;
        io.wd.feed(Duration::from_secs(1));
        let now = Instant::now().as_micros();

        while let Ok((src, t)) = EDGES.try_receive() {
            match src {
                Src::Wheel => core.on_wheel_edge(t),
                Src::Ignition => core.on_ign_edge(t),
            }
        }

        // opto outputs and buttons pull the pins low when active
        let mut raw = 0u16;
        for (i, p) in io.inputs.iter().enumerate() {
            if p.is_low() {
                raw |= 1 << i;
            }
        }
        if io.btn_mode.is_low() {
            raw |= input::BTN_MODE;
        }
        if io.btn_set.is_low() {
            raw |= input::BTN_SET;
        }
        core.on_inputs(raw, 1);

        let mut reading = [0u16; 3];
        let mut vbat_now = 0u32;
        for (k, ch) in io.ch.iter_mut().enumerate() {
            let mut s = 0u32;
            for _ in 0..4 {
                s += io.adc.blocking_read(ch).unwrap_or(0) as u32;
            }
            if k == 0 {
                vbat_now = s / 4;
            }
            reading[k] = avg[k].push(s);
        }
        core.on_adc(reading[0], reading[1], reading[2]);

        if (vbat_now as u16) < POWER_FAIL_RAW {
            low_ms = low_ms.saturating_add(1);
        } else {
            low_ms = 0;
            power_failed = false;
        }
        if low_ms >= 2 && !power_failed {
            power_failed = true;
            let m = core.on_power_fail();
            let _ = TO_HOST.try_send(m);
            warn!("power fail: odometer saved");
        }

        if HOST_LOST.try_take().is_some() {
            core.on_host_lost();
        }
        while let Ok(m) = FROM_HOST.try_receive() {
            if let Some(reply) = core.on_host(m, now) {
                let _ = TO_HOST.try_send(reply);
            }
        }
        if core.take_bootloader_request() {
            info!("rebooting into the USB bootloader");
            Timer::after_millis(50).await;
            embassy_rp::rom_data::reset_to_usb_boot(0, 0);
        }

        let p = core.poll(now);
        if let Some(t) = p.telemetry {
            let _ = TO_HOST.try_send(McuMsg::Telemetry(t));
        }
        if let Some(o) = p.odometer {
            let _ = TO_HOST.try_send(McuMsg::Odometer(o));
        }
        if p.backlight_permille != last_duty {
            last_duty = p.backlight_permille;
            io.pwm_cfg.compare_b =
                ((io.pwm_cfg.top as u32 + 1) * p.backlight_permille as u32 / 1000) as u16;
            io.pwm.set_config(&io.pwm_cfg);
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut dev: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    dev.run().await
}

#[embassy_executor::task]
async fn usb_rx(mut rx: Receiver<'static, Driver<'static, USB>>) {
    let mut buf = [0u8; 64];
    loop {
        rx.wait_connection().await;
        // a new connection must speak first (docs/adr/0002)
        HOST_LOST.signal(());
        let mut dec = FrameDecoder::<MAX_FRAME>::new(Dir::HostToMcu);
        loop {
            match rx.read_packet(&mut buf).await {
                Ok(n) => dec.feed(&buf[..n], |r| {
                    if let Ok(p) = r
                        && let Ok(m) = tdm_proto::decode::<HostMsg>(p)
                    {
                        let _ = FROM_HOST.try_send(m);
                    }
                }),
                Err(_) => {
                    HOST_LOST.signal(());
                    break;
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn usb_tx(mut tx: Sender<'static, Driver<'static, USB>>) {
    let mut out = [0u8; MAX_FRAME];
    loop {
        tx.wait_connection().await;
        'conn: loop {
            let m = TO_HOST.receive().await;
            let Ok(n) = tdm_proto::encode(&m, Dir::McuToHost, &mut out) else {
                continue;
            };
            for chunk in out[..n].chunks(64) {
                // a host that stopped reading must not block the queue
                match with_timeout(Duration::from_millis(50), tx.write_packet(chunk)).await {
                    Ok(Ok(())) => {}
                    _ => break 'conn,
                }
            }
        }
    }
}

// ---------------------------------------------------------------- main

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("tdm-dash firmware {}", env!("CARGO_PKG_VERSION"));

    let mut wd = Watchdog::new(p.WATCHDOG);
    let reset_reason = match wd.reset_reason() {
        Some(embassy_rp::watchdog::ResetReason::TimedOut) => 2,
        Some(embassy_rp::watchdog::ResetReason::Forced) => 1,
        None => 0,
    };
    wd.start(Duration::from_secs(1));

    // edges on the high-priority executor
    interrupt::SWI_IRQ_1.set_priority(Priority::P2);
    let high = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_1);
    high.spawn(edge_task(Input::new(p.PIN_2, Pull::Up), Src::Wheel).unwrap());
    high.spawn(edge_task(Input::new(p.PIN_3, Pull::Up), Src::Ignition).unwrap());

    let mut i2c_cfg = i2c::Config::default();
    i2c_cfg.frequency = 1_000_000;
    let fram = Fram {
        i2c: I2c::new_blocking(p.I2C0, p.PIN_5, p.PIN_4, i2c_cfg),
    };
    let core = McuCore::new(fram, reset_reason);
    info!(
        "odometer {} m, status {}",
        core.odometer().odo_m(),
        defmt::Debug2Format(&core.odo_status())
    );

    // 25 kHz PWM: inaudible and fine for the TPS61169 dimming input
    let mut pwm_cfg = pwm::Config::default();
    pwm_cfg.top = (embassy_rp::clocks::clk_sys_freq() / 25_000 - 1) as u16;
    pwm_cfg.compare_b = 0;
    let pwm = Pwm::new_output_b(p.PWM_SLICE7, p.PIN_15, pwm_cfg.clone());

    let io = Io {
        inputs: [
            Input::new(p.PIN_6, Pull::Up),
            Input::new(p.PIN_7, Pull::Up),
            Input::new(p.PIN_8, Pull::Up),
            Input::new(p.PIN_9, Pull::Up),
            Input::new(p.PIN_10, Pull::Up),
        ],
        btn_mode: Input::new(p.PIN_11, Pull::Up),
        btn_set: Input::new(p.PIN_12, Pull::Up),
        adc: Adc::new_blocking(p.ADC, adc::Config::default()),
        ch: [
            AdcChannel::new_pin(p.PIN_26, Pull::None),
            AdcChannel::new_pin(p.PIN_27, Pull::None),
            AdcChannel::new_pin(p.PIN_28, Pull::None),
        ],
        pwm,
        pwm_cfg,
        wd,
    };
    spawner.spawn(control(core, io).unwrap());

    // USB CDC-ACM towards the Pi
    let driver = Driver::new(p.USB, Irqs);
    let mut cfg = embassy_usb::Config::new(tdm_proto::USB_VID, tdm_proto::USB_PID);
    cfg.manufacturer = Some("tdm-dash");
    cfg.product = Some(tdm_proto::USB_PRODUCT);
    cfg.serial_number = Some("tdm-io-1");
    cfg.max_power = 100;
    cfg.max_packet_size_0 = 64;
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static CDC_STATE: StaticCell<State> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        cfg,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );
    let class = CdcAcmClass::new(&mut builder, CDC_STATE.init(State::new()), 64);
    let usb = builder.build();
    let (tx, rx) = class.split();
    spawner.spawn(usb_task(usb).unwrap());
    spawner.spawn(usb_rx(rx).unwrap());
    spawner.spawn(usb_tx(tx).unwrap());
}
