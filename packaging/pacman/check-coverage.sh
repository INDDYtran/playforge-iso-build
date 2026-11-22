#!/usr/bin/env bash
# Playforge – prüft, dass jede versionierte Overlay-Datei in genau einem Paket
# landet. Läuft nach `git add` (nutzt git ls-files, damit unversionierte
# Arbeitsdateien nicht stören).
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
base="$repo_dir/packaging/pacman"
tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

dummy="$tmp/playforge-control-center"
printf '#!/bin/sh\nexit 0\n' >"$dummy"
chmod 0755 "$dummy"
PLAYFORGE_CONTROL_CENTER="$dummy" bash "$base/stage.sh" >/dev/null

{
    git -C "$repo_dir" ls-files overlay
    # Paketdatei außerhalb von overlay/: der Schlüssel wird bewusst nicht per
    # Tar-Update verteilt, sondern nur vom Paket installiert.
    printf 'overlay/usr/share/playforge/pacman-repo.asc\n'
    printf 'overlay/etc/pacman.d/playforge.conf\n'
} | sed 's|^overlay/||' |
    grep -v '^etc/pacman.conf$' |
    LC_ALL=C sort >"$tmp/all"

find "$base" -path '*/stage/*' -type f -printf '%P\n' |
    sed 's|^[^/]*/stage/||' |
    grep -v '^usr/local/bin/playforge-control-center$' |
    LC_ALL=C sort >"$tmp/staged"

if ! diff -u "$tmp/all" "$tmp/staged" >"$tmp/diff"; then
    printf 'Overlay-Dateien und Paket-Staging weichen ab:\n\n' >&2
    cat "$tmp/diff" >&2
    exit 1
fi
printf 'OK: %s Overlay-Dateien sind genau einem Paket zugeordnet.\n' "$(wc -l <"$tmp/all")"
