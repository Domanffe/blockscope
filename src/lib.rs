//! Blockscope: what the crosshair is on. `game.raycast` names the block,
//! `recipes.items` turns its id into the name the player's language uses,
//! and `render.set-hud` draws a Jade-style box beside the crosshair. The
//! game draws the box under screens and hides it with F1, like any HUD.
//! Its page in Options > Mods sets the reach, where the box sits, the
//! background's opacity, coordinates and the item icon; the choice is kept
//! in `storage`.

use std::collections::HashMap;

use horizon_mod_sdk::{
    Mod, Registration, game, recipes,
    render::{self, FillShape, ItemShape, Point, Rect, Shape, TextShape},
    storage,
    ui::{self, ButtonWidget, Widget},
};

/// How far the crosshair reaches, in blocks. `game.raycast` takes any of
/// them up to 1024, across the distant terrain.
const REACHES: [u32; 4] = [5, 20, 64, 1024];
const PLACES: [&str; 4] = [
    "Beside the crosshair",
    "Top left",
    "Top centre",
    "Top right",
];
const OPACITIES: [u8; 4] = [100, 75, 50, 25];

const REACH_BUTTON: u32 = 1;
const PLACE_BUTTON: u32 = 2;
const BACKGROUND_BUTTON: u32 = 3;
const COORDS_BUTTON: u32 = 4;
const ICON_BUTTON: u32 = 5;

/// White name, grey id, distance and coordinates.
const INK: u32 = 0xFF_FFFF;
const INK_DIM: u32 = 0xA0_A0A0;
const PAD: f32 = 3.0;
const LINE: f32 = 10.0;
const ICON: f32 = 16.0;
const ICON_GAP: f32 = 2.0;
/// Clear of the crosshair, and of the screen's edge.
const GAP: f32 = 10.0;
const EDGE: f32 = 2.0;

struct Blockscope {
    /// Item id to its name in the player's language, read once in a world.
    names: HashMap<String, String>,
    reach: usize,
    place: usize,
    opacity: usize,
    coords: bool,
    icon: bool,
    /// The box last sent, so an unchanged one costs no call.
    shown: Option<String>,
}

impl Blockscope {
    /// The item table carries every block's name, in the player's language.
    /// It is read on the first tick in a world: there is no registry on the
    /// title screen.
    fn load_names(&mut self) {
        if self.names.is_empty() && game::get_player().is_some() {
            self.names = recipes::items()
                .into_iter()
                .map(|item| (item.id, item.name))
                .collect();
        }
    }

    fn hide(&mut self) {
        if self.shown.take().is_some() {
            render::set_hud(&[]);
        }
    }

    fn draw(&mut self) {
        let Some(hit) = game::raycast(REACHES[self.reach] as f32) else {
            self.hide();
            return;
        };
        let Some(player) = game::get_player() else {
            self.hide();
            return;
        };
        let named = self.names.get(&hit.block);
        let name = named.cloned().unwrap_or_else(|| pretty(&hit.block));
        let (dx, dy, dz) = (hit.x - player.x, hit.y - player.y, hit.z - player.z);
        let detail = format!(
            "{}  {:.1} m",
            hit.block,
            (dx * dx + dy * dy + dz * dz).sqrt()
        );
        let mut rows = vec![(name, INK), (detail, INK_DIM)];
        if self.coords {
            rows.push((
                format!("{} {} {}", hit.block_x, hit.block_y, hit.block_z),
                INK_DIM,
            ));
        }
        // An icon only for a block the item table names: the game draws
        // nothing for an id it has no item for, which would leave a gap.
        let icon = (self.icon && named.is_some()).then(|| hit.block.clone());
        let key = format!("{rows:?}{icon:?}{}{}", self.place, self.opacity);
        if self.shown.as_ref() == Some(&key) {
            return;
        }
        let offset = if icon.is_some() { ICON + ICON_GAP } else { 0.0 };
        let text_w = rows
            .iter()
            .map(|(text, _)| render::text_width(text))
            .fold(0.0, f32::max);
        let w = PAD * 2.0 + offset + text_w;
        let lines_h = (rows.len() as f32 - 1.0) * LINE + 9.0;
        let h = PAD * 2.0 + lines_h.max(if icon.is_some() { ICON } else { 0.0 });
        let (gui_w, gui_h) = render::gui_size();
        let (x, y) = place(w, h, gui_w, gui_h, self.place);
        let mut shapes = vec![Shape::Fill(FillShape {
            rect: Rect { x, y, w, h },
            color: alpha(OPACITIES[self.opacity]),
        })];
        if let Some(item) = icon {
            shapes.push(Shape::Item(ItemShape {
                item,
                at: Point {
                    x: x + PAD,
                    y: y + PAD,
                },
                scale: 1.0,
            }));
        }
        for (i, (text, color)) in rows.into_iter().enumerate() {
            shapes.push(Shape::Text(TextShape {
                text,
                at: Point {
                    x: x + PAD + offset,
                    y: y + PAD + i as f32 * LINE,
                },
                color,
                centered: false,
                shadow: true,
            }));
        }
        render::set_hud(&shapes);
        self.shown = Some(key);
    }

