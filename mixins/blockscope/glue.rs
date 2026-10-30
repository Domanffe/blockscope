// Blockscope's game half: the crosshair's block or creature, the settings,
// and the box drawn on the HUD. `runtime.rs` holds the rest.

use std::cell::RefCell;
use std::collections::HashMap;

use horizon_client::containers::BlockEntity;
use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_client::lang;

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

/// What the crosshair is on, read once: the game's own raycast at `reach`.
struct Pick {
    block: String,
    item: String,
    point: [f64; 3],
    pos: [i32; 3],
    state: String,
    hardness: f32,
    requires_tool: bool,
}

fn hit(game: &Game<'_>, reach: f64) -> Option<Pick> {
    let (origin, dir) = game.aim()?;
    let hit = game.clip_ray(origin, dir, reach)?;
    let pos = hit.pos;
    let state = game.world.block(pos[0], pos[1], pos[2]);
    let facts = game.res.facts().get(state);
    Some(Pick {
        block: facts.name.to_string(),
        item: facts.item.to_string(),
        point: hit.point,
        pos,
        state: game.state_string(state),
        hardness: facts.hardness,
        requires_tool: facts.requires_tool,
    })
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

/// The creature the crosshair is on, before any block: its name in the
/// player's language, its health and its type's.
fn creature(game: &Game<'_>) -> Option<(String, f32, f32)> {
    let Some(Some(id)) = game.pick_entity_hit() else {
        return None;
    };
    let mob = game.mobs.iter().find(|m| m.id as i32 == id)?;
    let path = mob.kind.id.replace(':', ".");
    let name = lang::get_or(&format!("entity.{path}"), &m::pretty(&mob.kind.id));
    Some((name, mob.health, mob.kind.max_health))
}

/// A container's slots, row by row, with the columns its own screen shows;
/// none for a block that holds nothing, or holds nothing yet.
fn container<'a>(game: &'a Game<'_>, pos: [i32; 3]) -> Option<(Vec<Option<&'a ItemStack>>, usize)> {
    let entity = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))?;
    let (slots, columns) = match entity {
        BlockEntity::Chest(slots) => {
            let columns = m::grid(slots.len())?.0;
            (
                slots.iter().map(Option::as_ref).collect::<Vec<_>>(),
                columns,
            )
        }
        // A furnace's own screen: input, fuel, result.
        BlockEntity::Furnace(furnace) => (furnace.items.iter().map(Option::as_ref).collect(), 3),
        BlockEntity::Brewing(stand) => (stand.items.iter().map(Option::as_ref).collect(), 3),
        _ => return None,
    };
    slots
        .iter()
        .any(Option::is_some)
        .then_some((slots, columns))
}

/// The box: its lines, its icon and a container's slots, wherever the
/// settings put it.
fn draw(
    game: &Game<'_>,
    batch: &mut HudBatch,
    settings: &m::Settings,
    rows: &[(String, u32)],
    icon: Option<&ItemStack>,
    slots: Option<(&[Option<&ItemStack>], usize)>,
) {
    let text_w = rows
        .iter()
        .map(|(text, _)| HudBatch::text_width(&game.gui.atlas, text))
        .fold(0.0f32, f32::max);
    let cells = slots.map(|(slots, columns)| (slots, columns, slots.len().div_ceil(columns)));
    let (w, h) = m::size(
        rows.len(),
        text_w,
        icon.is_some(),
        cells.map(|(_, columns, lines)| (columns, lines)),
    );
    let (x, y) = m::place(w, h, batch.width, batch.height, settings.place);
    game.native_fill(
        batch,
        [x, y, w, h],
        m::alpha(m::OPACITIES[settings.opacity]),
    );
    let offset = if icon.is_some() {
        m::ICON + m::ICON_GAP
    } else {
        0.0
    };
    if let Some(stack) = icon {
        game.draw_item(batch, stack, x + m::PAD, y + m::PAD);
    }
    for (i, (text, ink)) in rows.iter().enumerate() {
        game.native_text(
            batch,
            text,
            [x + m::PAD + offset, y + m::PAD + i as f32 * m::LINE],
            *ink,
            true,
        );
    }
    let Some((slots, columns, _)) = cells else {
        return;
    };
    let lines_h = if rows.is_empty() {
        0.0
    } else {
        (rows.len() - 1) as f32 * m::LINE + 9.0
    };
    let text_h = lines_h.max(if icon.is_some() { m::ICON } else { 0.0 });
    let top = y + m::PAD + text_h + m::GRID_GAP;
    for (i, slot) in slots.iter().enumerate() {
        let (column, line) = (i % columns, i / columns);
        let (cx, cy) = (
            x + m::PAD + column as f32 * m::CELL,
            top + line as f32 * m::CELL,
        );
        game.native_fill(batch, [cx, cy, m::CELL, m::CELL], 0x3300_0000);
        if let Some(stack) = slot {
            game.draw_item(batch, stack, cx + 1.0, cy + 1.0);
        }
    }
}

/// The HUD hook: the crosshair's block or creature in a box, while playing
/// with the mod on. The game draws it under screens and hides it with F1.
pub(super) fn hud(game: &mut Game<'_>, batch: &mut HudBatch, corners: &mut Corners) {
    let _ = corners;
    if !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let settings = settings(game);
    if settings.entities
        && let Some((name, health, max_health)) = creature(game)
    {
        let rows = vec![
            (name, m::INK),
            (format!("{health:.0} / {max_health:.0}"), m::INK_DIM),
        ];
        draw(game, batch, &settings, &rows, None, None);
        return;
    }
    let Some(player) = game.native_player() else {
        return;
    };
    let Some(pick) = hit(game, settings.reach()) else {
        return;
    };
    let named = name_of(game, &pick.item);
    let name = named.unwrap_or_else(|| m::pretty(&pick.block));
    let (dx, dy, dz) = (
        pick.point[0] - player.x,
        pick.point[1] - player.y,
        pick.point[2] - player.z,
    );
    let mut rows = vec![
        (name, m::INK),
        (
            format!(
                "{} {:.1} m",
                pick.block,
                (dx * dx + dy * dy + dz * dz).sqrt()
            ),
            m::INK_DIM,
        ),
    ];
    let properties = m::properties(&pick.state);
    if settings.coords {
        let [x, y, z] = pick.pos;
        rows.push((format!("{x} {y} {z}"), m::INK_DIM));
    }
    if settings.harvest {
        let correct = game.res.items().correct_for_drops(game.held(), &pick.block);
        rows.push((
            m::harvest(pick.hardness, pick.requires_tool, correct),
            m::INK_DIM,
        ));
    }
    if settings.states
        && let Some(properties) = properties
    {
        rows.push((format!("State: {properties}"), m::INK_DIM));
    }
    if settings.growth
        && let Some((age, ripens)) = properties.and_then(|p| m::growth(&pick.block, p))
    {
        let percent = age * 100 / ripens.max(1);
        rows.push((format!("Growth {age}/{ripens} ({percent}%)"), m::INK_DIM));
    }
    let stack = settings.icon.then(|| icon(game, &pick.item)).flatten();
    let slots = settings
        .containers
        .then(|| container(game, pick.pos))
        .flatten();
    draw(
        game,
        batch,
        &settings,
        &rows,
        stack.as_ref(),
        slots
            .as_ref()
            .map(|(slots, columns)| (slots.as_slice(), *columns)),
    );
}
