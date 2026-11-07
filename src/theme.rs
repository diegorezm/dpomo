//! Color themes. To add a new built-in theme, add one entry to [`PRESETS`].

use egui::{Color32, CornerRadius, Stroke, Visuals};
use serde::{Deserialize, Serialize};

use crate::timer::Phase;

pub const CUSTOM: &str = "custom";

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    #[serde(with = "hex")]
    pub background: Color32,
    /// Cards, buttons and the empty part of the ring.
    #[serde(with = "hex")]
    pub surface: Color32,
    #[serde(with = "hex")]
    pub text: Color32,
    #[serde(with = "hex")]
    pub muted: Color32,
    #[serde(with = "hex")]
    pub focus: Color32,
    #[serde(with = "hex")]
    pub short_break: Color32,
    #[serde(with = "hex")]
    pub long_break: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        PRESETS[0].1
    }
}

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

const fn theme(bg: u32, surface: u32, text: u32, muted: u32, f: u32, s: u32, l: u32) -> Theme {
    Theme {
        background: rgb(bg),
        surface: rgb(surface),
        text: rgb(text),
        muted: rgb(muted),
        focus: rgb(f),
        short_break: rgb(s),
        long_break: rgb(l),
    }
}

/// Built-in themes. The first one is the default.
pub const PRESETS: &[(&str, Theme)] = &[
    (
        "Rosé Pine",
        theme(
            0x191724, 0x26233a, 0xe0def4, 0x908caa, 0xeb6f92, 0x9ccfd8, 0xc4a7e7,
        ),
    ),
    (
        "Catppuccin",
        theme(
            0x1e1e2e, 0x313244, 0xcdd6f4, 0xa6adc8, 0xf38ba8, 0xa6e3a1, 0x89b4fa,
        ),
    ),
    (
        "Tokyo Night",
        theme(
            0x1a1b26, 0x24283b, 0xc0caf5, 0x9aa5ce, 0xf7768e, 0x9ece6a, 0x7aa2f7,
        ),
    ),
    (
        "Nord",
        theme(
            0x2e3440, 0x3b4252, 0xeceff4, 0xa3acbb, 0xbf616a, 0xa3be8c, 0x88c0d0,
        ),
    ),
    (
        "Gruvbox",
        theme(
            0x282828, 0x3c3836, 0xebdbb2, 0xa89984, 0xfb4934, 0xb8bb26, 0x83a598,
        ),
    ),
    (
        "Mono",
        theme(
            0x111111, 0x222222, 0xf2f2f2, 0x8a8a8a, 0xf2f2f2, 0xbdbdbd, 0x9a9a9a,
        ),
    ),
    (
        "Latte",
        theme(
            0xeff1f5, 0xdce0e8, 0x4c4f69, 0x6c6f85, 0xd20f39, 0x40a02b, 0x1e66f5,
        ),
    ),
    (
        "Paper",
        theme(
            0xf7f5f0, 0xe9e5dc, 0x2b2b2b, 0x7a7468, 0xe4572e, 0x2a9d8f, 0x6c63ff,
        ),
    ),
];

pub fn preset(name: &str) -> Option<Theme> {
    PRESETS.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
}

impl Theme {
    pub fn accent(&self, phase: Phase) -> Color32 {
        match phase {
            Phase::Focus => self.focus,
            Phase::ShortBreak => self.short_break,
            Phase::LongBreak => self.long_break,
        }
    }

    pub fn is_dark(&self) -> bool {
        let [r, g, b, _] = self.background.to_array();
        (0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b)) < 128.0
    }

    /// Push this theme into egui's global style. Cheap, but only call it when
    /// the theme actually changes.
    pub fn apply(&self, ctx: &egui::Context) {
        let dark = self.is_dark();
        let mut v = if dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        v.panel_fill = self.background;
        v.window_fill = self.surface;
        v.extreme_bg_color = self.surface;
        v.faint_bg_color = self.surface;
        v.code_bg_color = self.surface;
        v.override_text_color = Some(self.text);
        v.hyperlink_color = self.focus;
        v.selection.bg_fill = self.focus.gamma_multiply(0.45);
        v.selection.stroke = Stroke::new(1.0, self.text);
        v.window_corner_radius = CornerRadius::same(14);
        v.window_stroke = Stroke::new(1.0, mix(self.surface, self.text, 0.08));

        let radius = CornerRadius::same(8);
        let w = &mut v.widgets;
        w.noninteractive.bg_fill = self.background;
        w.noninteractive.weak_bg_fill = self.background;
        w.noninteractive.bg_stroke = Stroke::new(1.0, self.surface);
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.text);
        for (state, t) in [
            (&mut w.inactive, 0.07),
            (&mut w.hovered, 0.14),
            (&mut w.active, 0.20),
            (&mut w.open, 0.14),
        ] {
            let fill = mix(self.surface, self.text, t);
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.bg_stroke = Stroke::NONE;
            state.fg_stroke = Stroke::new(1.5, self.text);
            state.corner_radius = radius;
            state.expansion = 0.0;
        }
        // Slider rails, checkbox boxes, etc. use the inactive bg_fill.
        w.inactive.bg_fill = mix(self.surface, self.text, 0.14);

        let egui_theme = if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.set_theme(egui_theme);
        ctx.set_visuals_of(egui_theme, v);
        ctx.all_styles_mut(|s| {
            s.spacing.item_spacing = egui::vec2(8.0, 8.0);
            s.spacing.button_padding = egui::vec2(10.0, 5.0);
            s.spacing.interact_size.y = 26.0;
        });
    }
}

/// Linear blend between two colors, `t` in 0..=1.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

/// Serialize colors as `"#rrggbb"` so the config file stays readable.
mod hex {
    use egui::Color32;
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(c: &Color32, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Color32, D::Error> {
        let s = String::deserialize(d)?;
        let digits = s.trim_start_matches('#');
        let n = u32::from_str_radix(digits, 16)
            .ok()
            .filter(|_| digits.len() == 6)
            .ok_or_else(|| {
                D::Error::custom(format!("invalid color {s:?}, expected \"#rrggbb\""))
            })?;
        Ok(super::rgb(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_names_are_unique_and_not_custom() {
        for (i, (name, _)) in PRESETS.iter().enumerate() {
            assert_ne!(*name, CUSTOM);
            assert!(
                PRESETS[i + 1..].iter().all(|(n, _)| n != name),
                "duplicate {name}"
            );
        }
    }

    #[test]
    fn light_presets_are_detected() {
        assert!(preset("Rosé Pine").unwrap().is_dark());
        assert!(!preset("Latte").unwrap().is_dark());
    }
}
