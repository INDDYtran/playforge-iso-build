#!/usr/bin/env bash
# Playforge – zerlegt ein ISO in Release-Assets unterhalb der GitHub-Grenze.
#
# GitHub erlaubt pro Release-Asset höchstens 2 GiB (2 147 483 648 Bytes), aber
# bis zu 1000 Assets pro Release und keine Obergrenze für die Gesamtgröße.
# Ein 3-GB-ISO wird deshalb in Teile geschnitten, die sich mit `cat` wieder
# zusammensetzen lassen.
set -Eeuo pipefail

iso="${1:-}"
out="${2:-}"
part_size="${PLAYFORGE_PART_SIZE:-1800M}"
limit=$((2 * 1024 * 1024 * 1024))

msg() { printf '\n==> %s\n' "$*"; }
die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

[[ -n "$iso" ]] || die "Aufruf: publish-iso-parts.sh <iso> <zielverzeichnis> [teilgröße]"
[[ -f "$iso" ]] || die "ISO nicht gefunden: $iso"
[[ -n "$out" ]] || die "Zielverzeichnis fehlt."

name="$(basename "$iso")"
mkdir -p -- "$out"
rm -f -- "$out/$name".part-*

msg "Zerlege $name ($(du -h "$iso" | cut -f1)) in Teile à $part_size"
# -d: numerische Endung, -a 2: 00 … 99 – genug für 99 × 1,8 GiB.
split -b "$part_size" -d -a 2 -- "$iso" "$out/$name.part-"

parts=("$out/$name".part-*)
[[ -e "${parts[0]}" ]] || die "split hat keine Teile erzeugt."
for part in "${parts[@]}"; do
    size="$(stat -c %s "$part")"
    if (( size >= limit )); then
        die "$(basename "$part") ist mit $size Bytes nicht unter der 2-GiB-Grenze."
    fi
done

msg "Prüfsummen schreiben"
(
    cd "$out"
    sha256sum "$name".part-* >"$name.parts.sha256"
)
if [[ -f "$iso.sha256" ]]; then
    cp -- "$iso.sha256" "$out/$name.sha256"
else
    (
        cd "$(dirname -- "$iso")"
        sha256sum "$name" >"$out/$name.sha256"
    )
fi

# Reihenfolge und Namen für das Zusammensetzen festhalten.
{
    printf 'Playforge ISO %s\n' "$name"
    printf 'Teile: %s\n' "${#parts[@]}"
    printf 'Teilgröße: %s\n' "$part_size"
    printf '\n'
    printf '1. Alle Teile herunterladen: %s.part-00 bis %s.part-%02d\n' \
        "$name" "$name" "$((${#parts[@]} - 1))"
    printf '2. Zusammensetzen:  cat %s.part-* > %s\n' "$name" "$name"
    printf '3. Prüfen:          sha256sum -c %s.sha256\n' "$name"
    printf '4. Auf USB schreiben: sudo dd if=%s of=/dev/sdX bs=4M status=progress oflag=sync\n' "$name"
} >"$out/REJOIN.txt"

{
    printf '## Playforge Linux\n\n'
    printf 'Das ISO liegt wegen der 2-GiB-Grenze von GitHub in **%s Teilen**.\n\n' "${#parts[@]}"
    printf '| Datei | Größe |\n|---|---|\n'
    for part in "${parts[@]}"; do
        printf '| `%s` | %s |\n' "$(basename "$part")" "$(du -h "$part" | cut -f1)"
    done
    printf '| `%s.sha256` | Prüfsumme des ganzen ISO |\n' "$name"
    printf '| `%s.parts.sha256` | Prüfsummen der Teile |\n' "$name"
    printf '| `REJOIN.txt` | Anleitung |\n\n'
    printf '```bash\n'
    printf 'cat %s.part-* > %s\n' "$name" "$name"
    printf 'sha256sum -c %s.sha256\n' "$name"
    printf 'sudo dd if=%s of=/dev/sdX bs=4M status=progress oflag=sync\n' "$name"
    printf '```\n\n'
    printf 'Nur die Teile herunterladen – einzelne Teile sind kein bootfähiges ISO.\n'
} >"$out/RELEASE.md"

msg "Fertig"
printf '  Teile:   %s\n' "${#parts[@]}"
for part in "${parts[@]}"; do
    printf '    %-52s %s\n' "$(basename "$part")" "$(du -h "$part" | cut -f1)"
done
printf '  Ziel:    %s\n' "$out"
