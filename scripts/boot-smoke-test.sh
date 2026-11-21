#!/usr/bin/env bash
# Playforge – echter Boot-Smoke-Test.
#
# Startet das gebaute ISO in QEMU (q35, TCG) mit OVMF und serieller Konsole.
# Das Live-System schreibt am Ende von live-setup den Marker
# PLAYFORGE_LIVE_READY auf /dev/console bzw. /dev/ttyS0. Erst wenn dieser
# Marker im seriellen Protokoll erscheint, gilt der Test als erfolgreich.
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Nutzung: scripts/boot-smoke-test.sh <iso>

Umgebung:
  PLAYFORGE_BOOT_TIMEOUT   Timeout in Sekunden (Standard: 1200)
  PLAYFORGE_SMOKE_LOG      Zielpfad des seriellen Protokolls
                           (Standard: <ISO-Verzeichnis>/smoke-test.log)
  PLAYFORGE_QEMU_BINARY    QEMU-Binary (Standard: qemu-system-x86_64)
  PLAYFORGE_OVMF_CODE      OVMF-Code-Datei (sonst automatische Suche)
  PLAYFORGE_OVMF_VARS      OVMF-Variablen-Datei (sonst automatische Suche)
EOF
}

die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

case "${1:-}" in
    -h | --help)
        usage
        exit 0
        ;;
esac

iso="${1:-}"
if [[ -z "$iso" || ! -f "$iso" ]]; then
    usage >&2
    exit 2
fi
iso="$(realpath -- "$iso")"

timeout="${PLAYFORGE_BOOT_TIMEOUT:-1200}"
[[ "$timeout" =~ ^[0-9]+$ ]] || die "PLAYFORGE_BOOT_TIMEOUT muss eine Zahl sein: $timeout"
(( timeout > 0 )) || die "PLAYFORGE_BOOT_TIMEOUT muss größer als 0 sein."

qemu_bin="${PLAYFORGE_QEMU_BINARY:-qemu-system-x86_64}"
command -v "$qemu_bin" >/dev/null 2>&1 ||
    die "QEMU fehlt: $qemu_bin (Arch: pacman -S qemu-system-x86)."

find_ovmf() {
    local code vars
    for code in \
        /usr/share/edk2/x64/OVMF_CODE.4m.fd \
        /usr/share/edk2/x64/OVMF_CODE.4m.fds \
        /usr/share/edk2-ovmf/x64/OVMF_CODE.4m.fd \
        /usr/share/edk2-ovmf/x64/OVMF_CODE.fd \
        /usr/share/edk2/ovmf/OVMF_CODE.fd \
        /usr/share/OVMF/OVMF_CODE_4M.fd \
        /usr/share/OVMF/OVMF_CODE.fd; do
        [[ -f "$code" ]] || continue
        vars="${code/OVMF_CODE/OVMF_VARS}"
        [[ -f "$vars" ]] || continue
        printf '%s\n%s\n' "$code" "$vars"
        return 0
    done
    return 1
}

if [[ -n "${PLAYFORGE_OVMF_CODE:-}" || -n "${PLAYFORGE_OVMF_VARS:-}" ]]; then
    [[ -n "${PLAYFORGE_OVMF_CODE:-}" && -n "${PLAYFORGE_OVMF_VARS:-}" ]] ||
        die "PLAYFORGE_OVMF_CODE und PLAYFORGE_OVMF_VARS müssen gemeinsam gesetzt sein."
    [[ -f "$PLAYFORGE_OVMF_CODE" ]] || die "OVMF-Code fehlt: $PLAYFORGE_OVMF_CODE"
    [[ -f "$PLAYFORGE_OVMF_VARS" ]] || die "OVMF-Variablen fehlen: $PLAYFORGE_OVMF_VARS"
    ovmf_code="$PLAYFORGE_OVMF_CODE"
    ovmf_vars="$PLAYFORGE_OVMF_VARS"
