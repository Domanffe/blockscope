# Publishing

The artifact is built and ready. Only the submit form is left, and it needs a GitHub sign-in, which
only you can do.

## The file

`C:\Users\world\Documents\HermesWorkspace\mods\blockscope\dist\blockscope-0.3.1.hnmod` — 13776 bytes
SHA-256 `4fea94e5337aa82df3d981fc6231fecc80161262add577b596bb6661e0e303d1`

It holds `horizon-native.json` (game 26.3, schema 1, hooks 2, pinned to Horizon `4aa6854` and
horizon-mixins `a2af5c0`) and `mixins/blockscope/`.

## The form

https://playhorizon.dev/mods/submit — GitHub sign-in, then:

- **Project id**: `blockscope` (free text; the existing id is what makes this an update)
- **Name**: Blockscope
- **Summary** (≤160 chars): The block under your crosshair: its name, id, distance and icon.
- **Description** (first paragraph only is kept): a Jade-style tooltip for Horizon — the block under
  your crosshair with its name in your language, block id, distance and icon, plus hardness, the tool
  it wants, its state properties, a crop's growth and, for a container, its slots. Ten switches on its
  own settings page.
- **Authors**: Alexander (Domanffe)
- **Licence**: MIT
- **Source**: https://github.com/Domanffe/blockscope
- **File**: `blockscope-0.3.1.hnmod`
- **Changelog**: Rebuilt on the 2026-11-01 Nightly (Horizon `4aa6854`). All five hook points are named
  now (mods, settings, settings_click, hud, tick), so they follow the game if it moves them.
- Public key and signature: leave empty (unsigned).
