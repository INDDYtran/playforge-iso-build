#!/usr/bin/env bash
# Playforge – stellt sicher, dass "releases/latest" auf das Update-Release
# zeigt. playforge-update holt seine Dateien über releases/latest/download;
# zeigt "latest" auf ein ISO-Release, findet es kein Update mehr.
#
# Aufruf: verify-latest-release.sh <owner/repo> [tag]
set -Eeuo pipefail

repo="${1:-${REPO:-}}"
tag="${2:-system-update}"

[[ -n "$repo" ]] || {
    printf 'Aufruf: %s <owner/repo> [tag]\n' "$(basename "$0")" >&2
    exit 1
}

latest="$(gh api "repos/$repo/releases/latest" --jq '.tag_name')"
if [[ "$latest" != "$tag" ]]; then
    printf 'releases/latest zeigt auf "%s" statt auf "%s" – korrigiere.\n' \
        "$latest" "$tag" >&2
    gh release edit "$tag" --repo "$repo" --latest
    latest="$(gh api "repos/$repo/releases/latest" --jq '.tag_name')"
fi

if [[ "$latest" != "$tag" ]]; then
    printf 'Fehler: "%s" ist nicht das neueste Release (aktuell: "%s").\n' \
        "$tag" "$latest" >&2
    exit 1
fi

printf 'releases/latest = %s\n' "$latest"
