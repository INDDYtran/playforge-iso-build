#!/usr/bin/env bash
# Playforge – privilegierter Helfer des Control Centers.
#
# Das Control Center schreibt selbst nie nach /etc: Es ruft dieses Skript über
# pkexec auf (auf dem Live-Medium über das passwortlose sudo des Benutzers
# gamer). Jeder Befehl prüft Anzahl und Wertebereich der Argumente; Dienste und
# Profile stammen aus festen Listen, freie Pfade werden nie übernommen.
set -Eeuo pipefail

die() { printf 'Fehler: %s\n' "$*" >&2; exit 2; }

gamemode_conf=/etc/gamemode.ini
environment_conf=/etc/environment.d/90-playforge.conf
skel_mangohud=/etc/skel/.config/MangoHud/MangoHud.conf

usage() {
    cat <<'EOF'
Nutzung: control-helper.sh <Befehl> [Argumente]

Befehle:
  profile <performance|balanced|reset>
      Leistungsprofil über playforge-tweaks setzen
  shader-cache <4G|8G|12G|16G|32G>
      MESA_SHADER_CACHE_MAX_SIZE in /etc/environment.d setzen
  gamemode <key=value> [...]
      /etc/gamemode.ini anpassen (desiredgov, renice, softrealtime)
  mangohud <standard|compact|detailed|off>
      MangoHud-Profil für neue Konten und den aufrufenden Benutzer schreiben
  service <start|stop|restart|enable|disable> <unit>
      einen der freigegebenen systemd-Dienste schalten
  cpu <governor> <epp>
      CPU-Governor und Energy-Performance-Preference setzen und beim
      Booten über playforge-cpu-tune.service wiederherstellen
  sysctl <key=value> [...]
      vm.swappiness, vm.vfs_cache_pressure, vm.max_map_count und
      fs.inotify.max_user_watches dauerhaft setzen
  io-scheduler <none|mq-deadline|kyber|bfq>
      Datenträger-Scheduler per udev-Regel setzen
  graphics-env <KEY=WERT> [...]
      MESA_GLTHREAD, AMD_VULKAN_ICD und RADV_PERFTEST setzen
  updates apply [--flatpak] [--mirrors]
      pacman, optional Flatpak und die Paketspiegel aktualisieren
  autoupdate <off|daily|weekly>
      systemd-Timer für automatische Updates einrichten

Der Helfer gehört zum Playforge Control Center und wird mit
Administratorrechten aufgerufen.
EOF
}

