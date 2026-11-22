#!/usr/bin/env bash
# Playforge – Installationstest im Live-System.
#
# Bootet das ISO per QEMU (direkt über Kernel/Initramfs) mit einer leeren
# 40-GiB-Platte und startet die Test-Unit im Live-System. Diese ruft den
# echten Installer auf und prüft danach im Zielsystem die automatische
# Aktivierung der signierten Playforge-Paketquelle.
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Nutzung: scripts/install-smoke-test.sh <iso>

Umgebung:
  PLAYFORGE_INSTALL_TIMEOUT  Timeout in Sekunden (Standard: 3600)
  PLAYFORGE_INSTALL_LOG      Zielpfad des seriellen Protokolls
                             (Standard: <ISO-Verzeichnis>/install-test.log)
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
[[ -n "$iso" && -f "$iso" ]] || {
    usage >&2
    exit 2
}
iso="$(realpath -- "$iso")"

timeout="${PLAYFORGE_INSTALL_TIMEOUT:-3600}"
[[ "$timeout" =~ ^[0-9]+$ ]] || die "PLAYFORGE_INSTALL_TIMEOUT muss eine Zahl sein."
(( timeout > 0 )) || die "PLAYFORGE_INSTALL_TIMEOUT muss größer als 0 sein."

for cmd in qemu-system-x86_64 bsdtar truncate; do
    command -v "$cmd" >/dev/null 2>&1 || die "$cmd fehlt."
done

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

ovmf_list="$(find_ovmf)" || die "OVMF nicht gefunden."
mapfile -t ovmf <<<"$ovmf_list"
(( ${#ovmf[@]} >= 2 )) || die "OVMF-Code oder -Variablen fehlen."

log="${PLAYFORGE_INSTALL_LOG:-$(dirname "$iso")/install-test.log}"
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

cp -- "${ovmf[1]}" "$tmpdir/OVMF_VARS.fd"
chmod u+w -- "$tmpdir/OVMF_VARS.fd"

printf '==> Installationstest\n'
printf '    ISO:     %s\n' "$iso"
printf '    Log:     %s\n' "$log"
printf '    Timeout: %ss\n\n' "$timeout"

bsdtar -xOf "$iso" playforge/boot/x86_64/vmlinuz-linux >"$tmpdir/vmlinuz" ||
    die "Kernel konnte nicht aus dem ISO gelesen werden."
bsdtar -xOf "$iso" playforge/boot/x86_64/initramfs-linux.img >"$tmpdir/initramfs" ||
    die "Initramfs konnte nicht aus dem ISO gelesen werden."
# Sparse Raw-Platte: qemu-img ist im Builder-Container nicht installiert,
# eine 40-GiB-Raw-Datei belegt ohne Schreibzugriffe praktisch keinen Platz.
truncate -s 40G "$tmpdir/disk.raw" ||
    die "Virtuelle Platte konnte nicht angelegt werden."

qemu_args=(
    -name playforge-install-smoke
    -machine q35
    -accel tcg,thread=multi
    -cpu max
    -m 6144
    -smp 4
    -global ICH9-LPC.disable_s3=1
    -drive "if=pflash,format=raw,readonly=on,file=${ovmf[0]}"
    -drive "if=pflash,format=raw,file=$tmpdir/OVMF_VARS.fd"
    -kernel "$tmpdir/vmlinuz"
    -initrd "$tmpdir/initramfs"
    -append "archisobasedir=playforge archisodevice=/dev/sr0 console=ttyS0,115200n8 systemd.unit=multi-user.target systemd.wants=playforge-install-test.service playforge.install_test=1"
    -device virtio-scsi-pci,id=scsi0
    -drive "id=cdrom0,if=none,format=raw,media=cdrom,read-only=on,file=$iso"
    -device "scsi-cd,bus=scsi0.0,drive=cdrom0"
    -drive "file=$tmpdir/disk.raw,if=virtio,format=raw"
    -nic "user,model=virtio-net-pci"
    -display none
    -serial "file:$log"
    -monitor none
    -no-reboot
)

qemu-system-x86_64 "${qemu_args[@]}" >"$qemu_log" 2>&1 &
qemu_pid=$!

deadline=$((SECONDS + timeout))
result=""
while ((SECONDS < deadline)); do
    if grep -qF 'PLAYFORGE_INSTALL_TEST_OK' "$log" 2>/dev/null; then
        result="ok"
        break
    fi
    if grep -qF 'PLAYFORGE_INSTALL_TEST_FAILED' "$log" 2>/dev/null; then
        result="failed"
        break
    fi
    if ! qemu_alive; then
        qemu_rc=0
        wait "$qemu_pid" || qemu_rc=$?
        qemu_pid=""
        result="qemu-exit"
        printf 'QEMU beendet (Exit %s), bevor ein Testmarker erschien.\n' "$qemu_rc" >&2
        break
    fi
    sleep 10
done

if [[ "$result" == "ok" ]]; then
    printf 'OK: Installation und automatische Paketquellen-Aktivierung bestanden.\n'
    printf 'Serielles Protokoll: %s\n\n' "$log"
    tail -n 80 -- "$log" || true
    exit 0
fi

if [[ -z "$result" ]]; then
    printf 'FEHLER: Timeout nach %ss ohne Testmarker.\n' "$timeout" >&2
else
    printf 'FEHLER: Installationstest fehlgeschlagen (%s).\n' "$result" >&2
fi
printf 'Serielles Protokoll: %s\n\n' "$log" >&2
tail -n 200 -- "$log" >&2 || true
if [[ -s "$qemu_log" ]]; then
    printf '\nQEMU-Ausgabe:\n' >&2
    tail -n 80 -- "$qemu_log" >&2 || true
fi
exit 1
