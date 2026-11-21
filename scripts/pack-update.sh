#!/usr/bin/env bash
# Playforge – packt das Systemupdate (Overlay + Control Center) für ein
# GitHub-Release. Ergebnis:
#
#   out/playforge-update.tar.gz           Nutzdaten + manifest.tsv
#   out/playforge-update.tar.gz.sha256    Prüfsumme des Archivs
#   out/playforge-update.version          Versionskennung für playforge-update
#
# Das Archiv enthält nur, was der Updater schreiben darf: die Dateien aus
# overlay/ (identisch zum Installations-Overlay) und das fertig gebaute
# Control Center.
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
out_dir="${PLAYFORGE_UPDATE_OUT:-$repo_dir/out}"
version="${PLAYFORGE_VERSION:-}"
binary="${PLAYFORGE_CONTROL_CENTER:-}"
epoch="${SOURCE_DATE_EPOCH:-$(date -u +%s)}"

msg() { printf '\n==> %s\n' "$*"; }
die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

if [[ -z "$version" ]]; then
    if version="$(git -C "$repo_dir" rev-parse --short=12 HEAD 2>/dev/null)" && [[ -n "$version" ]]; then
        :
    else
        version="$(date -u +%Y%m%d-%H%M)"
    fi
fi
[[ "$version" =~ ^[A-Za-z0-9._-]+$ ]] || die "Ungültige Versionskennung: $version"

if [[ -z "$binary" ]]; then
    for candidate in \
        "$out_dir/control-center/playforge-control-center" \
        "$repo_dir/control-center/target/release/playforge-control-center" \
        "$repo_dir/control-center/target/debug/playforge-control-center"; do
        if [[ -f "$candidate" ]]; then
            binary="$candidate"
            break
        fi
    done
fi
[[ -n "$binary" && -f "$binary" ]] ||
    die "Control-Center-Binary nicht gefunden. Erst scripts/build-control-center.sh ausführen."

stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
payload="$stage/payload"
mkdir -p "$payload"

msg "Sammle Overlay-Dateien"
cp -a "$repo_dir/overlay/." "$payload/"

# pacman.conf gehört dem installierten System: Mirrors, eigene Repositories und
# Halte-Regeln des Nutzers dürfen von einem Update nicht überschrieben werden.
rm -f -- "$payload/etc/pacman.conf"
install -Dm0755 "$binary" "$payload/usr/local/bin/playforge-control-center"
mkdir -p "$payload/etc/playforge"
printf '%s\n' "$version" >"$payload/etc/playforge/version"

msg "Erzeuge Manifest"
# Erlaubte Zielpfade aus dem Updater übernehmen – so gibt es nur eine Quelle
# und kein Update kann entstehen, das der Updater später ablehnt.
allowed=()
while IFS= read -r entry; do
    if [[ -n "$entry" ]]; then
        allowed+=("$entry")
    fi
done < <(sed -n '/^allowed_prefixes=(/,/^)/p' \
    "$repo_dir/overlay/usr/local/bin/playforge-update" |
    grep -oE '"[^"]+"' | tr -d '"')
(( ${#allowed[@]} > 0 )) || die "Erlaubte Pfade konnten nicht aus playforge-update gelesen werden."

is_allowed() {
    local rel="$1" prefix
    for prefix in "${allowed[@]}"; do
        [[ "$rel" == "$prefix"* ]] && return 0
    done
    return 1
}

manifest="$stage/manifest.tsv"
: >"$manifest"
unknown=()
while IFS= read -r rel; do
    [[ -n "$rel" ]] || continue
    if ! is_allowed "$rel"; then
        unknown+=("$rel")
        continue
    fi
    sum="$(sha256sum "$payload/$rel" | cut -d' ' -f1)"
    mode="$(stat -c '%a' "$payload/$rel")"
    # Ein Windows-Checkout verliert das Ausführungsbit. Dateien mit Shebang
    # sind Skripte und müssen auf dem Zielsystem ausführbar sein – dieselbe
    # Regel, die playforge-install beim Kopieren anwendet.
    if [[ "$(head -c 2 "$payload/$rel")" == "#!" ]]; then
        mode="755"
    fi
    printf '%s\t%s\t%s\n' "$sum" "$mode" "$rel" >>"$manifest"
done < <(cd "$payload" && find . -type f -printf '%P\n' | LC_ALL=C sort)

if (( ${#unknown[@]} > 0 )); then
    printf 'Nicht erlaubte Pfade:\n' >&2
    printf '  %s\n' "${unknown[@]}" >&2
    die "allowed_prefixes in overlay/usr/local/bin/playforge-update ergänzen."
fi
files="$(wc -l <"$manifest")"

msg "Packe Archiv"
mkdir -p "$out_dir"
archive="$out_dir/playforge-update.tar.gz"
tar -C "$stage" \
    --sort=name \
    --owner=0 --group=0 --numeric-owner \
    --mtime="@${epoch}" \
    -czf "$archive" manifest.tsv payload

(
    cd "$out_dir"
    sha256sum playforge-update.tar.gz >playforge-update.tar.gz.sha256
)
printf '%s\n' "$version" >"$out_dir/playforge-update.version"

msg "Fertig"
printf '  Version:  %s\n' "$version"
printf '  Dateien:  %s\n' "$files"
printf '  Archiv:   %s (%s)\n' "$archive" "$(du -h "$archive" | cut -f1)"
printf '  Prüfsumme: %s\n' "$(cut -d' ' -f1 "$out_dir/playforge-update.tar.gz.sha256")"