# Setzt einen Schlüssel in einer INI-Sektion und lässt den Rest der Datei
# unverändert. Vorhandene Einträge werden ersetzt, fehlende ergänzt.
set_ini_value() {
    local file=$1 section=$2 key=$3 value=$4 tmp
    [[ -f "$file" ]] || die "$file ist nicht vorhanden."
    tmp="$(mktemp)"
    awk -v section="$section" -v key="$key" -v value="$value" '
        BEGIN { in_section = 0; written = 0 }
        /^[[:space:]]*\[/ {
            if (in_section && !written) { print key " = " value; written = 1 }
            line = $0
            gsub(/^[[:space:]]+/, "", line)
            gsub(/[[:space:]]+$/, "", line)
            in_section = (line == "[" section "]")
            print
            next
        }
        in_section && $0 ~ "^[[:space:]]*" key "[[:space:]]*=" {
            if (!written) { print key " = " value; written = 1 }
            next
        }
        { print }
        END { if (in_section && !written) print key " = " value }
    ' "$file" >"$tmp"
    cat "$tmp" >"$file"
    rm -f "$tmp"
}

# Setzt oder entfernt eine KEY=VALUE-Zeile in einer flachen Datei.
set_conf_value() {
    local file=$1 key=$2 value=$3 tmp
    install -d -m 0755 "$(dirname "$file")"
    touch "$file"
    tmp="$(mktemp)"
    if [[ -n "$value" ]]; then
        awk -v key="$key" -v value="$value" '
            BEGIN { done = 0 }
            index($0, key "=") == 1 { if (!done) { print key "=" value; done = 1 } next }
            { print }
            END { if (!done) print key "=" value }
        ' "$file" >"$tmp"
    else
        awk -v key="$key" 'index($0, key "=") == 1 { next } { print }' "$file" >"$tmp"
    fi
    cat "$tmp" >"$file"
    rm -f "$tmp"
}

# Schreibt Governor und EPP in alle Cpufreq-Policies.
apply_cpu() {
    local governor=$1 epp=$2 changed=0 path
    if [[ -n "$governor" ]]; then
        for path in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            [[ -f "$path" ]] || continue
            if printf '%s' "$governor" >"$path" 2>/dev/null; then
                changed=$((changed + 1))
            fi
        done
    fi
    if [[ -n "$epp" ]]; then
        for path in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do
            [[ -f "$path" ]] || continue
            printf '%s' "$epp" >"$path" 2>/dev/null || true
        done
    fi
    printf 'CPU: Governor %s, EPP %s gesetzt (%s Policies).\n' \
        "${governor:-—}" "${epp:-—}" "$changed"
}

write_autoupdate_units() {
    local calendar=$1
    cat >/etc/systemd/system/playforge-autoupdate.service <<'EOF'
[Unit]
Description=Playforge – automatische Systemupdates
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
Nice=10
ExecStart=/usr/local/lib/playforge/control-helper.sh updates apply --flatpak
EOF
    cat >/etc/systemd/system/playforge-autoupdate.timer <<EOF
[Unit]
Description=Playforge – automatische Systemupdates (${calendar})

[Timer]
OnCalendar=${calendar}
Persistent=true
RandomizedDelaySec=30min

[Install]
WantedBy=timers.target
EOF
    chmod 0644 /etc/systemd/system/playforge-autoupdate.service \
        /etc/systemd/system/playforge-autoupdate.timer
    systemctl daemon-reload
}

# Muss inhaltlich zu ui::gaming::preset_conf im Control Center passen.
write_preset() {
    case "$1" in
        standard)
            cat <<'EOF'
# Playforge-Preset: standard
gpu_stats
cpu_stats
ram
vram
fps
frametime
frame_timing
gpu_temp
cpu_temp
position=top-left
font_size=18
background_alpha=0.35
round_corners=8
EOF
            ;;
        compact)
            cat <<'EOF'
# Playforge-Preset: compact
gpu_stats
cpu_stats
fps
frametime
position=top-right
font_size=14
background_alpha=0.25
round_corners=6
EOF
            ;;
        detailed)
            cat <<'EOF'
# Playforge-Preset: detailed
gpu_stats
gpu_temp
gpu_power
gpu_clock
cpu_stats
cpu_temp
cpu_power
cpu_mhz
ram
vram
fps
frametime
frame_timing
position=top-left
font_size=16
background_alpha=0.4
round_corners=8
toggle_hud=Shift_R+F12
EOF
            ;;
        off)
            cat <<'EOF'
# Playforge-Preset: off
no_display
EOF
            ;;
        *)
            die "Unbekanntes MangoHud-Profil: $1"
            ;;
    esac
}

