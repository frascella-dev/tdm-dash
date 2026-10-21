//! Fitting the UI to the panel (docs/adr/0013): the layout family from the
//! aspect ratio, the scale from the size, and the logical canvas that
//! dash.slint lays out in.

use slint::PhysicalSize;
use slint::platform::WindowEvent;
use slint::platform::software_renderer::RenderingRotation;
use tdm_logic::Form;

/// Panels at least this many times wider than tall get the bar layouts.
pub const BAR_FROM: f32 = 2.2;
/// Smallest logical canvas of each family: the scale makes the panel cover
/// it, and the extra space goes to the side or around the centre.
const WIDE_MIN: (f32, f32) = (800.0, 480.0);
const BAR_MIN: (f32, f32) = (1200.0, 400.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Wide,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub family: Family,
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// Size of the picture as the rider sees it, after the rotation.
    pub physical: (u32, u32),
    pub logical: (f32, f32),
}

/// `w`×`h` is the panel mode; `rotation` is clockwise in degrees.
pub fn fit(w: u32, h: u32, rotation: u16, form: Form, scale: Option<f64>) -> Fit {
    let (w, h) = if rotation % 180 == 90 { (h, w) } else { (w, h) };
    let (wf, hf) = (w.max(1) as f32, h.max(1) as f32);
    let family = match form {
        Form::Auto if wf / hf >= BAR_FROM => Family::Bar,
        Form::Auto | Form::Wide => Family::Wide,
        Form::Bar => Family::Bar,
    };
    let (mw, mh) = match family {
        Family::Wide => WIDE_MIN,
        Family::Bar => BAR_MIN,
    };
    let scale = scale.map_or((wf / mw).min(hf / mh), |s| s as f32);
    Fit {
        family,
        scale,
        physical: (w, h),
        logical: (wf / scale, hf / scale),
    }
}

/// Gives the window the fitted scale and size.
pub fn apply(window: &slint::Window, fit: &Fit) {
    window.dispatch_event(WindowEvent::ScaleFactorChanged {
        scale_factor: fit.scale,
    });
    window.set_size(PhysicalSize::new(fit.physical.0, fit.physical.1));
}

/// The renderer setting for a clockwise rotation in degrees.
pub fn rendering_rotation(degrees: u16) -> RenderingRotation {
    match degrees {
        90 => RenderingRotation::Rotate90,
        180 => RenderingRotation::Rotate180,
        270 => RenderingRotation::Rotate270,
        _ => RenderingRotation::NoRotation,
    }
}

/// Parses `WIDTHxHEIGHT`.
pub fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let bad = || format!("size '{s}' is not WIDTHxHEIGHT");
    let (w, h) = s.split_once('x').ok_or_else(bad)?;
    let (w, h) = (w.parse().map_err(|_| bad())?, h.parse().map_err(|_| bad())?);
    if !(64..=8192).contains(&w) || !(64..=8192).contains(&h) {
        return Err(bad());
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auto(w: u32, h: u32) -> Fit {
        fit(w, h, 0, Form::Auto, None)
    }

    fn close(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 0.5 && (a.1 - b.1).abs() < 0.5
    }

    #[test]
    fn wide_panels() {
        for (w, h, scale, logical) in [
            (800, 480, 1.0, (800.0, 480.0)),
            (1024, 600, 1.25, (819.2, 480.0)),
            (1280, 720, 1.5, (853.3, 480.0)),
            (1280, 800, 1.6, (800.0, 500.0)),
            (1024, 768, 1.28, (800.0, 600.0)),
            (480, 272, 0.5667, (847.1, 480.0)),
        ] {
            let f = auto(w, h);
            assert_eq!(f.family, Family::Wide, "{w}x{h}");
            assert!((f.scale - scale).abs() < 1e-3, "{w}x{h}: {}", f.scale);
            assert!(close(f.logical, logical), "{w}x{h}: {:?}", f.logical);
        }
    }

    #[test]
    fn bar_panels() {
        for (w, h, scale, logical) in [
            (1280, 400, 1.0, (1280.0, 400.0)),
            (1920, 480, 1.2, (1600.0, 400.0)),
            (1920, 720, 1.6, (1200.0, 450.0)),
            (1280, 320, 0.8, (1600.0, 400.0)),
        ] {
            let f = auto(w, h);
            assert_eq!(f.family, Family::Bar, "{w}x{h}");
            assert!((f.scale - scale).abs() < 1e-3, "{w}x{h}: {}", f.scale);
            assert!(close(f.logical, logical), "{w}x{h}: {:?}", f.logical);
        }
    }

    #[test]
    fn rotation_swaps_the_sides() {
        let f = fit(400, 1280, 90, Form::Auto, None);
        assert_eq!((f.family, f.physical), (Family::Bar, (1280, 400)));
        assert_eq!(fit(400, 1280, 270, Form::Auto, None).physical, (1280, 400));
        assert_eq!(fit(800, 480, 180, Form::Auto, None).physical, (800, 480));
    }

    #[test]
    fn profile_overrides() {
        let f = fit(1280, 720, 0, Form::Bar, None);
        assert_eq!(f.family, Family::Bar);
        assert!(close(f.logical, (1200.0, 675.0)));
        let f = fit(1280, 400, 0, Form::Wide, Some(0.8));
        assert_eq!((f.family, f.scale), (Family::Wide, 0.8));
        assert!(close(f.logical, (1600.0, 500.0)));
    }

    #[test]
    fn sizes_parse() {
        assert_eq!(parse_size("1280x400"), Ok((1280, 400)));
        assert!(parse_size("1280").is_err() && parse_size("0x400").is_err());
    }
}
