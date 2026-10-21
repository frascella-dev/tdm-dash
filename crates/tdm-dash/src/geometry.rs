//! Gauge geometry: dial faces rendered once with tiny-skia (the software
//! renderer cannot rotate), needle polygons and arc segments as SVG paths.

use std::fmt::Write;

use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::theme::Colors;

pub const DIAL: f32 = 340.0;
pub const R: f32 = 160.0;
const C: f32 = DIAL / 2.0;

/// Dial sweep: 270° clockwise through the top, 0 at bottom-left.
pub fn dial_angle(f: f64) -> f64 {
    (135.0 + 270.0 * f).to_radians()
}

fn polar(cx: f64, cy: f64, r: f64, a: f64) -> (f64, f64) {
    (cx + r * a.cos(), cy + r * a.sin())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialKind {
    Speed { max_kmh: u32 },
    Rpm { max_rpm: u32, redline: u32 },
}

pub struct DialLabel {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub red: bool,
}

fn rgba(c: slint::Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(c.red(), c.green(), c.blue(), c.alpha())
}

fn paint(c: slint::Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(rgba(c));
    p.anti_alias = true;
    p
}

fn to_image(pm: &Pixmap) -> Image {
    let buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(pm.data(), pm.width(), pm.height());
    // tiny-skia stores premultiplied alpha
    Image::from_rgba8_premultiplied(buf)
}

/// Tick marks, red zone and (speed) Italian limit markers, drawn on a
/// `px`×`px` image (the dial size in physical pixels) so it is shown 1:1.
/// Numerals are separate Text items so they use the same font as the rest
/// of the UI; their positions are in dial coordinates (340×340).
pub fn dial_face(kind: DialKind, c: &Colors, px: u32) -> (Image, Vec<DialLabel>) {
    let mut pm = Pixmap::new(px, px).expect("pixmap");
    let tf = Transform::from_scale(px as f32 / DIAL, px as f32 / DIAL);
    let (max, minor, major) = match kind {
        DialKind::Speed { max_kmh } => (max_kmh as f64, 10.0, 20.0),
        DialKind::Rpm { max_rpm, .. } => (max_rpm as f64, 250.0, 1000.0),
    };
    let redline = match kind {
        DialKind::Rpm { redline, .. } => redline as f64,
        _ => f64::INFINITY,
    };
    let stroke = |w: f32| Stroke {
        width: w,
        line_cap: tiny_skia::LineCap::Butt,
        ..Stroke::default()
    };

    // outer ring
    let mut pb = PathBuilder::new();
    let steps = 120;
    for i in 0..=steps {
        let (x, y) = polar(
            C as f64,
            C as f64,
            R as f64,
            dial_angle(i as f64 / steps as f64),
        );
        if i == 0 {
            pb.move_to(x as f32, y as f32)
        } else {
            pb.line_to(x as f32, y as f32)
        }
    }
    if let Some(p) = pb.finish() {
        pm.stroke_path(&p, &paint(c.faint), &stroke(2.0), tf, None);
    }
    // red zone band
    if redline.is_finite() {
        let mut pb = PathBuilder::new();
        let f0 = redline / max;
        for i in 0..=40 {
            let f = f0 + (1.0 - f0) * i as f64 / 40.0;
            let (x, y) = polar(C as f64, C as f64, R as f64 - 6.0, dial_angle(f));
            if i == 0 {
                pb.move_to(x as f32, y as f32)
            } else {
                pb.line_to(x as f32, y as f32)
            }
        }
        if let Some(p) = pb.finish() {
            pm.stroke_path(&p, &paint(c.red), &stroke(9.0), tf, None);
        }
    }
    let mut labels = Vec::new();
    let mut v = 0.0;
    while v <= max + 1e-6 {
        let f = v / max;
        let a = dial_angle(f);
        let is_major = (v % major).abs() < 1e-6;
        let is_half = matches!(kind, DialKind::Rpm { .. }) && (v % 500.0).abs() < 1e-6;
        let len = if is_major {
            18.0
        } else if is_half {
            11.0
        } else {
            6.0
        };
        let red = v >= redline;
        let col = if red {
            c.red
        } else if is_major {
            c.ink
        } else {
            c.dim
        };
        let (x0, y0) = polar(C as f64, C as f64, R as f64 - 2.0, a);
        let (x1, y1) = polar(C as f64, C as f64, R as f64 - 2.0 - len, a);
        let mut pb = PathBuilder::new();
        pb.move_to(x0 as f32, y0 as f32);
        pb.line_to(x1 as f32, y1 as f32);
        if let Some(p) = pb.finish() {
            pm.stroke_path(
                &p,
                &paint(col),
                &stroke(if is_major { 3.2 } else { 1.6 }),
                tf,
                None,
            );
        }
        if is_major {
            let (tx, ty) = polar(C as f64, C as f64, R as f64 - 40.0, a);
            let text = match kind {
                DialKind::Speed { .. } => format!("{v}"),
                DialKind::Rpm { .. } => format!("{}", v / 1000.0),
            };
            labels.push(DialLabel {
                x: tx as f32,
                y: ty as f32,
                text,
                red,
            });
        }
        v += minor;
    }
    if let DialKind::Speed { max_kmh } = kind {
        // Italian limits: urban, extra-urban, expressway, motorway
        for lim in [50.0, 90.0, 110.0, 130.0] {
            if lim > max_kmh as f64 {
                continue;
            }
            let a = dial_angle(lim / max_kmh as f64);
            let (x, y) = polar(C as f64, C as f64, R as f64 + 6.0, a);
            let (ux, uy) = (a.cos(), a.sin()); // outward
            let (nx, ny) = (-uy, ux);
            let mut pb = PathBuilder::new();
            pb.move_to((x - ux * 3.0) as f32, (y - uy * 3.0) as f32);
            pb.line_to(
                (x + ux * 4.0 + nx * 4.0) as f32,
                (y + uy * 4.0 + ny * 4.0) as f32,
            );
            pb.line_to(
                (x + ux * 4.0 - nx * 4.0) as f32,
                (y + uy * 4.0 - ny * 4.0) as f32,
            );
            pb.close();
            if let Some(p) = pb.finish() {
                pm.fill_path(&p, &paint(c.needle), FillRule::Winding, tf, None);
            }
        }
    }
    (to_image(&pm), labels)
}

/// Needle polygon in dial coordinates (340×340), with a tight bounding box
/// so only the area around the needle is redrawn.
#[derive(Debug, Clone, PartialEq)]
pub struct Needle {
    pub cmd: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn needle(f: f64) -> Needle {
    let a = dial_angle(f.clamp(-0.01, 1.01));
    let (ca, sa) = (a.cos(), a.sin());
    let (nx, ny) = (-sa, ca);
    let (c, tip, tail, wb, wt) = (C as f64, R as f64 - 8.0, 22.0, 5.5, 1.6);
    let p = [
        (c + ca * tip + nx * wt, c + sa * tip + ny * wt),
        (c + ca * tip - nx * wt, c + sa * tip - ny * wt),
        (c - ca * tail - nx * wb, c - sa * tail - ny * wb),
        (c - ca * tail + nx * wb, c - sa * tail + ny * wb),
    ];
    let mut cmd = String::with_capacity(80);
    let _ = write!(
        cmd,
        "M {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} Z",
        p[0].0, p[0].1, p[1].0, p[1].1, p[2].0, p[2].1, p[3].0, p[3].1
    );
    let x0 = p.iter().map(|q| q.0).fold(f64::INFINITY, f64::min).floor() - 1.0;
    let y0 = p.iter().map(|q| q.1).fold(f64::INFINITY, f64::min).floor() - 1.0;
    let x1 = p
        .iter()
        .map(|q| q.0)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        + 1.0;
    let y1 = p
        .iter()
        .map(|q| q.1)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        + 1.0;
    Needle {
        cmd,
        x: x0 as f32,
        y: y0 as f32,
        w: (x1 - x0) as f32,
        h: (y1 - y0) as f32,
    }
}

/// One arc segment; `cmd` uses absolute screen coordinates and the
/// bounding box keeps the Path item (and its redraw region) tight.
pub struct ArcSeg {
    pub cmd: String,
    pub at: f32,
    pub red: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Sport layout: 50 segments on a wide arc across the top of the screen.
pub fn arc_segments(max_rpm: u32, redline: u32) -> (Vec<ArcSeg>, Vec<DialLabel>) {
    let (cx, cy, r0) = (400.0, 532.0, 430.0);
    let (a0, a1, n) = (220.0f64, 320.0f64, 50);
    let mut segs = Vec::with_capacity(n);
    for i in 0..n {
        let s0 = (a0 + (a1 - a0) * i as f64 / n as f64 + 0.35).to_radians();
        let s1 = (a0 + (a1 - a0) * (i + 1) as f64 / n as f64 - 0.35).to_radians();
        let r1 = 470.0 + 6.0 * (i as f64 / n as f64 * 1.6).min(1.0);
        let (ox0, oy0) = polar(cx, cy, r1, s0);
        let (ox1, oy1) = polar(cx, cy, r1, s1);
        let (ix1, iy1) = polar(cx, cy, r0, s1);
        let (ix0, iy0) = polar(cx, cy, r0, s0);
        let cmd = format!(
            "M {ox0:.1} {oy0:.1} A {r1:.1} {r1:.1} 0 0 1 {ox1:.1} {oy1:.1} L {ix1:.1} {iy1:.1} A {r0:.1} {r0:.1} 0 0 0 {ix0:.1} {iy0:.1} Z"
        );
        let at = (i + 1) as f32 / n as f32;
        // the arc between corners bulges outward by at most r(1 − cos(Δ/2))
        let bulge = r1 * (1.0 - ((s1 - s0) / 2.0).cos());
        let xs = [ox0, ox1, ix0, ix1];
        let ys = [oy0, oy1, iy0, iy1];
        let x0 = xs.iter().cloned().fold(f64::INFINITY, f64::min) - bulge - 1.0;
        let x1 = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + bulge + 1.0;
        let y0 = ys.iter().cloned().fold(f64::INFINITY, f64::min) - bulge - 1.0;
        let y1 = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + bulge + 1.0;
        segs.push(ArcSeg {
            cmd,
            at,
            red: at * max_rpm as f32 > redline as f32,
            x: x0.floor() as f32,
            y: y0.floor() as f32,
            w: (x1 - x0.floor()).ceil() as f32,
            h: (y1 - y0.floor()).ceil() as f32,
        });
    }
    let labels = (0..=max_rpm / 1000)
        .map(|k| {
            let a = (a0 + (a1 - a0) * (k * 1000) as f64 / max_rpm as f64).to_radians();
            let (x, y) = polar(cx, cy, r0 - 18.0, a);
            DialLabel {
                x: x as f32,
                y: y as f32 + 4.0,
                text: k.to_string(),
                red: k * 1000 >= redline,
            }
        })
        .collect();
    (segs, labels)
}

/// A numeral on a straight rpm scale: thousands of rpm at `frac` of the
/// scale; `major` marks the even thousands, kept where the scale is short.
pub struct Mark {
    pub frac: f32,
    pub text: String,
    pub red: bool,
    pub major: bool,
}

pub fn rpm_marks(max_rpm: u32, redline: u32) -> Vec<Mark> {
    (0..=max_rpm / 1000)
        .map(|k| Mark {
            frac: (k * 1000) as f32 / max_rpm as f32,
            text: k.to_string(),
            red: k * 1000 >= redline,
            major: k % 2 == 0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needle_points_up_at_half_scale() {
        let n = needle(0.5);
        assert!(n.y < 20.0 && n.h < 200.0 && n.w < 20.0);
        let p = n.cmd;
        // tip is above the centre
        let nums: Vec<f64> = p.split(' ').filter_map(|t| t.parse().ok()).collect();
        assert!(nums[1] < 30.0 && (nums[0] - 170.0).abs() < 3.0, "{p}");
    }

    #[test]
    fn arc_has_red_zone_at_the_end() {
        let (segs, labels) = arc_segments(10_000, 8_000);
        assert_eq!(segs.len(), 50);
        assert_eq!(segs.iter().filter(|s| s.red).count(), 10);
        assert_eq!(labels.len(), 11);
        // the numerals follow max_rpm: 0–12 over the same arc
        let (_, labels) = arc_segments(12_000, 10_500);
        assert_eq!(labels.len(), 13);
        assert!(labels[11].red && !labels[10].red);
    }

    #[test]
    fn rpm_marks_follow_max_rpm() {
        let m = rpm_marks(12_000, 10_000);
        assert_eq!(m.len(), 13);
        assert_eq!((m[6].frac, m[6].major, m[10].red), (0.5, true, true));
        assert!(!m[9].red && !m[7].major);
    }

    #[test]
    fn dial_face_is_drawn_at_the_requested_size() {
        let c = crate::theme::colors(tdm_logic::Theme::Day);
        let (img, labels) = dial_face(DialKind::Speed { max_kmh: 220 }, c, 510);
        assert_eq!((img.size().width, img.size().height), (510, 510));
        // numerals stay in dial coordinates
        assert!(labels.iter().all(|l| (0.0..DIAL).contains(&l.x)));
    }
}