command="${1:-}"
if [[ $# -gt 0 ]]; then
    shift
fi

if [[ "$command" == "" || "$command" == "-h" || "$command" == "--help" || "$command" == "help" ]]; then
    usage
    exit 0
fi

if [[ $EUID -ne 0 ]]; then
    die "Dieser Helfer benötigt root-Rechte (Aufruf über das Control Center)."
fi

case "$command" in
    profile)
        [[ $# -eq 1 ]] || die "profile <performance|balanced|reset>"
        case "$1" in
            performance | balanced | reset) ;;
            *) die "Unbekanntes Profil: $1" ;;
        esac
        exec /usr/local/bin/playforge-tweaks "$1"
        ;;
    shader-cache)
        [[ $# -eq 1 ]] || die "shader-cache <4G|8G|12G|16G|32G>"
        [[ "$1" =~ ^(4|8|12|16|32)G$ ]] || die "Unzulässige Größe: $1"
        install -d -m 0755 /etc/environment.d
        touch "$environment_conf"
        if grep -q '^MESA_SHADER_CACHE_MAX_SIZE=' "$environment_conf"; then
            sed -i "s|^MESA_SHADER_CACHE_MAX_SIZE=.*|MESA_SHADER_CACHE_MAX_SIZE=$1|" \
                "$environment_conf"
        else
            printf 'MESA_SHADER_CACHE_MAX_SIZE=%s\n' "$1" >>"$environment_conf"
        fi
        printf 'Shader-Cache-Limit auf %s gesetzt (wirkt nach der nächsten Anmeldung).\n' "$1"
        ;;
    gamemode)
        [[ $# -ge 1 ]] || die "gamemode <key=value> [...]"
        for pair in "$@"; do
            case "$pair" in
                desiredgov=performance | desiredgov=powersave) ;;
                softrealtime=auto | softrealtime=on | softrealtime=off) ;;
                inhibit_screensaver=0 | inhibit_screensaver=1) ;;
                reaper_freq=*)
                    value="${pair#reaper_freq=}"
                    [[ "$value" =~ ^[0-9]+$ ]] && ((value <= 120)) ||
                        die "reaper_freq muss zwischen 0 und 120 liegen."
                    ;;
                renice=*)
                    value="${pair#renice=}"
                    [[ "$value" =~ ^([0-9]|1[0-9])$ ]] ||
                        die "renice muss zwischen 0 und 19 liegen."
                    ;;
                *) die "Unbekannte GameMode-Einstellung: $pair" ;;
            esac
        done
        for pair in "$@"; do
            set_ini_value "$gamemode_conf" general "${pair%%=*}" "${pair#*=}"
        done
        printf 'GameMode-Konfiguration aktualisiert: %s\n' "$*"
        ;;
    mangohud)
        [[ $# -eq 1 ]] || die "mangohud <standard|compact|detailed|off>"
        preset="$1"
        install -d -m 0755 "$(dirname "$skel_mangohud")"
        tmp_conf="$(mktemp)"
        write_preset "$preset" >"$tmp_conf"
        install -m 0644 "$tmp_conf" "$skel_mangohud"
        rm -f "$tmp_conf"

        # Aufrufender Benutzer: pkexec setzt PKEXEC_UID, sudo setzt SUDO_UID.
        uid="${PKEXEC_UID:-${SUDO_UID:-}}"
        if [[ -n "$uid" && "$uid" =~ ^[0-9]+$ ]]; then
            entry="$(getent passwd "$uid" || true)"
            home="$(printf '%s' "$entry" | cut -d: -f6)"
            gid="$(printf '%s' "$entry" | cut -d: -f4)"
            if [[ -n "$home" && -d "$home" ]]; then
                install -d -m 0755 -o "$uid" -g "$gid" "$home/.config/MangoHud"
                install -m 0644 -o "$uid" -g "$gid" "$skel_mangohud" \
                    "$home/.config/MangoHud/MangoHud.conf"
            fi
        fi
        printf 'MangoHud-Profil "%s" geschrieben.\n' "$preset"
        ;;
    service)
        [[ $# -eq 2 ]] || die "service <start|stop|restart|enable|disable> <unit>"
        action="$1"
        unit="$2"
        case "$action" in
            start | stop | restart | enable | disable) ;;
            *) die "Unbekannte Aktion: $action" ;;
        esac
        case "$unit" in
            playforge-profile.service | NetworkManager.service | bluetooth.service | \
                power-profiles-daemon.service | lactd.service | sddm.service | \
                fstrim.timer | reflector.timer | systemd-timesyncd.service) ;;
            *) die "Nicht freigegebene Unit: $unit" ;;
        esac
        if [[ "$unit" == "sddm.service" && ( "$action" == "stop" || "$action" == "disable" ) ]]; then
            die "Der Anmeldebildschirm lässt sich hier nicht abschalten – sonst ist keine Anmeldung mehr möglich."
        fi
        if ! systemctl "$action" "$unit"; then
            die "systemctl $action $unit ist fehlgeschlagen."
        fi
        printf '%s: %s ausgeführt.\n' "$unit" "$action"
        ;;
    cpu)
        [[ $# -eq 2 ]] || die "cpu <governor> <epp>"
        case "$1" in
            performance | powersave | schedutil | ondemand | conservative) ;;
            *) die "Unbekannter Governor: $1" ;;
        esac
        case "$2" in
            default | performance | balance_performance | balance_power | power) ;;
            *) die "Unbekannte Energy-Performance-Preference: $2" ;;
        esac
        install -d -m 0755 /etc/playforge
        printf 'GOVERNOR=%s\nEPP=%s\n' "$1" "$2" >/etc/playforge/cpu.conf
        cat >/etc/systemd/system/playforge-cpu-tune.service <<'EOF'
