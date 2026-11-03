# Clean-room rules

Celframe is written from general animation knowledge and our own specs. These rules
protect the project and everyone who contributes to it. They apply to every contribution,
whether written by a person or with an AI tool.

## You must not

- Look at, decompile or reproduce code, binaries, SDK headers, scripts, file formats,
  icons, layouts, themes, sample files or documentation text from proprietary animation
  software.
- Use the names of proprietary animation products in code, UI, docs or package names.
- Contribute anything you learned under an NDA or from a proprietary product's internals.
- Imitate an existing animation app's layout or visual design.

## You must

- Design from general animation knowledge and from specs in `docs/specs/`.
- Write a spec before code for any new feature.
- Use only dependencies with licences compatible with GPL-3.0-or-later. CI enforces this.
- Sign off every commit (see [CONTRIBUTING](CONTRIBUTING.md)).
- If something seems to need knowledge of a proprietary product's internals, stop and open
  an issue labelled `cleanroom-review` before writing anything.

## Having used other software

Knowing how to animate, including having used commercial tools as an artist, doesn't
disqualify you. Describe what animators need in your own words, not how a product
implements it.

## If you find a problem

If you think something in the repository breaks these rules, open an issue labelled
`cleanroom-review`. The material will be reviewed and removed if needed.
