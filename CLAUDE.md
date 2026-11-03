# Celframe: working rules

Free, GPL-3.0-or-later, clean-room 2D animation studio. Only Phase 1 is in scope.

## Clean-room rules (non-negotiable)

Follow [CLEANROOM.md](CLEANROOM.md). Never research proprietary animation products, and
never name them in the repo. If something seems to need their internals, stop and ask.

## Architecture rules

- The Rust core is the single source of truth. The UI sends commands through Tauri IPC and
  renders the state it gets back.
- All edits are commands that can be undone.
- Rendering direction is undecided until ADR-0001 is approved. Don't build on either option
  before then.

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
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
```

## Conventions

- One issue per branch, small PRs, tests first, conventional commits.
- Commit with `git commit -s`. PRs merge by rebase so sign-offs survive.
- Stop and ask before anything irreversible, any dependency with an unusual licence, or
  anything touching the clean-room rules.
- Stay on Tauri 2.x. Don't adopt alpha releases.