[Unit]
Description=Playforge CPU-Tuning (Governor und EPP)
After=multi-user.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/local/lib/playforge/control-helper.sh cpu-restore

[Install]
WantedBy=multi-user.target
EOF
        chmod 0644 /etc/systemd/system/playforge-cpu-tune.service
        systemctl daemon-reload
        systemctl enable playforge-cpu-tune.service >/dev/null 2>&1 || true
        apply_cpu "$1" "$2"
        ;;
    cpu-restore)
        [[ -f /etc/playforge/cpu.conf ]] || exit 0
        # shellcheck disable=SC1091
        . /etc/playforge/cpu.conf
        apply_cpu "${GOVERNOR:-}" "${EPP:-}"
        ;;
    sysctl)
        [[ $# -ge 1 ]] || die "sysctl <key=value> [...]"
        conf=/etc/sysctl.d/99-playforge-zz-tuning.conf
        install -d -m 0755 /etc/sysctl.d
        touch "$conf"
        for pair in "$@"; do
            [[ "$pair" == *=* ]] || die "Ungültiges Paar: $pair"
            key="${pair%%=*}"
            value="${pair#*=}"
            [[ "$value" =~ ^[0-9]+$ ]] || die "Wert für $key muss eine Zahl sein."
            case "$key" in
                vm.swappiness)
                    ((value <= 200)) || die "vm.swappiness liegt zwischen 0 und 200."
                    ;;
                vm.vfs_cache_pressure)
                    ((value <= 1000)) || die "vm.vfs_cache_pressure liegt zwischen 0 und 1000."
                    ;;
                vm.max_map_count)
                    ((value >= 65530 && value <= 2147483642)) ||
                        die "vm.max_map_count liegt zwischen 65530 und 2147483642."
                    ;;
                fs.inotify.max_user_watches)
                    ((value >= 8192 && value <= 1048576)) ||
                        die "fs.inotify.max_user_watches liegt zwischen 8192 und 1048576."
                    ;;
                *) die "Nicht freigegebener sysctl-Schlüssel: $key" ;;
            esac
        done
        for pair in "$@"; do
            set_conf_value "$conf" "${pair%%=*}" "${pair#*=}"
            sysctl -w "$pair" >/dev/null
        done
        printf 'sysctl-Werte gesetzt (%s).\n' "$conf"
        ;;
    io-scheduler)
        [[ $# -eq 1 ]] || die "io-scheduler <none|mq-deadline|kyber|bfq>"
        case "$1" in
            none | mq-deadline | kyber | bfq) ;;
            *) die "Unbekannter Scheduler: $1" ;;
        esac
        install -d -m 0755 /etc/udev/rules.d
        cat >/etc/udev/rules.d/60-playforge-io-scheduler.rules <<EOF
