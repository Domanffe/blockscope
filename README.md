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
- The block's icon, when the item table has an item for it.
- Its coordinates, on request.

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

The choice is kept between games.

## What it cannot show yet

Horizon's mod API 0.1 hands a mod the player, the world clock and one raycast hit. It has no
`world-read` and no crosshair target with the block's state, so hardness, the right tool, block
states, container contents and entities are out of reach for now. They wait on those interfaces,
not on a bigger mod.

Capabilities it asks for: `game-read` (the player and the raycast), `render` (the box), `recipes`
(the item table, for names and icons), `ui` (its settings page) and `storage` (the choice it keeps).
No chat, no commands.

## Build

```sh
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/ludovic111/horizon cargo-horizon-mod
cargo horizon-mod build
cargo horizon-mod check target/horizon-mod/blockscope.hmod
cargo horizon-mod test
```

## Install

Put `blockscope.hmod` in `horizon-mods/` beside `options.txt`, then turn it on in **Options > Mods**.

## License

MIT.
