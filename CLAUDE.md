# Celframe: working rules

Free, GPL-3.0-or-later, clean-room 2D animation studio. Work proceeds phase by phase: each
feature works fully before the next starts (maintainer decision). See
[the feature inventory](docs/features/inventory.md).

## Clean-room rules (non-negotiable)

Follow [CLEANROOM.md](CLEANROOM.md). Never research proprietary animation products, and
never name them in the repo. If something seems to need their internals, stop and ask.

## Architecture rules

- The Rust core is the single source of truth. The UI sends commands through Tauri IPC and
  renders the state it gets back.
- All edits are commands that can be undone.
- Rendering is native wgpu under a transparent webview ([ADR-0001](docs/adr/0001-rendering.md)).
  The webview stays transparent over the canvas area.
- New IDs come from the core, never the UI.
- The canvas view (zoom, pan, rotation) is display state: not undoable, not saved.
- The interface follows [the workspace spec](docs/specs/workspaces.md): an original layout. Every
  control either works or says which phase it arrives in.

## Layout

- `crates/celframe-core`: document model, commands, undo/redo
- `crates/celframe-geom`: strokes, Bezier curves, smoothing
- `crates/celframe-render`: wgpu renderer
- `crates/celframe-format`: `.celf` read and write
- `apps/desktop`: Tauri 2, React, TypeScript, Vite
- `packages/ui`: design tokens, components, Storybook
- `docs/adr`, `docs/specs`, `docs/features/inventory.md`, `docs/file-format.md`

## Commands

```bash
pnpm install
pnpm --filter @celframe/desktop tauri dev   # run the app
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
pnpm format:check && pnpm lint && pnpm test
pnpm licenses:check && cargo deny check licenses bans sources
pnpm cleanroom:check
```

- Renderer tests need a GPU or software adapter. CI sets `CELFRAME_REQUIRE_GPU` so a missing
  adapter fails instead of skipping.
- The golden-image references in `crates/celframe-render/tests/golden/` come from CI runners.
  Download a failed run's `golden-<os>` artifact, review it, and commit it as `<os>.png`.

## Conventions

- One issue per branch, small PRs, tests first, conventional commits.
- Commit with `git commit -s`. PRs merge by rebase so sign-offs survive.
- Stop and ask before anything irreversible, any dependency with an unusual licence, or
  anything touching the clean-room rules.
- Stay on Tauri 2.x. Don't adopt alpha releases.
- Stay on TypeScript 6.0.x for now: typescript-eslint doesn't support TypeScript 7 yet.
- Licence exceptions approved by the maintainer: 0BSD everywhere; BlueOak-1.0.0 and CC-BY-4.0
  for dev-only JS packages; Apache-2.0 WITH LLVM-exception for `target-lexicon` only.
- Don't use reverse-engineering tools on any other animation software, and don't copy
  layouts, icons, names or documentation from one.
