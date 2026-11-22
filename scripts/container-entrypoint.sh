#!/usr/bin/env bash
# Einstiegspunkt des Playforge-Builder-Containers.
set -Eeuo pipefail

cd /build

if [[ -z "${SOURCE_DATE_EPOCH:-}" || ! "${SOURCE_DATE_EPOCH}" =~ ^[0-9]+$ ]]; then
    echo "Fehler: SOURCE_DATE_EPOCH fehlt oder ist keine Unix-Zeit." >&2
    exit 1
fi

# Update-Paket statt ISO: nur Control Center bauen und das Release-Archiv
# schnüren. Wird von .github/workflows/publish-update.yml genutzt.
if [[ "${PLAYFORGE_JOB:-iso}" == "payload" ]]; then
    printf '\n==> Playforge Systemupdate paketieren\n'
    printf '    Version:          %s\n' "${PLAYFORGE_VERSION:-unbekannt}"
    printf '    SOURCE_DATE_EPOCH: %s\n\n' "${SOURCE_DATE_EPOCH}"

    bash scripts/build-control-center.sh
    bash scripts/build-file-browser.sh
    bash scripts/pack-update.sh
    exit 0
fi

printf '\n==> Playforge ISO-Build im Container\n'
printf '    archiso:          %s\n' "$(pacman -Q archiso)"
printf '    Basis-Image:      %s\n' "${PLAYFORGE_BASE_IMAGE:-unbekannt}"
printf '    Paket-Snapshot:   %s\n' "${ARCHIVE_DATE:-aktuelle Mirrors}"
printf '    SOURCE_DATE_EPOCH: %s (%s)\n\n' \
    "${SOURCE_DATE_EPOCH}" "$(date -u -d "@${SOURCE_DATE_EPOCH}" --iso-8601=seconds)"

mkdir -p out

# Alle Optionen werden an build.sh weitergegeben (z. B. --no-check).
bash build.sh "$@"

iso="$(find out -maxdepth 1 -type f -name 'playforge-*.iso' -printf '%T@ %p\n' \
    | sort -n | tail -n1 | cut -d' ' -f2-)"
if [[ -z "$iso" || ! -f "$iso" ]]; then
    echo "Fehler: Kein fertiges ISO in out/ gefunden." >&2
    exit 1
fi

bash /usr/local/bin/playforge-verify-iso "$iso"

(
    cd out
    sha256sum -c "$(basename "$iso").sha256"
)

sha256="$(cut -d' ' -f1 "out/$(basename "$iso").sha256")"

if [[ "${PLAYFORGE_SKIP_BOOT_TEST:-0}" == "1" ]]; then
    rm -f out/smoke-test.log out/smoke-test.log.qemu
    printf '\n==> Boot-Smoke-Test übersprungen (PLAYFORGE_SKIP_BOOT_TEST=1)\n'
    boot_test="übersprungen"
else
    printf '\n==> Boot-Smoke-Test (QEMU/OVMF, serielle Konsole)\n'
    PLAYFORGE_SMOKE_LOG="out/smoke-test.log" \
        bash /usr/local/bin/playforge-boot-smoke "$iso"
    boot_test="bestanden"
fi

{
    printf 'Playforge ISO\n'
    printf 'ISO: %s\n' "$(basename "$iso")"
    printf 'SHA256: %s\n' "$sha256"
    printf 'SOURCE_DATE_EPOCH: %s (%s)\n' \
        "$SOURCE_DATE_EPOCH" "$(date -u -d "@${SOURCE_DATE_EPOCH}" --iso-8601=seconds)"
    printf 'Paket-Snapshot: %s\n' "${ARCHIVE_DATE:-aktuelle Mirrors}"
    printf 'archiso: %s\n' "$(pacman -Q archiso)"
    printf 'Basis-Image: %s\n' "${PLAYFORGE_BASE_IMAGE:-unbekannt}"
    printf 'Boot-Smoke-Test: %s\n' "$boot_test"
} > out/BUILD-INFO.txt
chmod 0644 out/BUILD-INFO.txt

printf '\n==> Fertig\n'
printf '    ISO:    %s\n' "$iso"
printf '    SHA256: %s\n' "$sha256"
printf '    Info:   out/BUILD-INFO.txt\n'
if [[ -f out/smoke-test.log ]]; then
    printf '    Log:    out/smoke-test.log\n'
fi
