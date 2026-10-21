//! Direct DRM/KMS output for the Raspberry Pi (docs/adr/0003): two dumb
//! buffers, the software renderer draws into the back one, then a page flip
//! synchronised to vblank. No X11, Wayland, udev, libinput or seat daemon.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::{AsFd, BorrowedFd};
use std::rc::Rc;
use std::time::{Duration, Instant};

use drm::buffer::Buffer as _;
use drm::buffer::DrmFourcc;
use drm::control::dumbbuffer::DumbBuffer;
use drm::control::{Device as ControlDevice, PageFlipFlags, connector, crtc, framebuffer};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RenderingRotation, RepaintBufferType,
    TargetPixel,
};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, PlatformError};
use tdm_link::SourceSpec;

use crate::app::App;
use crate::{AppWindow, Options};

struct Card(File);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
impl drm::Device for Card {}
impl ControlDevice for Card {}

/// One pixel of a DRM_FORMAT_XRGB8888 buffer (little endian: B, G, R, X).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Xrgb8888 {
    b: u8,
    g: u8,
    r: u8,
    x: u8,
}

impl TargetPixel for Xrgb8888 {
    fn blend(&mut self, c: PremultipliedRgbaColor) {
        let a = (u8::MAX - c.alpha) as u16;
        self.r = (self.r as u16 * a / 255) as u8 + c.red;
        self.g = (self.g as u16 * a / 255) as u8 + c.green;
        self.b = (self.b as u16 * a / 255) as u8 + c.blue;
    }
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { b, g, r, x: 0xff }
    }
}

struct KmsPlatform {
    window: Rc<MinimalSoftwareWindow>,
    t0: Instant,
}

impl Platform for KmsPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        self.t0.elapsed()
    }
}

struct Output {
    card: Card,
    conn: connector::Handle,
    crtc: crtc::Handle,
    mode: drm::control::Mode,
    bufs: [DumbBuffer; 2],
    fbs: [framebuffer::Handle; 2],
}

fn open_output() -> Result<Output, String> {
    let mut last_err = String::from("no /dev/dri/card*");
    for i in 0..4 {
        let path = format!("/dev/dri/card{i}");
        let Ok(f) = OpenOptions::new().read(true).write(true).open(&path) else {
            continue;
        };
        let card = Card(f);
        match setup(card) {
            Ok(o) => return Ok(o),
            Err(e) => last_err = format!("{path}: {e}"),
        }
    }
    Err(last_err)
}

fn setup(card: Card) -> Result<Output, String> {
    let e = |x: std::io::Error| x.to_string();
    let res = card.resource_handles().map_err(e)?;
    let conns: Vec<connector::Info> = res
        .connectors()
        .iter()
        .filter_map(|c| card.get_connector(*c, true).ok())
        .collect();
    // prefer the DPI panel, then anything connected
    let con = conns
        .iter()
        .filter(|c| c.state() == connector::State::Connected && !c.modes().is_empty())
        .min_by_key(|c| {
            if c.interface() == connector::Interface::DPI {
                0
            } else {
                1
            }
        })
        .ok_or("no connected connector")?;
    let mode = con.modes()[0];
    let crtc = con
        .current_encoder()
        .and_then(|h| card.get_encoder(h).ok())
        .and_then(|enc| enc.crtc())
        .or_else(|| {
            con.encoders()
                .iter()
                .filter_map(|h| card.get_encoder(*h).ok())
                .find_map(|enc| res.filter_crtcs(enc.possible_crtcs()).first().copied())
        })
        .ok_or("no CRTC for the connector")?;
    let (w, h) = mode.size();
    let mk = || -> Result<(DumbBuffer, framebuffer::Handle), String> {
        let mut db = card
            .create_dumb_buffer((w as u32, h as u32), DrmFourcc::Xrgb8888, 32)
            .map_err(e)?;
        {
            let mut map = card.map_dumb_buffer(&mut db).map_err(e)?;
            map.as_mut().fill(0);
        }
        let fb = card.add_framebuffer(&db, 24, 32).map_err(e)?;
        Ok((db, fb))
    };
    let (b0, f0) = mk()?;
    let (b1, f1) = mk()?;
    card.set_crtc(crtc, Some(f0), (0, 0), &[con.handle()], Some(mode))
        .map_err(e)?;
    Ok(Output {
        conn: con.handle(),
        crtc,
        mode,
        bufs: [b0, b1],
        fbs: [f0, f1],
        card,
    })
}

/// Feeds the hardware watchdog; if the UI loop hangs, the Pi reboots.
struct Watchdog(Option<File>);

impl Watchdog {
    fn open() -> Self {
        Self(OpenOptions::new().write(true).open("/dev/watchdog").ok())
    }
    fn kick(&mut self) {
        if let Some(f) = &mut self.0 {
            let _ = f.write_all(b"k");
        }
    }
}

pub fn run(o: &Options) -> Result<(), String> {
    let mut out = open_output()?;
    let (w, h) = out.mode.size();
    let window = MinimalSoftwareWindow::new(RepaintBufferType::SwappedBuffers);
    window.set_size(PhysicalSize::new(w as u32, h as u32));
    let t0 = Instant::now();
    slint::platform::set_platform(Box::new(KmsPlatform {
        window: window.clone(),
        t0,
    }))
    .map_err(|e| e.to_string())?;

    let dash = crate::make_dash(o);
    let rotate = dash.profile().display.rotate_180;
    let source = crate::open_source(o, &dash, SourceSpec::Serial(None))?;
    let ui = AppWindow::new().map_err(|e| e.to_string())?;
    ui.show().map_err(|e| e.to_string())?;
    let mut app = App::new(ui.clone_strong(), dash, source);
    let mut wd = Watchdog::open();
    let mut back = 1usize;
    let mut shown_first = false;
    let frame = Duration::from_micros(16_667);
    eprintln!("tdm-dash: KMS {}x{} su connettore {:?}", w, h, out.conn);

    loop {
        let tick = Instant::now();
        let now = t0.elapsed().as_secs_f64();
        app.frame(now);
        slint::platform::update_timers_and_animations();
        let mut drawn = false;
        let card = &out.card;
        let buf = &mut out.bufs[back];
        window.draw_if_needed(|r| {
            if rotate {
                r.set_rendering_rotation(RenderingRotation::Rotate180);
            }
            let pitch = buf.pitch() as usize / 4;
            if let Ok(mut map) = card.map_dumb_buffer(buf) {
                let bytes = map.as_mut();
                let n = bytes.len() / 4;
                // SAFETY: the mapping is page aligned, Xrgb8888 is 4 plain
                // bytes with alignment 1, and `n` pixels fit in `bytes`.
                let px = unsafe {
                    std::slice::from_raw_parts_mut(bytes.as_mut_ptr().cast::<Xrgb8888>(), n)
                };
                r.render(px, pitch);
                drawn = true;
            }
        });
        if drawn {
            out.card
                .page_flip(out.crtc, out.fbs[back], PageFlipFlags::EVENT, None)
                .map_err(|e| e.to_string())?;
            // block until the flip completes (vblank)
            let _ = out.card.receive_events().map(|ev| ev.count());
            back = 1 - back;
            if !shown_first {
                shown_first = true;
                app.dash.first_frame_shown(t0.elapsed().as_secs_f64());
            }
        }
        wd.kick();
        if let Some(rest) = frame.checked_sub(tick.elapsed()) {
            std::thread::sleep(rest);
        }
    }
}
