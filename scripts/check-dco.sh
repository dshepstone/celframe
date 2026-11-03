#!/usr/bin/env bash
# Fails if a commit in the given range has no Signed-off-by line from its author.
set -euo pipefail
range="$1"
status=0
for commit in $(git rev-list "$range"); do
  author="$(git show -s --format='%an <%ae>' "$commit")"
  if ! git show -s --format='%B' "$commit" | grep -qF "Signed-off-by: $author"; then
    echo "$commit has no 'Signed-off-by: $author'. Use git commit -s."
    status=1
  fi
done
exit "$status"
