# Contributing to Celframe

Read [CLEANROOM.md](CLEANROOM.md) first. It applies to every contribution.

## Sign off your commits (DCO)

Every commit needs a `Signed-off-by` line, which certifies the
[Developer Certificate of Origin](https://developercertificate.org/):

```bash
git commit -s
```

## Workflow

1. Start from an issue. New features need a spec in `docs/specs/` first.
2. Work on one issue per branch, named `<type>/<issue>-<slug>`.
3. Write tests first.
4. Keep PRs small, under about 400 changed lines where possible, not counting lockfiles.
5. Use [conventional commit](https://www.conventionalcommits.org/) messages.
6. Open a PR and complete its checklist. PRs merge by rebase once CI is green.

Build and test commands are in [CLAUDE.md](CLAUDE.md).
