#!/usr/bin/env bash
# Prüft, ob das erzeugte ISO die für BIOS- und UEFI-Boot nötigen Artefakte enthält.
set -Eeuo pipefail

iso="${1:-}"
if [[ -z "$iso" || ! -f "$iso" ]]; then
    echo "Aufruf: $0 <playforge.iso>" >&2
    exit 2
fi

echo "==> Prüfe ISO-Struktur: $iso"

files="$(bsdtar -tf "$iso")" || {
    echo "Fehler: ISO-Inhalt konnte nicht gelesen werden." >&2
    exit 1
}

has_path() {
    local path="$1"
    # bsdtar kann ISO9660-Namen mit/ohne fuehrenden ./ oder ;1 ausgeben.
    grep -Fqi -- "$path" <<<"$files"
}

missing=0
for path in \
    "playforge/x86_64/airootfs.sfs" \
    "playforge/boot/x86_64/vmlinuz-linux" \
    "playforge/boot/x86_64/initramfs-linux.img" \
    "EFI/BOOT/BOOTX64.EFI" \
    "boot/syslinux/isolinux.bin"; do
    if ! has_path "$path"; then
        echo "Fehlt im ISO: $path" >&2
        missing=1
    fi
done
if (( missing != 0 )); then
    exit 1
fi

torito="$(xorriso -indev "$iso" -report_el_torito plain 2>&1)" || {
    echo "Fehler: El-Torito-Informationen konnten nicht gelesen werden." >&2
    exit 1
}

if ! grep -q 'El Torito boot img.*BIOS' <<<"$torito"; then
    echo "Fehler: Kein BIOS-El-Torito-Bootabbild gefunden." >&2
    exit 1
fi
if ! grep -q 'El Torito boot img.*UEFI' <<<"$torito"; then
    echo "Fehler: Kein UEFI-El-Torito-Bootabbild gefunden." >&2
    exit 1
fi

printf 'OK: BIOS- und UEFI-Bootabbild, Kernel, Initramfs und SquashFS vorhanden.\n'
