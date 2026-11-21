#!/usr/bin/env bash
# Playforge – richtet die systemweiten Theme-Komponenten ein.
# Wird vom Live-Setup und vom Installer im Zielsystem aufgerufen.
set -Eeuo pipefail

info() { printf '  %s\n' "$*"; }
warn() { printf 'Warnung: %s\n' "$*" >&2; }

setup_plasma_style() {
    local custom=/usr/share/plasma/desktoptheme/Playforge
    local base=""

    for candidate in \
        /usr/share/plasma/desktoptheme/breeze-dark \
        /usr/share/plasma/desktoptheme/breeze \
        /usr/share/plasma/desktoptheme/default; do
        if [[ -d "$candidate" ]]; then
            base="$candidate"
            break
        fi
    done

    if [[ -z "$base" ]]; then
        warn "Kein Plasma-Desktop-Theme gefunden – Plasma nutzt den Standardstil."
        return 0
    fi

    rm -rf "$custom"
    cp -a "$base" "$custom"
    rm -f "$custom/metadata.desktop"
    cat >"$custom/metadata.json" <<'EOF'
{
  "KPlugin": {
    "Authors": [
      {
        "Name": "Playforge"
      }
    ],
    "Description": "Playforge Plasma-Stil auf Basis von Breeze Dark.",
    "Description[de]": "Playforge Plasma-Stil auf Basis von Breeze Dark.",
    "Description[en]": "Playforge Plasma style based on Breeze Dark.",
    "EnabledByDefault": true,
    "Id": "Playforge",
    "License": "GPL-2.0-or-later",
    "Name": "Playforge",
    "Name[de]": "Playforge",
    "Name[en]": "Playforge",
    "Version": "1.0",
    "Website": "https://github.com/INDDYtran/playforge-iso-build"
  },
  "X-Plasma-API": "5.0"
}
EOF
    if [[ -f /usr/share/color-schemes/Playforge.colors ]]; then
        cp -f /usr/share/color-schemes/Playforge.colors "$custom/colors"
    fi

    install -d -m 0755 /etc/xdg
    printf '[Theme]\nname=Playforge\n' >/etc/xdg/plasmarc
    info "Plasma-Stil Playforge eingerichtet (Basis: $(basename "$base"))."
}

setup_sddm_theme() {
    local base=/usr/share/sddm/themes/breeze
    local theme=/usr/share/sddm/themes/playforge
    local wallpaper=/usr/share/wallpapers/Playforge/contents/images/2560x1440.png
    local logo=/usr/share/pixmaps/playforge-logo.svg

    if [[ ! -d "$base" ]]; then
        warn "Breeze-SDDM-Theme nicht gefunden – SDDM bleibt beim Standarddesign."
        return 0
    fi

    rm -rf "$theme"
    cp -a "$base" "$theme"
    if [[ -f "$theme/metadata.desktop" ]]; then
        sed -i 's/^Name=.*/Name=Playforge/' "$theme/metadata.desktop"
    fi
    cat >"$theme/theme.conf" <<EOF
[General]
showlogo=shown
showClock=true
logo=$logo
type=image
color=#0B0F1A
fontSize=10
background=$wallpaper
needsFullUserModel=false
EOF

    install -d -m 0755 /etc/sddm.conf.d
    cat >/etc/sddm.conf.d/20-playforge-theme.conf <<'EOF'
[Theme]
Current=playforge
EOF
    info "SDDM-Theme eingerichtet."
}

setup_plasma_style
setup_sddm_theme
