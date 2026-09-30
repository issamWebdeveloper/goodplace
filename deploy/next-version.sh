#!/usr/bin/env bash
# Affiche la prochaine version (vX.Y.Z) d'après les commits depuis le dernier tag :
#   majeure : « BREAKING CHANGE » dans le message, type suivi de « ! » (feat!: …) ou [major]
#   mineure : commit « feat: … » / « feat(portée): … » ou [minor]
#   corrective : tout le reste (fix, style, docs, chore…)
set -euo pipefail

last="$(git describe --tags --abbrev=0 --match 'v[0-9]*.[0-9]*.[0-9]*' 2>/dev/null || true)"
if [[ -z "$last" ]]; then
  echo v0.1.0
  exit 0
fi

log="$(git log --format='%s%n%b' "$last..HEAD")"
IFS=. read -r major minor patch <<<"${last#v}"

if grep -qE '^[a-z]+(\([^)]*\))?!:|BREAKING CHANGE|\[major\]' <<<"$log"; then
  echo "v$((major + 1)).0.0"
elif grep -qE '^feat(\([^)]*\))?:|\[minor\]' <<<"$log"; then
  echo "v$major.$((minor + 1)).0"
else
  echo "v$major.$minor.$((patch + 1))"
fi
