# Blockscope

A Jade-style block tooltip for [Horizon](https://github.com/ludovic111/horizon): the block under your
crosshair, its name in your language, its id and how far it is, in a small box that you can put where
you want.

![Blockscope in the top right corner](docs/screenshot.png)

## What it shows

- The block's name as your game language names it, from the block's own name and then the item's;
  blocks with no item get their id with the words capitalised ("Water").
- The block id, so a modded block is identifiable.
- The distance from your feet to the point the crosshair meets the block.
- The block's icon beside its name, on its own row, drawn by the game's own item renderer.
- The tool the block wants beside its hardness, as that tool's own icon (a grey pickaxe for stone);
  the heart of the HUD beside a creature's health.
- Its hardness, and whether what you hold is the tool it wants ("Hardness 3.0 · needs the right
  tool", "this tool harvests it", "Unbreakable").
- The block's state properties, as the game names them (`facing=north,type=single`).
- A crop's growth out of the age it ripens at ("Growth 5/7 (71%)").
- A container's slots, laid out as the container's own screen has them.
- Its coordinates, on request.
- A creature instead of a block: its name and its health ("Корова", "10 / 10").

Nothing is drawn when you look at no block within reach, while a screen is open, or with the HUD
hidden (F1).

## Settings

**Options > Mods > Blockscope > Settings**, one click per value:

- **Reach**: 5, 20, 64 or 1024 blocks. The far settings go across the distant terrain, as the game's
  own raycast does.
- **Position**: beside the crosshair (where it flips left near the screen's edge), top left, top
  centre or top right.
- **Background**: 100, 75, 50 or 25% opaque.
- **Coordinates**: on or off.
- **Item icon**: on or off.
- **Hardness and tool**, **Block states**, **Crop growth**, **Container contents**, **Creatures**:
  each line on or off.

The choice is kept between games, in `horizon-mods/.horizon-mod-data/blockscope/`.

## Native mod (0.6.0)

Horizon's WebAssembly mod API is gone: `blockscope.hmod` (0.2.0) does not run any more. 0.6.0 is a
native mod made with [horizon-mixins](https://github.com/horizon-rust/horizon-mixins), the bundle
under `mixins/blockscope/` that hooks the game's own code and is compiled into a build of the game:

- `horizon-mixin.toml`: the two modules and the five hook points (`native_hud`, `native_tick`,
  `native_mods`, `native_settings`, `native_settings_click`).
- `runtime.rs`: the settings, the rows, where the box goes, what a crop's age means and how a
  container's slots lie out, with no game types in them, so its tests run without a Horizon build.
- `glue.rs`: the crosshair hit, the item table, the container exchange and the drawing.
- `list.rs`, `settings.rs`, `click.rs`, `hud.rs`, `tick.rs`: the helpers the hook points call.

It reads the block from the game's own raycast and its name, icon, hardness, state and (for a crop)
its age from the game's tables, so the reach setting works across the whole render distance.

A container's slots are the server's: the window asks for the block under the crosshair, and the
server thread's tick answers with its slots. That works in single player and as a LAN host, where
both sides are one process. On a server this game is not hosting, the slots never reach it at all:
the protocol sends a chest's contents only when its menu opens, so a client-side tooltip cannot see
them without a server plugin of its own.

## Build

The logic on its own:

```sh
cd native && cargo test
```

The mod, compiled into a game build by the launcher:

```sh
horizon-launcher native-build mixins/blockscope --trust-native-code
```

The launcher downloads its own Rust toolchain, applies the mixins, compiles the game and installs the
result as a version of its own, which the instance the mods are installed in then plays. Turn on
**Settings > Game > Native mods (experimental)** first. On Windows 11 the first build takes about
8 minutes and 4.4 GB; a rebuild after a helper changes takes about 3.

## License

MIT.
