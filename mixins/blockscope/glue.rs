// Blockscope's game half: the crosshair's block, its item's name and icon,
// the settings, and the box drawn on the HUD. `runtime.rs` holds the rest.

use std::cell::RefCell;
use std::collections::HashMap;

use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;

use crate::game::Game;
use crate::game::native_blockscope as m;

#[derive(Default)]
struct State {
    /// Read once, then only what a click writes.
    settings: Option<m::Settings>,
    /// Item id to its name in the player's language.
    names: HashMap<String, String>,
    /// The item table was read (a block with no item does not read it again).
    named: bool,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|cell| f(cell.borrow_mut().get_or_insert_with(State::default)))
}

/// The settings the player kept, read from the WebAssembly mod's files the
/// first time.
pub(super) fn settings(game: &Game<'_>) -> m::Settings {
    with(|s| {
        *s.settings
            .get_or_insert_with(|| m::Settings::load(|key| game.native_setting(m::ID, key)))
    })
}

/// A click on the mod's settings page: the next value, kept.
pub(super) fn click(game: &Game<'_>, button: usize) {
    let mut settings = settings(game);
    settings.click(button);
    for (key, value) in settings.saved() {
        game.native_set_setting(m::ID, key, &value);
    }
    with(|s| s.settings = Some(settings));
}

/// What the crosshair is on: the block's id, its item's id, the point the
/// ray met it at and its position. The game's own raycast, at `reach`.
fn hit(game: &Game<'_>, reach: f64) -> Option<(String, String, [f64; 3], [i32; 3])> {
    let (origin, dir) = game.aim()?;
    let hit = game.clip_ray(origin, dir, reach)?;
    let state = game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
    let facts = game.res.facts().get(state);
    Some((
        facts.name.to_string(),
        facts.item.to_string(),
        hit.point,
        hit.pos,
    ))
}

/// The name the player's language gives `item`, from the game's item table.
fn name_of(game: &mut Game<'_>, item: &str) -> Option<String> {
    let known = with(|s| (s.named, s.names.get(item).cloned()));
    if let (true, name) = known {
        return name;
    }
    let tables = game.native_recipes()?;
    with(|s| {
        s.named = true;
        s.names
            .extend(tables.items.iter().map(|i| (i.id.clone(), i.name.clone())));
    });
    with(|s| s.names.get(item).cloned())
}

/// The block's item as a stack the game draws its icon from; none for a
/// block the item registry has no item for.
fn icon(game: &Game<'_>, item: &str) -> Option<ItemStack> {
    game.res.items().get(item)?;
    Some(ItemStack {
        item: item.into(),
        count: 1,
        damage: 0,
        extra: None,
    })
}

/// The HUD hook: the crosshair's block in a box, while playing with the mod
/// on. The game draws it under screens and hides it with F1.
pub(super) fn hud(game: &mut Game<'_>, batch: &mut HudBatch, corners: &mut Corners) {
    let _ = corners;
    if !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let settings = settings(game);
    let Some(player) = game.native_player() else {
        return;
    };
    let Some((block, item, point, pos)) = hit(game, settings.reach()) else {
        return;
    };
    let named = name_of(game, &item);
    let name = named.unwrap_or_else(|| m::pretty(&block));
    let (dx, dy, dz) = (
        point[0] - player.x,
        point[1] - player.y,
        point[2] - player.z,
    );
    let mut rows = vec![
        (name, m::INK),
        (
            format!("{} {:.1} m", block, (dx * dx + dy * dy + dz * dz).sqrt()),
            m::INK_DIM,
        ),
    ];
    if settings.coords {
        rows.push((format!("{} {} {}", pos[0], pos[1], pos[2]), m::INK_DIM));
    }
    let stack = settings.icon.then(|| icon(game, &item)).flatten();
    let text_w = rows
        .iter()
        .map(|(text, _)| HudBatch::text_width(&game.gui.atlas, text))
        .fold(0.0f32, f32::max);
    let (w, h) = m::size(rows.len(), text_w, stack.is_some());
    let (x, y) = m::place(w, h, batch.width, batch.height, settings.place);
    game.native_fill(
        batch,
        [x, y, w, h],
        m::alpha(m::OPACITIES[settings.opacity]),
    );
    let offset = if stack.is_some() {
        m::ICON + m::ICON_GAP
    } else {
        0.0
    };
    if let Some(stack) = &stack {
        game.draw_item(batch, stack, x + m::PAD, y + m::PAD);
    }
    for (i, (text, ink)) in rows.into_iter().enumerate() {
        game.native_text(
            batch,
            &text,
            [x + m::PAD + offset, y + m::PAD + i as f32 * m::LINE],
            ink,
            true,
        );
    }
}
