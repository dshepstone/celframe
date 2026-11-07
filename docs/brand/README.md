# Celframe brand

Celframe's logo is two overlapping frames, like cels laid on top of each other.

| File             | Use                                                              |
| ---------------- | ---------------------------------------------------------------- |
| `mark.svg`       | The frames alone, on any background                              |
| `tile.svg`       | The frames on a dark rounded tile: the app icon and the menu row |
| `logo.svg`       | The frames and the name, for light backgrounds                   |
| `logo-light.svg` | The frames and the name, for dark backgrounds                    |

- Colours: front frame `#f24b36`, back frame `#e38a9c`, tile `#0e0e10`.
- The app's icons in `apps/desktop/src-tauri/icons/` are generated from `apps/desktop/app-icon.svg`, a copy of `tile.svg`, with `pnpm --filter @celframe/desktop tauri icon app-icon.svg`.
- In the app, `apps/desktop/src/studio/brand.tsx` draws the same shapes.
- The name in the logo is set in the system's bold sans serif.
