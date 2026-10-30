# Publishing Blockscope 0.3.0

Everything below is prepared; nothing has been pushed or submitted yet.

## The file

- `blockscope-0.3.0.hnmod`, 13,690 bytes
- SHA-256 `ba4a1c887ab26a2a9189c671cbdbea3ce2ed212a6e3d32f1fffaa60386198358`
- Inside: `horizon-native.json` and `mixins/blockscope/` (8 files, the bundle *is* the source)
- Pins Horizon `a46ce6898bd72016e061aaec1afc1375b384deac` (Nightly 2026-10-30) and horizon-mixins
  `3f99a5c60dfa311cc365524c44ae2fd799bfa1de`; `"hooks": 2`, so the launcher may build it on any later
  Nightly whose `NATIVE_HOOKS_API` is at least 2.

## What was verified, and how

- **Built and played**: the launcher compiled it into `native-a46ce68-d645c4417f63` (463 s) and the
  game ran. Screenshots read back: stone shows an iron pickaxe beside `Hardness 1.5 · needs the right
  tool`; a chest shows a 9×3 grid with 12 diamonds, 7 oak logs and 3 bread; wheat shows
  `Ростки пшеницы … State: age=5 … Growth 5/7 (71%)`; a cow shows a heart and `10 / 10`.
- **Tests**: `cd native && cargo test` — 16 tests, the settings file format, growth, the tool table,
  the container grid and the "this answer belongs to this block" rule among them.
- **The registry's own reader accepts it**: `horizon_mods::index::inspect(Kind::NativeMod, file)` (run
  against the staged `horizon-mods` crate) answers `id: blockscope, version: 0.3.0, game: 26.3,
  api: native-1/a46ce68…/3f99a5c…`, and the SHA-256 it computes matches the one above.
- **The published CLI cannot check it**: the installed `cargo-horizon-mod` answers
  `the registry doesn't take native yet` for a `.hnmod`, and derives no kind from the extension. That is
  the tool being older than the index code, not a problem with the file.

## The registry entry

Replace `mods/blockscope.toml` with this. The two `.hmod` versions stay listed below the new one, as
`mods/zoom.toml` does; the index stops offering them and the launcher's update keeps a player's copy.

```toml
id = "blockscope"
kind = "native-mod"
name = "Blockscope"
summary = "The block under your crosshair: its name, id, distance and icon, with reach, placement and opacity on its own settings page. A Jade-style tooltip for Horizon."
description = """
A Jade-style tooltip for Horizon: the block under your crosshair, its name in your language, its block
id, how far it is and its icon.

- Hardness, and the icon of the tool the block wants; unbreakable blocks say so.
- The block's own state properties, and a crop's growth out of the age it ripens at.
- A container's slots, laid out as its own screen has them (chest, barrel, hopper, dispenser, furnace,
  brewing stand). The window asks the server thread, so this works in single player and as a LAN host;
  a remote server never sends a chest's contents, so there the grid stays away.
- A creature instead of a block: its name and its health, with the HUD's own heart.
- Ten switches on its own page: reach (5 to 1024 blocks), position, background opacity, coordinates,
  item icon, hardness and tool, block states, crop growth, container contents, creatures.

Now a native mod (0.3.0): built into the game by the launcher from a mixin bundle, like any native mod
(Settings → Game → Native mods (experimental), then Build). Its settings carry over from 0.2.0, in
horizon-mods/.horizon-mod-data/blockscope/. Its source is
[blockscope/mixins/blockscope](https://github.com/Domanffe/blockscope/tree/main/mixins/blockscope).
"""
authors = ["Domanffe"]
owners = ["Domanffe"]
license = "MIT"
source = "https://github.com/Domanffe/blockscope"

[[versions]]
version = "0.3.0"
published = "2026-10-30"
game = "26.3"
file = "blockscope-0.3.0.hnmod"
sha256 = "ba4a1c887ab26a2a9189c671cbdbea3ce2ed212a6e3d32f1fffaa60386198358"
changelog = "Native: the WebAssembly mod rewritten as a mixin bundle for the launcher's native mods, now with hardness and the tool's icon, block states, crop growth, a container's slots and a creature's health, each on its own switch."

[[versions]]
version = "0.2.0"
published = "2026-10-29"
game = "26.3"
file = "blockscope-0.2.0.hmod"
sha256 = "dcb8a3d3eefe37ebc7fd1886412dc9799c5fa4dee88fd419e7f46a712df00c12"
changelog = "Reach, placement, background, coordinates and the item icon on its own settings page (Options > Mods > Blockscope), kept between games."

[[versions]]
version = "0.1.0"
published = "2026-10-27"
game = "26.3"
file = "blockscope-0.1.0.hmod"
sha256 = "8c9de32150bc845c6e1d1d28a28d6f1746cea8c124aa7e683acdd597c1b49325"
changelog = "First release: the block under the crosshair, its name, id and distance, in a box beside the crosshair."
```

## The launcher icon, in the same PR

Add `icons/blockscope.png`: a copy of `docs/icon.png` (256×256 RGBA, the size the registry's other
icons use). Drop the entry's `icon = "https://raw.githubusercontent.com/…"` line: the registry derives
`https://horizon-rust.github.io/horizon-mods/icons/<id>.png` from the file, and the launcher fetches
images only from a short host list (`launcher/src/browser/images.rs`: Modrinth's and CurseForge's CDNs
and the registry's own Pages) — `raw.githubusercontent.com` is refused there, which is why Blockscope
has no icon in the mods list today.

## Submitting

The form is `playhorizon.dev/mods/submit` behind the author's GitHub login; there is no project picker,
so typing `blockscope` as the id is what turns this into an update. Fields:

- project id `blockscope`, version `0.3.0`, game version `26.3`
- file: the `blockscope-0.3.0.hnmod` attached with this note
- SHA-256: `ba4a1c887ab26a2a9189c671cbdbea3ce2ed212a6e3d32f1fffaa60386198358`
- changelog: the one line from the entry above
- description: **only the first paragraph survives**, so either send the single paragraph or attach the
  full text as a comment for the reviewer to paste
- leave the public key and signature empty: the entry is unsigned, and adding a key makes every later
  version require one

## What to say about the limits

State these in the PR or the entry rather than letting a reviewer find them:

- container contents come from the integrated server, so single player and a LAN host only; a server
  this game is not hosting never sends a chest's contents over the vanilla protocol.
- pinned to Nightly 2026-10-30; `hooks: 2` should let later Nightlies build it, untested.
- the mod compiles into the game, so a broken bundle fails the launcher's build instead of degrading.
