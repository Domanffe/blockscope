// Blockscope's own half: its settings, the rows of its box and where the
// box goes. No game types here; `glue.rs` converts the game's and draws.
// This file is also the bundle's Cargo library (`native/lib.rs`), so its
// tests run without a Horizon build.

/// The mod's id (its settings folder, as the WebAssembly mod's), its name
/// and version.
pub const ID: &str = "blockscope";
pub const NAME: &str = "Blockscope";
pub const VERSION: &str = "0.3.0";
pub const DESCRIPTION: &str = "The block under your crosshair: its name, id, distance and its icon, with reach, placement and opacity on its own settings page. A Jade-style tooltip for Horizon.";

/// How far the crosshair reaches, in blocks.
pub const REACHES: [u32; 4] = [5, 20, 64, 1024];
/// Where the box sits.
pub const PLACES: [&str; 4] = [
    "Beside the crosshair",
    "Top left",
    "Top centre",
    "Top right",
];
/// The background's opacity, in percent.
pub const OPACITIES: [u8; 4] = [100, 75, 50, 25];

/// White name, grey id, distance and coordinates.
pub const INK: u32 = 0xFF_FFFF;
pub const INK_DIM: u32 = 0xA0_A0A0;
pub const PAD: f32 = 3.0;
pub const LINE: f32 = 10.0;
pub const ICON: f32 = 16.0;
pub const ICON_GAP: f32 = 2.0;
/// Clear of the crosshair, and of the screen's edge.
pub const GAP: f32 = 10.0;
pub const EDGE: f32 = 2.0;

/// The settings' keys, as the WebAssembly mod wrote them.
const KEYS: [&str; 5] = ["reach", "place", "background", "coords", "icon"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub reach: usize,
    pub place: usize,
    pub opacity: usize,
    pub coords: bool,
    pub icon: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            reach: 1,
            place: 0,
            opacity: 2,
            coords: false,
            icon: true,
        }
    }
}

impl Settings {
    /// The settings `read` answers its keys for: a value or a label
    /// (`"20"`, `"Top left"`), and `true`/`false` for the flags. The
    /// WebAssembly mod's files are read as they are.
    pub fn load(read: impl Fn(&str) -> Option<String>) -> Self {
        let mut settings = Self::default();
        let value = |key: &str| read(key).map(|v| v.trim().to_owned());
        let reaches = REACHES.map(|r| r.to_string());
        if let Some(i) = value(KEYS[0]).and_then(|v| pick(&v, &reaches)) {
            settings.reach = i;
        }
        if let Some(i) = value(KEYS[1]).and_then(|v| pick(&v, &PLACES.map(String::from))) {
            settings.place = i;
        }
        let opacities = OPACITIES.map(|o| o.to_string());
        if let Some(i) = value(KEYS[2]).and_then(|v| pick(&v, &opacities)) {
            settings.opacity = i;
        }
        if let Some(v) = value(KEYS[3]) {
            settings.coords = v != "false";
        }
        if let Some(v) = value(KEYS[4]) {
            settings.icon = v != "false";
        }
        settings
    }
    /// The five keys with the values to keep, as the WebAssembly mod kept
    /// them, so a player's choice carries over.
    pub fn saved(&self) -> [(&'static str, String); 5] {
        [
            (KEYS[0], REACHES[self.reach].to_string()),
            (KEYS[1], PLACES[self.place].to_owned()),
            (KEYS[2], OPACITIES[self.opacity].to_string()),
            (KEYS[3], self.coords.to_string()),
            (KEYS[4], self.icon.to_string()),
        ]
    }
    /// Its page's buttons, one click per value.
    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("Reach: {} blocks", REACHES[self.reach]),
            format!("Position: {}", PLACES[self.place]),
            format!("Background: {}%", OPACITIES[self.opacity]),
            format!("Coordinates: {}", on(self.coords)),
            format!("Item icon: {}", on(self.icon)),
        ]
    }
    /// Button `button` of its page went: the next value.
    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.reach = (self.reach + 1) % REACHES.len(),
            1 => self.place = (self.place + 1) % PLACES.len(),
            2 => self.opacity = (self.opacity + 1) % OPACITIES.len(),
            3 => self.coords = !self.coords,
            4 => self.icon = !self.icon,
            _ => {}
        }
    }
    /// How far the crosshair reaches now, in blocks.
    pub fn reach(&self) -> f64 {
        f64::from(REACHES[self.reach])
    }
}

fn on(flag: bool) -> &'static str {
    if flag { "ON" } else { "OFF" }
}

/// The index `value` names in `labels`: one of them, or a number below the
/// list's length.
fn pick(value: &str, labels: &[String]) -> Option<usize> {
    if let Some(i) = labels.iter().position(|l| l == value) {
        return Some(i);
    }
    value.parse::<usize>().ok().filter(|i| *i < labels.len())
}

/// The box's size for `rows` lines of `text_w` and an icon or none.
pub fn size(rows: usize, text_w: f32, icon: bool) -> (f32, f32) {
    let offset = if icon { ICON + ICON_GAP } else { 0.0 };
    let lines_h = if rows == 0 {
        0.0
    } else {
        (rows - 1) as f32 * LINE + 9.0
    };
    let h = PAD * 2.0 + lines_h.max(if icon { ICON } else { 0.0 });
    (PAD * 2.0 + offset + text_w, h)
}