    fn page(&self) {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let button = |id, label: String| Widget::Button(ButtonWidget { id, label });
        ui::set_settings(&[
            button(
                REACH_BUTTON,
                format!("Reach: {} blocks", REACHES[self.reach]),
            ),
            button(PLACE_BUTTON, format!("Position: {}", PLACES[self.place])),
            button(
                BACKGROUND_BUTTON,
                format!("Background: {}%", OPACITIES[self.opacity]),
            ),
            button(COORDS_BUTTON, format!("Coordinates: {}", on(self.coords))),
            button(ICON_BUTTON, format!("Item icon: {}", on(self.icon))),
        ]);
    }

    fn save(&self) {
        let flag = |b: bool| if b { &b"true"[..] } else { &b"false"[..] };
        let _ = storage::set("reach", REACHES[self.reach].to_string().as_bytes());
        let _ = storage::set("place", PLACES[self.place].as_bytes());
        let _ = storage::set("background", OPACITIES[self.opacity].to_string().as_bytes());
        let _ = storage::set("coords", flag(self.coords));
        let _ = storage::set("icon", flag(self.icon));
    }
}

/// Where the box's top left goes: in a corner, centred at the top, or beside
/// the crosshair and flipped left when the screen's right edge is closer.
fn place(w: f32, h: f32, gui_w: f32, gui_h: f32, at: usize) -> (f32, f32) {
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
fn alpha(percent: u8) -> u32 {
    (u32::from(percent) * 255 / 100) << 24
}

/// `minecraft:grass_block` as "Grass Block", for the blocks with no item.
fn pretty(id: &str) -> String {
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

fn stored(key: &str) -> Option<String> {
    storage::get(key)
        .and_then(|v| String::from_utf8(v).ok())
        .map(|v| v.trim().to_owned())
}

/// The stored row's index, or `default` for a value not in `values`.
fn picked<T: PartialEq + ToString>(key: &str, values: &[T], default: usize) -> usize {
    stored(key)
        .and_then(|v| values.iter().position(|value| value.to_string() == v))
        .unwrap_or(default)
}

/// A stored flag; `default` when the key was never set.
fn flag(key: &str, default: bool) -> bool {
    stored(key).map_or(default, |v| v != "false")
}

impl Mod for Blockscope {
    fn init() -> Result<(Self, Registration), String> {
        let blockscope = Blockscope {
            names: HashMap::new(),
            reach: picked("reach", &REACHES, 1),
            place: picked("place", &PLACES, 0),
            opacity: picked("background", &OPACITIES, 2),
            coords: flag("coords", false),
            icon: flag("icon", true),
            shown: None,
        };
        blockscope.page();
        Ok((blockscope, Registration::new().on_tick()))
    }

    fn tick(&mut self, _tick: u64) {
        self.load_names();
        self.draw();
    }

    fn ui_select(&mut self, id: u32, _row: u32) {
        match id {
            REACH_BUTTON => self.reach = (self.reach + 1) % REACHES.len(),
            PLACE_BUTTON => self.place = (self.place + 1) % PLACES.len(),
            BACKGROUND_BUTTON => self.opacity = (self.opacity + 1) % OPACITIES.len(),
            COORDS_BUTTON => self.coords = !self.coords,
            ICON_BUTTON => self.icon = !self.icon,
            _ => return,
        }
        self.page();
        self.save();
        self.shown = None;
    }

    fn ui_settings_opened(&mut self) {
        self.page();
    }
}

horizon_mod_sdk::export_mod!(Blockscope);

#[cfg(test)]
mod tests {
    use super::{EDGE, GAP, alpha, place, pretty};

    #[test]
    fn a_block_with_no_item_gets_a_readable_name() {
        assert_eq!(pretty("minecraft:grass_block"), "Grass Block");
        assert_eq!(pretty("minecraft:water"), "Water");
        assert_eq!(pretty("some_mod:ancient_altar"), "Ancient Altar");
    }

    #[test]
    fn the_box_sits_clear_of_the_crosshair() {
        // 16 × 25 on a 1920 × 1080 GUI: right of the crosshair, centred.
        assert_eq!(place(16.0, 25.0, 1920.0, 1080.0, 0), (970.0, 527.5));
    }

    #[test]
    fn the_box_flips_left_at_the_screen_edge() {
        // 950 px wide: 960 + 10 + 950 passes 1918, so it goes left.
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
        assert_eq!(place(2000.0, 25.0, 1920.0, 1080.0, 3).0, EDGE);
        assert_eq!(place(2000.0, 25.0, 1920.0, 1080.0, 2).0, EDGE);
    }

    #[test]
    fn the_background_percentage_is_the_alpha() {
        assert_eq!(alpha(100), 0xFF00_0000);
        assert_eq!(alpha(50), 0x7F00_0000);
        assert_eq!(alpha(0), 0);
    }
}
