# Blockscope

A Jade-style block tooltip for [Horizon](https://github.com/ludovic111/horizon): the block under your
crosshair, its name in your language, its id and how far it is, in a small box that you can put where
you want.

![Blockscope in the top right corner](docs/screenshot.png)

## What it shows

- The block's name as your game language names it, from the item registry; blocks with no item get
  their id with the words capitalised ("Water").
- The block id, so a modded block is identifiable.
- The distance from your feet to the point the crosshair meets the block.
- The block's icon, drawn by the game's own item renderer.
- Its hardness, and whether what you hold is the tool it wants ("Hardness 3.0 · needs the right
  tool", "this tool harvests it", "Unbreakable").
- The block's state properties, as the game names them (`facing=north,type=single`).
- A crop's growth out of the age it ripens at ("Growth 5/7 (71%)").
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

**Container contents does not draw yet.** A chest's slots reach the client only when its menu is
open: the game's client-side `BlockEntity` is made empty with the chunk and updated only by
`remote_block_entity_data`, which needs an entity it already has. The mod's switch is there for the
version that reads the integrated server (see below), and shows nothing until then.


## Native mod (0.4.0)

Horizon's WebAssembly mod API is gone: `blockscope.hmod` (0.2.0) does not run any more. 0.4.0 is a
native mod made with [horizon-mixins](https://github.com/horizon-rust/horizon-mixins), the bundle
under `mixins/blockscope/` that hooks the game's own code and is compiled into a build of the game:

- `horizon-mixin.toml`: the two modules and the four hook points (`native_hud`, `native_mods`,
  `native_settings`, `native_settings_click`).
- `runtime.rs`: the settings, the rows and where the box goes, with no game types in them, so its
  tests run without a Horizon build.
- `glue.rs`: the crosshair hit, the item table and the drawing.
- `list.rs`, `settings.rs`, `click.rs`, `hud.rs`: the helpers the hook points call.

It reads the block from the game's own raycast and its name, icon, hardness, state and (for a crop)
its age from the game's tables, so the reach setting works across the whole render distance.

What it still does not show is a container's contents on a server it is not hosting: those slots live
on the server, and the vanilla protocol sends them only when a menu opens. Jade solves it with its
server plugin; here the pieces are in one process, so the plan is a hook on the server thread's tick
that answers the window through a shared cell. On a remote server that stays out of reach without
changing the protocol.

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
