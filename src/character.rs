//! Character packs: one SVG per pose, recolored with the theme at runtime.
//!
//! Built-ins live in `characters/<id>/` (embedded by `build.rs`); users add
//! their own folders to `<config dir>/dpomo/characters/<id>/`.
//!
//! A pack folder holds `idle.svg` (required), `focus.svg`, `short_break.svg`,
//! `long_break.svg` (each falls back to `idle.svg`) and an optional
//! `character.toml` with `name = "Display name"` and optional translations
//! in a `[names]` table (`"pt-BR" = "Nome"`).
//!
//! Recoloring: these exact colors in the SVG are replaced when loading:
//! - `#e05a4f` → the current phase color (accent)
//! - `#7a6464` → the theme's `muted` color (loose marks: zzz, steam…)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::Color32;
use serde::Deserialize;

use crate::timer::{Phase, Timer};

mod builtin {
    include!(concat!(env!("OUT_DIR"), "/builtin_characters.rs"));
}

pub const DEFAULT_ID: &str = "axolotl";
pub const ACCENT_SENTINEL: &str = "#e05a4f";
pub const INK_SENTINEL: &str = "#7a6464";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pose {
    Idle,
    Focus,
    ShortBreak,
    LongBreak,
}

impl Pose {
    pub const ALL: [Pose; 4] = [Pose::Idle, Pose::Focus, Pose::ShortBreak, Pose::LongBreak];

