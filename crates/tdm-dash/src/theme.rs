//! Day and night palettes (design/README.md, "Token").

use slint::{Color, ComponentHandle};
use tdm_logic::Theme;

use crate::{AppWindow, Palette};

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub bg: Color,
    pub panel: Color,
    pub ink: Color,
    pub dim: Color,
    pub faint: Color,
    pub ghost: Color,
    pub needle: Color,
    pub red: Color,
    pub red_dim: Color,
    pub amber: Color,
    pub amber_bg: Color,
    pub green: Color,
    pub blue: Color,
    pub crit_bg: Color,
    pub crit_ink: Color,
    pub caution_ink: Color,
}

const fn hex(v: u32) -> Color {
    Color::from_rgb_u8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

const fn hexa(v: u32, a: u8) -> Color {
    Color::from_argb_u8(a, (v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub const DAY: Colors = Colors {
    bg: hex(0xeceae4),
    panel: hex(0xdcd9d1),
    ink: hex(0x0f1113),
    dim: hex(0x575b5d),
    faint: hex(0xbdb9b0),
    ghost: hexa(0x0f1113, 28),
    needle: hex(0xd9410b),
    red: hex(0xcc1f1a),
    red_dim: hexa(0xcc1f1a, 77),
    amber: hex(0xb86e00),
    amber_bg: hex(0xf2a516),
    green: hex(0x0b9a3b),
    blue: hex(0x1849e6),
    crit_bg: hex(0xc41c17),
    crit_ink: hex(0xffffff),
    caution_ink: hex(0x1b1300),
};

pub const NIGHT: Colors = Colors {
    bg: hex(0x050607),
    panel: hex(0x0e1113),
    ink: hex(0xe8e2d6),
    dim: hex(0x8b877e),
    faint: hex(0x2f3234),
    ghost: hexa(0xe8e2d6, 23),
    needle: hex(0xff6a1a),
    red: hex(0xc3332b),
    red_dim: hexa(0xc3332b, 82),
    amber: hex(0xd9951c),
    amber_bg: hex(0xc98a14),
    green: hex(0x2bb655),
    blue: hex(0x3b6ff0),
    crit_bg: hex(0x9e211b),
    crit_ink: hex(0xfff4ec),
    caution_ink: hex(0x140d00),
};

pub fn colors(t: Theme) -> &'static Colors {
    match t {
        Theme::Day => &DAY,
        Theme::Night => &NIGHT,
    }
}

pub fn apply(ui: &AppWindow, c: &Colors) {
    let p = ui.global::<Palette>();
    p.set_bg(c.bg);
    p.set_panel(c.panel);
    p.set_ink(c.ink);
    p.set_dim(c.dim);
    p.set_faint(c.faint);
    p.set_ghost(c.ghost);
    p.set_needle(c.needle);
    p.set_red(c.red);
    p.set_red_dim(c.red_dim);
    p.set_amber(c.amber);
    p.set_amber_bg(c.amber_bg);
    p.set_green(c.green);
    p.set_blue(c.blue);
    p.set_crit_bg(c.crit_bg);
    p.set_crit_ink(c.crit_ink);
    p.set_caution_ink(c.caution_ink);
}
