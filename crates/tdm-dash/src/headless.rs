//! Off-screen rendering with the software renderer: screenshots, the render
//! benchmark and tests. The clock is driven by the caller, so animations and
//! timers advance with simulated time.

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel};
use slint::platform::{Platform, WindowAdapter};
use slint::{PhysicalSize, PlatformError, Rgb8Pixel};

pub const W: usize = 800;
pub const H: usize = 480;

pub struct Headless {
    pub window: Rc<MinimalSoftwareWindow>,
    clock: Rc<Cell<Duration>>,
    base: Cell<Duration>,
}

struct HeadlessPlatform {
    window: Rc<MinimalSoftwareWindow>,
    clock: Rc<Cell<Duration>>,
}

impl Platform for HeadlessPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        self.clock.get()
    }
}

impl Headless {
    /// Installs the headless platform; call once per process, before
    /// creating any window.
    pub fn install(repaint: RepaintBufferType) -> Self {
        let window = MinimalSoftwareWindow::new(repaint);
        window.set_size(PhysicalSize::new(W as u32, H as u32));
        let clock = Rc::new(Cell::new(Duration::ZERO));
        slint::platform::set_platform(Box::new(HeadlessPlatform {
            window: window.clone(),
            clock: clock.clone(),
        }))
        .expect("a Slint platform was already installed");
        Self {
            window,
            clock,
            base: Cell::new(Duration::ZERO),
        }
    }

    /// Sets the clock to `t` seconds after the last [`Headless::rebase`];
    /// Slint requires the platform clock to be monotonic.
    pub fn set_time(&self, t: f64) {
        let c = (self.base.get() + Duration::from_secs_f64(t)).max(self.clock.get());
        self.clock.set(c);
        slint::platform::update_timers_and_animations();
    }

    /// Starts a new time origin at the current clock (one per screenshot).
    pub fn rebase(&self) {
        self.base.set(self.clock.get() + Duration::from_secs(1));
    }

    /// Renders if anything changed; returns the time spent drawing.
    pub fn draw<P: slint::platform::software_renderer::TargetPixel>(
        &self,
        buf: &mut [P],
    ) -> Option<Duration> {
        let mut spent = None;
        self.window.draw_if_needed(|r| {
            let t = Instant::now();
            r.render(buf, W);
            spent = Some(t.elapsed());
        });
        spent
    }

    pub fn redraw_all(&self) {
        self.window.request_redraw();
    }
}

pub fn save_png(path: &Path, buf: &[Rgb8Pixel]) -> std::io::Result<()> {
    let f = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), W as u32, H as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(std::io::Error::other)?;
    let bytes: Vec<u8> = buf.iter().flat_map(|p| [p.r, p.g, p.b]).collect();
    w.write_image_data(&bytes).map_err(std::io::Error::other)?;
    Ok(())
}

pub fn rgb_buffer() -> Vec<Rgb8Pixel> {
    vec![Rgb8Pixel::new(0, 0, 0); W * H]
}

pub fn rgb565_buffer() -> Vec<Rgb565Pixel> {
    vec![Rgb565Pixel::default(); W * H]
}