    pub fn file_name(self) -> &'static str {
        match self {
            Pose::Idle => "idle.svg",
            Pose::Focus => "focus.svg",
            Pose::ShortBreak => "short_break.svg",
            Pose::LongBreak => "long_break.svg",
        }
    }

    /// Working while focus runs, resting during breaks, waving otherwise.
    pub fn for_timer(timer: &Timer) -> Pose {
        match timer.phase() {
            Phase::Focus if timer.is_running() => Pose::Focus,
            Phase::Focus => Pose::Idle,
            Phase::ShortBreak => Pose::ShortBreak,
            Phase::LongBreak => Pose::LongBreak,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Character {
    pub id: String,
    /// Default display name (`name` in character.toml, else the folder name).
    pub name: String,
    /// Translated names from the `[names]` table, keyed by language code.
    names: HashMap<String, String>,
    /// SVG source per pose, indexed like `Pose::ALL`.
    poses: [Arc<str>; 4],
}

impl Character {
    /// Build from `(file name, bytes)` pairs.
    fn from_files<'a>(
        id: &str,
        files: impl IntoIterator<Item = (&'a str, Vec<u8>)>,
    ) -> Result<Self, String> {
        let mut svgs: [Option<Arc<str>>; 4] = Default::default();
        let mut name = None;
        let mut names = HashMap::new();
        for (file, bytes) in files {
            let text = String::from_utf8(bytes)
                .map_err(|_| crate::i18n::fill(crate::i18n::tr().not_utf8, &[("file", file)]))?;
            if file == "character.toml" {
                #[derive(Deserialize)]
                struct Meta {
                    name: Option<String>,
                    #[serde(default)]
                    names: HashMap<String, String>,
                }
                let meta: Meta =
                    toml::from_str(&text).map_err(|e| format!("{file}: {}", e.message()))?;
                name = meta.name;
                names = meta.names;
            } else if let Some(i) = Pose::ALL.iter().position(|p| p.file_name() == file) {
                svgs[i] = Some(text.into());
            }
        }
        let idle = svgs[0].clone().ok_or(crate::i18n::tr().missing_idle)?;
        Ok(Self {
            id: id.to_owned(),
            name: name.unwrap_or_else(|| id.to_owned()),
            names,
            poses: svgs.map(|s| s.unwrap_or_else(|| idle.clone())),
        })
    }

    /// Name in the active UI language, falling back to `name`.
    pub fn display_name(&self) -> &str {
        self.name_in(crate::i18n::current().code())
    }

    /// Name for a language code ("pt-BR"…), falling back to `name`.
    pub fn name_in(&self, code: &str) -> &str {
        self.names.get(code).unwrap_or(&self.name)
    }

    /// The pose's SVG with theme colors applied.
    pub fn svg(&self, pose: Pose, accent: Color32, ink: Color32) -> String {
        let i = Pose::ALL.iter().position(|p| *p == pose).unwrap_or(0);
        self.poses[i]
            .replace(ACCENT_SENTINEL, &hex(accent))
            .replace(INK_SENTINEL, &hex(ink))
    }
}

fn hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

#[derive(Clone, Debug, Default)]
pub struct Characters {
    list: Vec<Character>,
    pub errors: Vec<String>,
}

impl Characters {
    pub fn dir() -> Option<PathBuf> {
        crate::config::Config::path().and_then(|p| Some(p.parent()?.join("characters")))
    }

    pub fn load() -> Self {
        let mut chars = Self::builtin();
        if let Some(dir) = Self::dir() {
            chars.load_dir(&dir);
        }
        chars
    }

    /// Only the characters embedded in the binary.
    pub fn builtin() -> Self {
        let mut chars = Self::default();
        for (id, files) in builtin::BUILTIN {
            let files = files.iter().map(|(n, b)| (*n, b.to_vec()));
            match Character::from_files(id, files) {
                Ok(c) => chars.insert(c),
                Err(e) => chars
                    .errors
                    .push(format!("{id} ({}): {e}", crate::i18n::tr().builtin)),
            }
        }
        chars
    }

    fn load_dir(&mut self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for pack in entries
            .filter_map(|e| Some(e.ok()?.path()))
            .filter(|p| p.is_dir())
        {
            let id = pack
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let files: Vec<(String, Vec<u8>)> = std::fs::read_dir(&pack)
                .into_iter()
                .flatten()
                .filter_map(|e| {
                    let path = e.ok()?.path();
                    let name = path.file_name()?.to_string_lossy().into_owned();
                    Some((name, std::fs::read(&path).ok()?))
                })
                .collect();
            match Character::from_files(&id, files.iter().map(|(n, b)| (n.as_str(), b.clone()))) {
                Ok(c) => self.insert(c),
                Err(e) => self.errors.push(format!("{id}: {e}")),
            }
        }
    }

    fn insert(&mut self, c: Character) {
        match self.list.iter_mut().find(|x| x.id == c.id) {
            Some(slot) => *slot = c,
            None => self.list.push(c),
        }
    }

    /// The character with this id, or the default one.
    pub fn get(&self, id: &str) -> Option<&Character> {
        self.list
            .iter()
            .find(|c| c.id == id)
            .or_else(|| self.list.iter().find(|c| c.id == DEFAULT_ID))
            .or(self.list.first())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Character> {
        self.list.iter()
    }
}

/// Keeps the recolored SVG for what's on screen, so we only rebuild it (and
/// re-rasterize) when the character, pose or colors change.
#[derive(Default)]
pub struct CharacterImage {
    key: Option<(String, Pose, Color32, Color32)>,
    uri: String,
    bytes: Arc<[u8]>,
    generation: u64,
}

impl CharacterImage {
    pub fn image(
        &mut self,
        ctx: &egui::Context,
        character: &Character,
        pose: Pose,
        accent: Color32,
        ink: Color32,
    ) -> egui::Image<'static> {
        let key = (character.id.clone(), pose, accent, ink);
        if self.key.as_ref() != Some(&key) {
            if !self.uri.is_empty() {
                ctx.forget_image(&self.uri);
            }
            self.generation += 1;
            self.uri = format!(
                "bytes://character/{}/{pose:?}/{}.svg",
                character.id, self.generation
            );
            self.bytes = character.svg(pose, accent, ink).into_bytes().into();
            self.key = Some(key);
        }
        egui::Image::from_bytes(
            self.uri.clone(),
            egui::load::Bytes::Shared(self.bytes.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_characters_have_every_pose_and_render() {
        let chars = Characters::load();
        assert!(chars.errors.is_empty(), "{:?}", chars.errors);
        let axolotl = chars.get(DEFAULT_ID).expect("default character");
        assert_eq!(axolotl.name, "Axolotl");
        assert_eq!(
            axolotl.names.get("pt-BR").map(String::as_str),
            Some("Axolote")
        );
        for (id, files) in builtin::BUILTIN {
            for pose in Pose::ALL {
                assert!(
                    files.iter().any(|(name, _)| *name == pose.file_name()),
                    "{id} is missing {}",
                    pose.file_name()
                );
            }
        }
        for character in chars
            .iter()
            .filter(|c| builtin::BUILTIN.iter().any(|(id, _)| *id == c.id))
        {
            for pose in Pose::ALL {
                let svg = character.svg(pose, Color32::RED, Color32::GRAY);
                assert!(!svg.contains(ACCENT_SENTINEL) && !svg.contains(INK_SENTINEL));
                let size = egui::SizeHint::Size {
                    width: 200,
                    height: 160,
                    maintain_aspect_ratio: true,
                };
                egui_extras::image::load_svg_bytes_with_size(
                    svg.as_bytes(),
                    size,
                    &Default::default(),
                )
                .unwrap_or_else(|e| panic!("{} {pose:?}: {e}", character.id));
            }
        }
    }

    #[test]
    fn missing_poses_fall_back_to_idle() {
        let c = Character::from_files("x", [("idle.svg", b"<svg/>".to_vec())]).unwrap();
        assert_eq!(
            c.svg(Pose::LongBreak, Color32::RED, Color32::GRAY),
            "<svg/>"
        );
        assert_eq!(c.name, "x");
        assert!(Character::from_files("y", [("focus.svg", b"<svg/>".to_vec())]).is_err());
    }
}
