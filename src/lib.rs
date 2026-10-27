//! Block Info: what the crosshair is on. `game.raycast` names the block,
//! `recipes.items` turns its id into the name the player's language uses,
//! and `render.set-hud` draws a Jade-style box beside the crosshair. The
//! game draws the box under screens and hides it with F1, like any HUD.

use std::collections::HashMap;

use horizon_mod_sdk::{
    Mod, Registration, game, recipes,
    render::{self, FillShape, Point, Rect, Shape, TextShape},
};

/// How far the crosshair reaches, in blocks.
const REACH: f32 = 20.0;
/// The box: translucent black, white name, grey id and distance.
const BACKGROUND: u32 = 0x9000_0000;
const INK: u32 = 0xFF_FFFF;
const INK_DIM: u32 = 0xA0_A0A0;
const PAD: f32 = 3.0;
const LINE: f32 = 10.0;
/// Clear of the crosshair, and of the screen's edge.
const GAP: f32 = 10.0;
const EDGE: f32 = 2.0;

struct BlockInfo {
    /// Item id to its name in the player's language, read once in a world.
    names: HashMap<String, String>,
    /// The two lines drawn now; `None` when the box is hidden.
    shown: Option<(String, String)>,
}

impl BlockInfo {
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
        let Some(hit) = game::raycast(REACH) else {
            self.hide();
            return;
        };
        let Some(player) = game::get_player() else {
            self.hide();
            return;
        };
        let name = self
            .names
            .get(&hit.block)
            .cloned()
            .unwrap_or_else(|| pretty(&hit.block));
        let (dx, dy, dz) = (hit.x - player.x, hit.y - player.y, hit.z - player.z);
        let detail = format!(
            "{}  {:.1} m",
            hit.block,
            (dx * dx + dy * dy + dz * dz).sqrt()
        );
        if self.shown.as_ref() == Some(&(name.clone(), detail.clone())) {
            return;
        }
        let w = render::text_width(&name).max(render::text_width(&detail)) + PAD * 2.0;
        let h = PAD * 2.0 + LINE + 9.0;
        let (gui_w, gui_h) = render::gui_size();
        let (cx, cy) = (gui_w / 2.0, gui_h / 2.0);
        let mut x = cx + GAP;
        if x + w > gui_w - EDGE {
            x = cx - GAP - w;
        }
        let y = (cy - h / 2.0).clamp(EDGE, (gui_h - h - EDGE).max(EDGE));
        render::set_hud(&[
            Shape::Fill(FillShape {
                rect: Rect { x, y, w, h },
                color: BACKGROUND,
            }),
            Shape::Text(TextShape {
                text: name.clone(),
                at: Point { x: x + PAD, y: y + PAD },
                color: INK,
                centered: false,
                shadow: true,
            }),
            Shape::Text(TextShape {
                text: detail.clone(),
                at: Point {
                    x: x + PAD,
                    y: y + PAD + LINE,
                },
                color: INK_DIM,
                centered: false,
                shadow: true,
            }),
        ]);
        self.shown = Some((name, detail));
    }
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

impl Mod for BlockInfo {
    fn init() -> Result<(Self, Registration), String> {
        Ok((
            BlockInfo {
                names: HashMap::new(),
                shown: None,
            },
            Registration::new().on_tick(),
        ))
    }

    fn tick(&mut self, _tick: u64) {
        self.load_names();
        self.draw();
    }
}

horizon_mod_sdk::export_mod!(BlockInfo);

#[cfg(test)]
mod tests {
    use super::pretty;

    #[test]
    fn a_block_with_no_item_gets_a_readable_name() {
        assert_eq!(pretty("minecraft:grass_block"), "Grass Block");
        assert_eq!(pretty("minecraft:water"), "Water");
        assert_eq!(pretty("some_mod:ancient_altar"), "Ancient Altar");
    }
}