# Playforge Control Center – Scheduler für alle Datenträger
ACTION=="add|change", SUBSYSTEM=="block", KERNEL=="sd[a-z]|nvme[0-9]*n[0-9]*|mmcblk[0-9]*|vd[a-z]", ATTR{queue/scheduler}="$1"
EOF
        chmod 0644 /etc/udev/rules.d/60-playforge-io-scheduler.rules
        for path in /sys/block/*/queue/scheduler; do
            [[ -w "$path" ]] || continue
            if grep -q "$1" "$path" 2>/dev/null; then
                printf '%s' "$1" >"$path" 2>/dev/null || true
            fi
        done
        udevadm control --reload-rules >/dev/null 2>&1 || true
        printf 'Scheduler %s gesetzt.\n' "$1"
        ;;
    graphics-env)
        [[ $# -ge 1 ]] || die "graphics-env <KEY=WERT> [...]"
        install -d -m 0755 /etc/environment.d
        touch "$environment_conf"
        for pair in "$@"; do
            [[ "$pair" == *=* ]] || die "Ungültiges Paar: $pair"
            key="${pair%%=*}"
            value="${pair#*=}"
            case "$key" in
                MESA_GLTHREAD)
                    [[ "$value" =~ ^(true|false)$ ]] || die "MESA_GLTHREAD muss true oder false sein."
                    ;;
                AMD_VULKAN_ICD)
                    [[ "$value" =~ ^(RADV|AMDVLK)$ ]] || die "AMD_VULKAN_ICD muss RADV oder AMDVLK sein."
                    ;;
                RADV_PERFTEST)
                    [[ "$value" =~ ^(gpl)?$ ]] || die "RADV_PERFTEST erlaubt nur gpl oder leer."
                    ;;
                *) die "Nicht freigegebene Variable: $key" ;;
            esac
        done
        for pair in "$@"; do
            set_conf_value "$environment_conf" "${pair%%=*}" "${pair#*=}"
        done
        printf 'Grafik-Umgebung aktualisiert; wirkt nach der nächsten Anmeldung.\n'
        ;;
    updates)
        flatpak_flag=0
        mirrors_flag=0
        for arg in "$@"; do
            case "$arg" in
                apply) ;;
                --flatpak) flatpak_flag=1 ;;
                --mirrors) mirrors_flag=1 ;;
                *) die "Unbekannte Option: $arg" ;;
            esac
        done
        install -d -m 0755 /var/lib/playforge
        printf '== Playforge: Systemupdate %s ==\n' "$(date -Is)"
        if ((mirrors_flag)) && command -v reflector >/dev/null 2>&1; then
            printf '\n== Paketspiegel (reflector) ==\n'
            reflector --latest 20 --protocol https --sort rate --save \
                /etc/pacman.d/mirrorlist ||
                printf 'reflector meldete einen Fehler – Spiegel bleiben unverändert.\n'
        fi
        printf '\n== pacman ==\n'
        pacman -Syu --noconfirm --needed
        if ((flatpak_flag)) && command -v flatpak >/dev/null 2>&1; then
            printf '\n== Flatpak (System) ==\n'
            flatpak update -y --noninteractive || printf 'Flatpak meldete einen Fehler.\n'
            uid="${PKEXEC_UID:-${SUDO_UID:-}}"
            if [[ -n "$uid" && "$uid" =~ ^[0-9]+$ ]]; then
                user="$(getent passwd "$uid" | cut -d: -f1)"
                home="$(getent passwd "$uid" | cut -d: -f6)"
                if [[ -n "$user" && -n "$home" ]]; then
                    printf '\n== Flatpak (%s) ==\n' "$user"
                    runuser -u "$user" -- env HOME="$home" flatpak --user update -y \
                        --noninteractive || printf 'Flatpak (Benutzer) meldete einen Fehler.\n'
                fi
            fi
        fi
        date +%s >/var/lib/playforge/last-update
        printf '\nFertig: %s\n' "$(date -Is)"
        ;;
    autoupdate)
        [[ $# -eq 1 ]] || die "autoupdate <off|daily|weekly>"
        install -d -m 0755 /etc/playforge
        case "$1" in
            off)
                printf 'MODE=off\n' >/etc/playforge/updates.conf
                systemctl disable --now playforge-autoupdate.timer >/dev/null 2>&1 || true
                printf 'Automatische Updates deaktiviert.\n'
                ;;
            daily | weekly)
                printf 'MODE=%s\n' "$1" >/etc/playforge/updates.conf
                if [[ "$1" == daily ]]; then
                    write_autoupdate_units daily
                else
                    write_autoupdate_units weekly
                fi
                systemctl enable --now playforge-autoupdate.timer
                printf 'Automatische Updates: %s.\n' "$1"
                ;;
            *) die "Unbekannter Modus: $1" ;;
        esac
        ;;
    *)
        usage
        exit 2
        ;;
esac