/// Where the box's top left goes: in a corner, centred at the top, or beside
/// the crosshair and flipped left when the screen's right edge is closer.
pub fn place(w: f32, h: f32, gui_w: f32, gui_h: f32, at: usize) -> (f32, f32) {
    match at {
        1 => (EDGE, EDGE),
        2 => (((gui_w - w) / 2.0).max(EDGE), EDGE),
        3 => ((gui_w - w - EDGE).max(EDGE), EDGE),
        _ => {
            let (cx, cy) = (gui_w / 2.0, gui_h / 2.0);
            let mut x = cx + GAP;
            if x + w > gui_w - EDGE {
                x = cx - GAP - w;
            }
            let y = (cy - h / 2.0).clamp(EDGE, (gui_h - h - EDGE).max(EDGE));
            (x, y)
        }
    }
}

/// `50` as `0x7F000000`: the background's opacity is the fill's alpha.
pub fn alpha(percent: u8) -> u32 {
    (u32::from(percent) * 255 / 100) << 24
}

/// `minecraft:grass_block` as "Grass Block", for the blocks with no item.
pub fn pretty(id: &str) -> String {
    let path = id.split(':').nth(1).unwrap_or(id);
    let mut out = String::with_capacity(path.len());
    for (i, word) in path.split('_').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_with_no_item_gets_a_readable_name() {
        assert_eq!(pretty("minecraft:grass_block"), "Grass Block");
        assert_eq!(pretty("minecraft:water"), "Water");
        assert_eq!(pretty("some_mod:ancient_altar"), "Ancient Altar");
    }

    #[test]
    fn the_box_sits_clear_of_the_crosshair() {
        assert_eq!(place(16.0, 25.0, 1920.0, 1080.0, 0), (970.0, 527.5));
    }

    #[test]
    fn the_box_flips_left_at_the_screen_edge() {
        assert_eq!(place(950.0, 25.0, 1920.0, 1080.0, 0).0, 960.0 - GAP - 950.0);
    }

    #[test]
    fn a_corner_place_pins_the_box_there() {
        assert_eq!(place(100.0, 25.0, 1920.0, 1080.0, 1), (EDGE, EDGE));
        assert_eq!(
            place(100.0, 25.0, 1920.0, 1080.0, 2),
            ((1920.0 - 100.0) / 2.0, EDGE)
        );
        assert_eq!(
            place(100.0, 25.0, 1920.0, 1080.0, 3),
            (1920.0 - 100.0 - EDGE, EDGE)
        );
        // A box wider than the screen stays at the left edge, never negative.
        assert_eq!(place(2000.0, 25.0, 1920.0, 1080.0, 2).0, EDGE);
        assert_eq!(place(2000.0, 25.0, 1920.0, 1080.0, 3).0, EDGE);
    }

    #[test]
    fn the_background_percentage_is_the_alpha() {
        assert_eq!(alpha(100), 0xFF00_0000);
        assert_eq!(alpha(50), 0x7F00_0000);
        assert_eq!(alpha(0), 0);
    }

    #[test]
    fn a_click_walks_the_values_and_wraps() {
        let mut s = Settings::default();
        s.click(0);
        assert_eq!(REACHES[s.reach], 64);
        s.click(0);
        s.click(0);
        assert_eq!(REACHES[s.reach], 5);
        s.click(3);
        assert!(s.coords);
        s.click(3);
        assert!(!s.coords);
        let before = s;
        s.click(9);
        assert_eq!(s, before);
    }

    #[test]
    fn the_webassembly_mods_files_are_read_back() {
        let kept = [
            ("reach", "20"),
            ("place", "Top right"),
            ("background", "25"),
            ("coords", "true"),
            ("icon", "false"),
        ];
        let s = Settings::load(|key| {
            kept.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        });
        assert_eq!(
            s,
            Settings {
                reach: 1,
                place: 3,
                opacity: 3,
                coords: true,
                icon: false,
            }
        );
        // Nothing kept: the defaults, and what it writes reads back the same.
        let fresh = Settings::load(|_| None);
        assert_eq!(fresh, Settings::default());
        let again = Settings::load(|key| {
            fresh
                .saved()
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        });
        assert_eq!(again, fresh);
    }

    #[test]
    fn the_pages_labels_say_what_is_kept() {
        let labels = Settings::default().labels();
        assert_eq!(labels[0], "Reach: 20 blocks");
        assert_eq!(labels[2], "Background: 50%");
        assert_eq!(labels[3], "Coordinates: OFF");
        assert_eq!(labels[4], "Item icon: ON");
    }

    #[test]
    fn a_line_of_text_sets_the_box_height() {
        let (w, h) = size(2, 40.0, false);
        assert_eq!((w, h), (46.0, 25.0));
        let (w, h) = size(1, 40.0, true);
        assert_eq!((w, h), (64.0, 22.0));
    }
}
