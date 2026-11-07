# Contributing to Celframe

Thanks for helping. Celframe is built with students in mind, so first contributions are welcome. Look for issues labelled `good first issue`.

Read [CLEANROOM.md](CLEANROOM.md) first. It applies to every contribution.

## Set up

Follow the build steps in the [README](README.md#build-and-run). Then check that everything passes:

```bash
cargo fmt --all --check
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

```bash
cargo nextest run --workspace
```

```bash
pnpm format:check
```

```bash
pnpm lint
```

```bash
pnpm test
```

## Sign off your commits (DCO)

Every commit needs a `Signed-off-by` line, which certifies the
[Developer Certificate of Origin](https://developercertificate.org/). CI checks it.

```bash
git commit -s
```

## Workflow

1. Start from an issue. New features need a spec in `docs/specs/` first, written in your own words.
2. Work on one issue per branch, named `<type>/<issue>-<slug>`, such as `feat/42-onion-skin`.
3. Write tests first.
4. Keep PRs small, under about 400 changed lines where possible, not counting lockfiles.
5. Use [conventional commit](https://www.conventionalcommits.org/) messages.
6. Open a PR and complete its clean-room and test checklists. PRs merge by rebase once CI is green, so each commit keeps its sign-off.

## Rules that CI enforces

- Formatting, lint and tests pass on Windows, macOS and Linux.
- Every dependency's licence is on the allowlist in `deny.toml` and `scripts/check-js-licences.mjs`. A new licence needs a maintainer's decision.
- No banned product names appear anywhere in the repo.
- The golden-image test matches its reference render. See `crates/celframe-render/tests/golden/README.md` to update a reference.

## Architecture in brief

- The Rust core owns the document. The UI sends commands over Tauri IPC and renders the state it gets back.
- Every edit is an undoable command.
- The canvas is drawn natively with wgpu under a transparent webview. See [ADR-0001](docs/adr/0001-rendering.md).

Decisions are recorded in `docs/adr/`. If you want to change one, open an issue first.
