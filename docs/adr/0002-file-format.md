# ADR-0002: File format

Status: accepted

## Context

Students need files that are safe, inspectable and able to outlive any one version of Celframe. Projects will later hold large assets such as images and audio. The full spec is in [the file format document](../file-format.md).

## Decision

- **A zip of JSON files and assets.** Zip tools are everywhere, JSON is readable, and binary assets can sit alongside the JSON later without being encoded into it.
- **One JSON entry per scene.** This keeps a future reader free to load one scene at a time.
- **Reproducible output.** Fixed entry order and timestamps make identical projects byte-identical, which helps testing and version control.
- **Atomic saves.** Saving writes a temp file beside the target, syncs it, then renames it over the target, so a crash or failed save can't damage the existing file.
- **Validation in both directions.** The same rules run on save and load, so bad data is caught before it can replace a good file.
- **Migrations on raw JSON.** Rust types only describe the current schema, so older files are upgraded one version at a time before being parsed.
- **Exact floats.** JSON numbers use shortest round-trip output and `float_roundtrip` parsing, so positions come back with identical bits. The golden-image test depends on this.

## Consequences

- Each save rewrites the whole archive, which may become slow for large projects. Later phases can revisit this, for example by copying asset entries without recompressing them.
- JSON is larger and slower than a binary format. Compression offsets some of the size, and stroke data can move to a binary asset in a later schema version if needed.
- `.celf` is tied to the working name. Decide on it before Release 0.1, when files will start to exist in the wild.
