//! Off-screen rendering with the software renderer: screenshots, the render
//! benchmark and tests. The clock is driven by the caller, so animations and
//! timers advance with simulated time.

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::platform::software_renderer::{
    MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel, TargetPixel,
};
use slint::platform::{Platform, WindowAdapter};
use slint::{PlatformError, Rgb8Pixel};
use tdm_logic::Form;

use crate::screen::{self, Fit};

pub struct Headless {
    pub window: Rc<MinimalSoftwareWindow>,
    clock: Rc<Cell<Duration>>,
    base: Cell<Duration>,
    /// Panel size in buffer pixels (before the rotation) and the rotation.
    panel: Cell<(usize, usize)>,
    rotation: Cell<u16>,
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
    /// Installs the headless platform with an 800×480 panel; call once per
    /// process, before creating any window.
    pub fn install(repaint: RepaintBufferType) -> Self {
        let window = MinimalSoftwareWindow::new(repaint);
        let clock = Rc::new(Cell::new(Duration::ZERO));
        slint::platform::set_platform(Box::new(HeadlessPlatform {
            window: window.clone(),
            clock: clock.clone(),
        }))
        .expect("a Slint platform was already installed");
        let h = Self {
            window,
            clock,
            base: Cell::new(Duration::ZERO),
            panel: Cell::new((0, 0)),
            rotation: Cell::new(0),
        };
        h.set_panel(800, 480, 0, Form::Auto, None);
        h
    }

    /// Simulates a panel of `w`×`h` pixels mounted with `rotation` degrees
    /// clockwise; returns how the UI fits on it.
    pub fn set_panel(&self, w: u32, h: u32, rotation: u16, form: Form, scale: Option<f64>) -> Fit {
        let fit = screen::fit(w, h, rotation, form, scale);
        self.panel.set((w as usize, h as usize));
        self.rotation.set(rotation);
        screen::apply(self.window.window(), &fit);
        self.window.request_redraw();
        fit
    }

    /// Panel width and height in pixels: the size of the buffers.
    pub fn panel(&self) -> (usize, usize) {
        self.panel.get()
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
    pub fn draw<P: TargetPixel>(&self, buf: &mut [P]) -> Option<Duration> {
        let mut spent = None;
        let stride = self.panel.get().0;
        let rotation = screen::rendering_rotation(self.rotation.get());
        self.window.draw_if_needed(|r| {
            r.set_rendering_rotation(rotation);
            let t = Instant::now();
            r.render(buf, stride);
            spent = Some(t.elapsed());
        });
        spent
    }

    pub fn redraw_all(&self) {
        self.window.request_redraw();
    }

    pub fn rgb_buffer(&self) -> Vec<Rgb8Pixel> {
        let (w, h) = self.panel.get();
        vec![Rgb8Pixel::new(0, 0, 0); w * h]
    }

    pub fn rgb565_buffer(&self) -> Vec<Rgb565Pixel> {
        let (w, h) = self.panel.get();
        vec![Rgb565Pixel::default(); w * h]
    }
}

/// Saves a `w`×`h` frame.
pub fn save_png(path: &Path, buf: &[Rgb8Pixel], w: usize, h: usize) -> std::io::Result<()> {
    let f = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header().map_err(std::io::Error::other)?;
    let bytes: Vec<u8> = buf.iter().flat_map(|p| [p.r, p.g, p.b]).collect();
    wr.write_image_data(&bytes).map_err(std::io::Error::other)?;
    Ok(())
}