else
    ovmf_list="$(find_ovmf)" || die "OVMF nicht gefunden (Arch: pacman -S edk2-ovmf)."
    mapfile -t ovmf <<<"$ovmf_list"
    (( ${#ovmf[@]} >= 2 )) || die "OVMF-Code oder OVMF-Variablen fehlen."
    ovmf_code="${ovmf[0]}"
    ovmf_vars="${ovmf[1]}"
fi

log="${PLAYFORGE_SMOKE_LOG:-$(dirname "$iso")/smoke-test.log}"
mkdir -p -- "$(dirname "$log")"
log="$(realpath -m -- "$log")"
qemu_log="$log.qemu"
: >"$log"
: >"$qemu_log"

tmpdir="$(mktemp -d)"
qemu_pid=""

qemu_alive() {
    if [[ -n "$qemu_pid" && -r "/proc/$qemu_pid/stat" ]]; then
        [[ "$(awk '{print $3}' "/proc/$qemu_pid/stat" 2>/dev/null)" != "Z" ]]
    else
        [[ -n "$qemu_pid" ]] && kill -0 "$qemu_pid" 2>/dev/null
    fi
}

cleanup() {
    local rc=$?
    trap - EXIT
    if qemu_alive; then
        kill "$qemu_pid" 2>/dev/null || true
        wait "$qemu_pid" 2>/dev/null || true
    fi
    rm -rf -- "$tmpdir"
    exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

cp -- "$ovmf_vars" "$tmpdir/OVMF_VARS.fd"
chmod u+w -- "$tmpdir/OVMF_VARS.fd"

printf '==> Boot-Smoke-Test\n'
printf '    ISO:    %s\n' "$iso"
printf '    QEMU:   %s\n' "$qemu_bin"
printf '    OVMF:   %s\n' "$ovmf_code"
printf '    Log:    %s\n' "$log"
printf '    Timeout: %ss\n\n' "$timeout"

qemu_args=(
    -name playforge-boot-smoke
    -machine q35
    -accel tcg,thread=multi
    -cpu max
    -m 4096
    -smp 4
    -global ICH9-LPC.disable_s3=1
    -drive "if=pflash,format=raw,readonly=on,file=$ovmf_code"
    -drive "if=pflash,format=raw,file=$tmpdir/OVMF_VARS.fd"
    # q35 erkennt -cdrom nicht zuverlässig; wie run_archiso einen
    # virtio-SCSI-Controller mit scsi-cd verwenden. Das Arch-Initramfs
    # enthält die nötigen virtio-scsi-Module.
    -device virtio-scsi-pci,id=scsi0
    -drive "id=cdrom0,if=none,format=raw,media=cdrom,read-only=on,file=$iso"
    -device "scsi-cd,bus=scsi0.0,drive=cdrom0,bootindex=1"
    -boot order=d,menu=off,reboot-timeout=5000
    -nic none
    -display none
    -serial "file:$log"
    -monitor none
    -no-reboot
)

"$qemu_bin" "${qemu_args[@]}" >"$qemu_log" 2>&1 &
qemu_pid=$!

deadline=$(( SECONDS + timeout ))
ready=0
failure=""

while (( SECONDS < deadline )); do
    if grep -qF 'PLAYFORGE_LIVE_READY' "$log" 2>/dev/null; then
        ready=1
        break
    fi

    if grep -Eiq \
        'Kernel panic|No working init found|unable to mount root fs|Attempted to kill init|No bootable device|Boot failed|BdsDxe: failed to load|Failed to load image' \
        "$log" 2>/dev/null; then
        failure="Das Gast-System meldet einen Bootfehler."
        break
    fi

    if ! qemu_alive; then
        qemu_rc=0
        wait "$qemu_pid" || qemu_rc=$?
        qemu_pid=""
        failure="QEMU wurde beendet (Exit $qemu_rc), bevor PLAYFORGE_LIVE_READY erschien."
        break
    fi

    sleep 5
done

if (( ready == 0 )) && [[ -z "$failure" ]]; then
    failure="Timeout nach ${timeout}s: PLAYFORGE_LIVE_READY wurde nicht gesehen."
fi

if (( ready )); then
    printf 'OK: Live-System hat den Bereitschafts-Marker erreicht.\n'
    printf 'Serielles Protokoll: %s\n\n' "$log"
    tail -n 60 -- "$log" || true
    exit 0
fi

printf '\nFEHLER: Boot-Smoke-Test fehlgeschlagen: %s\n' "$failure" >&2
printf 'Serielles Protokoll: %s\n\n' "$log" >&2
printf 'Letzte serielle Ausgabe:\n' >&2
tail -n 120 -- "$log" >&2 || true

if [[ -s "$qemu_log" ]]; then
    printf '\nQEMU-Ausgabe:\n' >&2
    tail -n 80 -- "$qemu_log" >&2 || true
fi

exit 1
