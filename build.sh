#!/usr/bin/env bash
# build.sh – baut das Playforge-ISO aus dem offiziellen archiso-Releng-Profil.
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
work_dir="$repo_dir/work"
out_dir="$repo_dir/out"
keep_work=0
skip_check=0

msg() { printf '\n==> %s\n' "$*"; }
warn() { printf 'Warnung: %s\n' "$*" >&2; }
die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Nutzung: bash build.sh [Optionen]

Baut out/playforge-<datum>-x86_64.iso mit mkarchiso.

Optionen:
  --keep-work   Arbeitsverzeichnis (work/) nach dem Build behalten
  --no-check    Paketnamen nicht vorab gegen die Repositories prüfen
  -h, --help    Diese Hilfe anzeigen

Voraussetzungen:
  * Arch Linux (x86_64) mit installiertem Paket "archiso"
  * aktiviertes [multilib]-Repository (für steam und lib32-*)
  * ca. 25 GB freier Speicher, Internetzugang
EOF
}

while (( $# > 0 )); do
    case "$1" in
        --keep-work) keep_work=1 ;;
        --no-check) skip_check=1 ;;
        -h | --help)
            usage
            exit 0
            ;;
        *) die "Unbekannte Option: $1" ;;
    esac
    shift
done

msg "Prüfe Build-Umgebung"
[[ -f /etc/arch-release ]] || die "Dieses Skript muss auf Arch Linux laufen."
for cmd in mkarchiso pacman awk sed find; do
    command -v "$cmd" >/dev/null 2>&1 || die "Befehl '$cmd' fehlt."
done
[[ -d /usr/share/archiso/configs/releng ]] || die "Paket 'archiso' installieren: pacman -S --needed archiso"
[[ -f "$repo_dir/overlay/usr/local/share/playforge/packages.target" ]] ||
    die "Paketliste overlay/usr/local/share/playforge/packages.target fehlt."
if (( EUID == 0 )); then
    warn "Build als root – mkarchiso warnt davor. Besser als normaler Benutzer starten."
fi

msg "Bereite Arbeitsverzeichnis vor"
# Immer frisch starten: mkarchiso würde sonst alte run_once-Marker wiederverwenden.
rm -rf -- "$work_dir"
mkdir -p -- "$work_dir" "$out_dir"
profile_dir="$work_dir/profile"
rm -rf -- "$profile_dir"
mkdir -p -- "$profile_dir"

# Basisprofil des installierten archiso kopieren, dann Playforge-Overlay darüber.
cp -a /usr/share/archiso/configs/releng/. "$profile_dir/"
cp -a "$repo_dir/profile/." "$profile_dir/"

# Gemeinsame Root-Dateien (Live-System und Installation) mit auf das Medium nehmen.
install -d "$profile_dir/airootfs/usr/local/share/playforge/overlay"
cp -a "$repo_dir/overlay/." "$profile_dir/airootfs/usr/local/share/playforge/overlay/"
install -m 0644 "$repo_dir/overlay/etc/pacman.conf" "$profile_dir/pacman.conf"

msg "Erzeuge Paketliste"
{
    printf '# Automatisch erzeugt von build.sh – nicht bearbeiten.\n'
    cat /usr/share/archiso/configs/releng/packages.x86_64
    sed -e 's/#.*//' -e 's/[[:space:]]//g' "$repo_dir/overlay/usr/local/share/playforge/packages.target" | awk 'NF'
} | awk 'NF && $1 !~ /^#/ { if (!seen[$1]++) print $1 }' >"$profile_dir/packages.x86_64"

if (( ! skip_check )); then
    msg "Prüfe Paketnamen gegen die Repositories"
    missing=()
    while IFS= read -r pkg; do
        pacman -Si --config "$profile_dir/pacman.conf" "$pkg" >/dev/null 2>&1 || missing+=("$pkg")
    done < <(grep -v '^#' "$profile_dir/packages.x86_64")
    if (( ${#missing[@]} > 0 )); then
        printf 'Nicht gefunden: %s\n' "${missing[*]}" >&2
        die "Paketnamen prüfen oder mit --no-check überspringen (multilib aktiviert und 'pacman -Sy' ausgeführt?)."
    fi
fi

msg "Passe Bootmenü an"
while IFS= read -r -d '' file; do
    sed -i 's/Arch Linux/Playforge/g' "$file"
done < <(
    # Nur Textkonfigurationen bearbeiten: syslinux/splash.png und andere
    # Binärdateien im Profil dürfen nicht durch sed verändert werden.
    for dir in efiboot syslinux grub; do
        [[ -d "$profile_dir/$dir" ]] || continue
        find "$profile_dir/$dir" -type f \( -name '*.conf' -o -name '*.cfg' \) -print0
    done
)

msg "Konfiguriere Live-Dienste"
systemd_dir="$profile_dir/airootfs/etc/systemd/system"
mkdir -p -- "$systemd_dir/multi-user.target.wants" "$systemd_dir/timers.target.wants"

# releng nutzt systemd-networkd + iwd + sshd; im Live-Desktop übernimmt NetworkManager.
rm -f -- \
    "$systemd_dir/multi-user.target.wants/systemd-networkd.service" \
    "$systemd_dir/multi-user.target.wants/iwd.service" \
    "$systemd_dir/multi-user.target.wants/sshd.service" \
    "$systemd_dir/network-online.target.wants/systemd-networkd-wait-online.service" \
    "$systemd_dir/sockets.target.wants/systemd-networkd.socket" \
    "$systemd_dir/getty@tty1.service.d/autologin.conf"

ln -sfn /etc/systemd/system/playforge-live-setup.service \
    "$systemd_dir/multi-user.target.wants/playforge-live-setup.service"
ln -sfn /usr/lib/systemd/system/NetworkManager.service \
    "$systemd_dir/multi-user.target.wants/NetworkManager.service"
ln -sfn /usr/lib/systemd/system/bluetooth.service \
    "$systemd_dir/multi-user.target.wants/bluetooth.service"
ln -sfn /usr/lib/systemd/system/power-profiles-daemon.service \
    "$systemd_dir/multi-user.target.wants/power-profiles-daemon.service"
ln -sfn /usr/lib/systemd/system/fstrim.timer \
    "$systemd_dir/timers.target.wants/fstrim.timer"
ln -sfn /usr/lib/systemd/system/sddm.service \
    "$systemd_dir/display-manager.service"

msg "Starte mkarchiso (das dauert je nach Hardware 20–60 Minuten)"
mkarchiso -v -w "$work_dir" -o "$out_dir" "$profile_dir"

iso="$(find "$out_dir" -maxdepth 1 -type f -name 'playforge-*.iso' -printf '%T@ %p\n' | sort -n | tail -n1 | cut -d' ' -f2-)"
[[ -n "$iso" && -f "$iso" ]] || die "Kein ISO in $out_dir gefunden."
(
    cd "$out_dir"
    sha256sum "$(basename "$iso")" >"$(basename "$iso").sha256"
)

msg "Fertig"
printf 'ISO:    %s\n' "$iso"
printf 'Größe:  %s\n' "$(du -h "$iso" | cut -f1)"
printf 'SHA256: %s\n' "$(cut -d' ' -f1 "$out_dir/$(basename "$iso").sha256")"

if (( keep_work )); then
    msg "Arbeitsverzeichnis behalten: $work_dir"
else
    rm -rf -- "$work_dir"
fi
