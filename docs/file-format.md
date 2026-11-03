# The .celf file format

This is the open specification of Celframe's project format. It describes schema version 1.

## Container

A `.celf` file is a standard zip archive. Anyone can open it with ordinary zip tools.

| Entry              | Required | Contents                                       |
| ------------------ | -------- | ---------------------------------------------- |
| `project.json`     | Yes      | The project manifest                           |
| `scenes/<id>.json` | Yes      | One entry per scene, named by its ID           |
| `assets/`          | No       | Reserved for later phases. Empty in version 1. |

- Text is UTF-8 JSON, compressed with deflate.
- Celframe writes entries in a fixed order with fixed timestamps: `project.json`, scenes in project order, then `assets/`. The same project always gives the same bytes.
- Readers should ignore entries they don't recognize. Celframe doesn't keep them when it saves again.
- Celframe reads at most 1 GiB of decompressed JSON from one file.

## project.json

```json
{
  "schemaVersion": 1,
  "id": "0190f0a0-0000-7000-8000-000000000001",
  "name": "Bouncing ball",
  "scenes": ["0190f0a0-0000-7000-8000-000000000002"]
}
```

| Field           | Type          | Meaning                                         |
| --------------- | ------------- | ----------------------------------------------- |
| `schemaVersion` | integer ≥ 1   | The schema version of the whole file            |
| `id`            | UUID          | The project ID                                  |
| `name`          | string        | The project name                                |
| `scenes`        | array of UUID | Scene order. Must contain at least one scene ID |

## Scene entries

```json
{
  "id": "0190f0a0-0000-7000-8000-000000000002",
  "name": "Scene 1",
  "width": 1920,
  "height": 1080,
  "frameRate": 24,
  "layers": [
    {
      "id": "0190f0a0-0000-7000-8000-000000000003",
      "name": "Layer 1",
      "visible": true,
      "locked": false,
      "drawings": [
        {
          "id": "0190f0a0-0000-7000-8000-000000000004",
          "strokes": [
            {
              "id": "0190f0a0-0000-7000-8000-000000000005",
              "baseWidth": 4.0,
              "color": [0, 0, 0, 255],
              "samples": [{ "x": 0.0, "y": 0.0, "pressure": 0.5 }]
            }
          ]
        }
      ],
      "exposures": [
        {
          "startFrame": 0,
          "length": 2,
          "drawing": "0190f0a0-0000-7000-8000-000000000004"
        }
      ]
    }
  ]
}
```

Units and axes are defined in [the coordinate spec](specs/units-and-coordinates.md). Lists run from the bottom of the stack to the top: the first layer is the lowest layer, and the first stroke is drawn first.

## Validity rules

A reader must reject a file that breaks any of these rules. Celframe also refuses to write such a document.

- A project has at least one scene, and each scene's `id` matches its entry name.
- IDs are unique among scenes in a project, layers in a scene, drawings in a layer and strokes in a drawing.
- A scene's `width`, `height` and `frameRate` are greater than 0.
- Stroke positions are finite numbers, and `baseWidth` is finite and not negative.
- `pressure` is from 0.0 to 1.0.
- `color` is straight RGBA, with 8 bits per channel.
- Exposures are sorted by `startFrame`, don't overlap, last at least 1 frame, and use a drawing on the same layer.

Undo history and view state, such as zoom, aren't stored.

## Schema versions

- A file with a **newer** version than the reader supports must be refused, not guessed at.
- A file with an **older** version is upgraded in memory, one version at a time, on raw JSON. The original file isn't changed until the user saves.
- Version 0 or a missing version is invalid.
- Any change to saved data raises the version and adds a migration and a round-trip test.

| Version | Changes       |
| ------- | ------------- |
| 1       | First version |
