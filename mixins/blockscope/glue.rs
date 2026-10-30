// Blockscope's game half: the crosshair's block or creature, the settings,
// and the box drawn on the HUD. `runtime.rs` holds the rest.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use horizon_client::containers::BlockEntity;
use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_client::lang;

use crate::game::Game;
use crate::game::native_blockscope as m;

/// A container's slots are the server's: the window asks for the block under
/// the crosshair here, and the server thread's tick answers with its slots.
/// The client's own copy of a chest stays empty until its menu opens.
// shortcut: one ask at a time, so two windows (a host and a guest) overwrite
// each other's answer and a grid can flicker; key them per window when a LAN
// host plays with company and it shows.
static WANTED: Mutex<Option<(String, [i32; 3])>> = Mutex::new(None);
static ANSWER: Mutex<Option<Grid>> = Mutex::new(None);

/// A container the server answered for: its block, the columns its own screen
/// shows and its slots.
#[derive(Clone)]
struct Grid {
    pos: [i32; 3],
    block: String,
    columns: usize,
    slots: Vec<Option<ItemStack>>,
}

fn lock<T>(cell: &Mutex<T>) -> MutexGuard<'_, T> {
    cell.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Default)]
struct State {
    /// Read once, then only what a click writes.
    settings: Option<m::Settings>,
    /// Item id to its name in the player's language.
    names: HashMap<String, String>,
    /// The item table was read (a block with no item does not read it again).
    named: bool,
    /// The last container the server sent for the block under the crosshair.
    grid: Option<Grid>,
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

/// A container's slots and the columns its own screen shows, out of a block
/// entity: a chest's grid, a furnace's or a brewing stand's three columns.
fn grid_of(entity: &BlockEntity) -> Option<(Vec<Option<ItemStack>>, usize)> {
    match entity {
        BlockEntity::Chest(slots) => {
            let columns = m::grid(slots.len())?.0;
            Some((slots.clone(), columns))
        }
        BlockEntity::Furnace(furnace) => Some((furnace.items.to_vec(), 3)),
        BlockEntity::Brewing(stand) => Some((stand.items.to_vec(), 3)),
        _ => None,
    }
}

/// The server thread's tick: the slots of the container the window asked
/// about. A guest's or a dedicated server's game has no window asking.
pub(super) fn tick(game: &mut Game<'_>) {
    if game.native_window() {
        return;
    }
    let Some((block, pos)) = lock(&WANTED).clone() else {
        return;
    };
    let entity = game.world.block_entities.get(&(pos[0], pos[1], pos[2]));
    let Some((slots, columns)) = entity.and_then(grid_of) else {
        return;
    };
    *lock(&ANSWER) = Some(Grid {
        pos,
        block,
        columns,
        slots,
    });
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
    block_entity: bool,
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
        block_entity: facts.block_entity,
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

/// The block's own name ("Wheat Crops" where its item is the seeds), else the
/// item's, else a readable id.
fn block_name(block: &str, item: Option<String>) -> String {
    let fallback = item.unwrap_or_else(|| m::pretty(block));
    let key = format!("block.{}", block.replace(':', "."));
    lang::get_or(&key, &fallback)
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
    let key = format!("entity.{}", mob.kind.id.replace(':', "."));
    Some((
        lang::get_or(&key, &m::pretty(&mob.kind.id)),
        mob.health,
        mob.kind.max_health,
    ))
}

/// The box: its lines, its icon and a container's slots, wherever the
/// settings put it.
fn draw(
    game: &Game<'_>,
    batch: &mut HudBatch,
    settings: &m::Settings,
    rows: &[(String, u32)],
    icon: Option<&ItemStack>,
    slots: Option<(&[Option<ItemStack>], usize)>,
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
        game.native_fill(batch, [cx, cy, m::CELL, m::CELL], 0x6600_0000);
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
    // A container's slots: ask the server thread for this block, and keep its
    // answer until the crosshair moves to another block.
    if settings.containers && pick.block_entity {
        *lock(&WANTED) = Some((pick.block.clone(), pick.pos));
        let stale = with(|s| {
            s.grid
                .as_ref()
                .is_none_or(|g| !m::same_block(g.pos, &g.block, pick.pos, &pick.block))
        });
        if stale {
            // The clone happens only for an answer about this very block: a
            // sign or a campfire never gets one, and must not copy the chest's
            // slots over again every frame.
            let matched = lock(&ANSWER)
                .as_ref()
                .filter(|g| m::same_block(g.pos, &g.block, pick.pos, &pick.block))
                .cloned();
            if let Some(grid) = matched {
                with(|s| s.grid = Some(grid));
            }
        }
    } else {
        // Nothing to show for a block that holds nothing: drop what a chest
        // left here, or its slots would hang over stone.
        with(|s| s.grid = None);
    }
    let named = name_of(game, &pick.item);
    let name = block_name(&pick.block, named);
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
    // shortcut: draw runs inside the State borrow so a container's slots are
    // not cloned every frame; a with() call inside draw would panic on it.
    with(|s| {
        let slots = s.grid.as_ref().filter(|g| {
            m::same_block(g.pos, &g.block, pick.pos, &pick.block)
                && g.slots.iter().any(Option::is_some)
        });
        draw(
            game,
            batch,
            &settings,
            &rows,
            stack.as_ref(),
            slots.map(|g| (g.slots.as_slice(), g.columns)),
        );
    });
}
