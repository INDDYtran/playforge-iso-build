#!/usr/bin/env bash
# Playforge – wendet das Theme in der laufenden Plasma-Sitzung an.
# Wird beim ersten Login automatisch gestartet; mit --force manuell erneut.
set -u

force=0
if [[ "${1:-}" == "--force" ]]; then
    force=1
fi

config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
state_home="${XDG_STATE_HOME:-$HOME/.local/state}"
marker="$config_home/playforge-theme-applied"
wallpaper=/usr/share/wallpapers/Playforge/contents/images/2560x1440.png
lookandfeel=org.playforge.desktop

install -d -m 0700 "$state_home/playforge"
exec >>"$state_home/playforge/theme.log" 2>&1

printf '\n[%s] Playforge-Theme wird angewendet (force=%s)\n' "$(date -Is)" "$force"

if (( ! force )) && [[ -f "$marker" ]]; then
    printf 'Bereits angewendet – nichts zu tun.\n'
    exit 0
fi

# Globales Theme anwenden: Farben, Icons, Plasma-Stil und Splashscreen.
if command -v plasma-apply-lookandfeel >/dev/null 2>&1; then
    plasma-apply-lookandfeel -a "$lookandfeel" || true
fi

# Fallbacks, falls das LNF-Werkzeug nicht verfügbar ist oder nicht alles setzt.
if command -v plasma-apply-colorscheme >/dev/null 2>&1; then
    plasma-apply-colorscheme Playforge || true
fi
if command -v plasma-apply-desktoptheme >/dev/null 2>&1; then
    plasma-apply-desktoptheme Playforge || true
fi
if command -v kwriteconfig6 >/dev/null 2>&1; then
    kwriteconfig6 --file kdeglobals --group KDE --key LookAndFeelPackage "$lookandfeel" || true
    kwriteconfig6 --file kdeglobals --group General --key ColorScheme Playforge || true
    kwriteconfig6 --file kdeglobals --group Icons --key Theme Playforge || true
    kwriteconfig6 --file plasmarc --group Theme --key name Playforge || true
    kwriteconfig6 --file ksplashrc --group KSplash --key Theme "$lookandfeel" || true
    kwriteconfig6 --file ksplashrc --group KSplash --key Engine KSplashQML || true
fi

# Wallpaper erst setzen, wenn plasmashell über D-Bus erreichbar ist.
wallpaper_ok=0
if command -v plasma-apply-wallpaperimage >/dev/null 2>&1 && [[ -f "$wallpaper" ]]; then
    for _ in $(seq 1 30); do
        if plasma-apply-wallpaperimage "$wallpaper"; then
            wallpaper_ok=1
            break
        fi
        sleep 1
    done
fi

if (( wallpaper_ok )); then
    install -d -m 0700 "$config_home"
    touch "$marker"
    printf 'Theme erfolgreich angewendet.\n'
else
    printf 'Wallpaper konnte noch nicht gesetzt werden – nächster Login versucht es erneut.\n'
fi
