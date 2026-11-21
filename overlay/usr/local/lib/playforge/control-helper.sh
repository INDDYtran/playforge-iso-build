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
    *)
        usage
        exit 2
        ;;
esac
