#!/usr/bin/env bash
# Playforge – verteilt Overlay-Dateien und das Control-Center-Binary auf die
# Paket-Staging-Verzeichnisse. Wird von build-repo.sh und check-coverage.sh
# aufgerufen.
#
#   PLAYFORGE_CONTROL_CENTER=/pfad/playforge-control-center \
#       bash packaging/pacman/stage.sh
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
base="$repo_dir/packaging/pacman"
overlay="$repo_dir/overlay"
cc="${PLAYFORGE_CONTROL_CENTER:-}"

die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

[[ -d "$overlay" ]] || die "overlay/ fehlt."
[[ -n "$cc" && -f "$cc" ]] || die "PLAYFORGE_CONTROL_CENTER fehlt oder ist keine Datei."

rm -rf -- \
    "$base/playforge-tools/stage" \
    "$base/playforge-theme/stage" \
    "$base/playforge-control-center/stage"

# ---------------------------------------------------------------- tools
tools="$base/playforge-tools/stage"
mkdir -p \
    "$tools/usr/local/bin" \
    "$tools/usr/local/lib" \
    "$tools/usr/local/share/playforge" \
    "$tools/usr/share/libalpm" \
    "$tools/usr/share/playforge" \
    "$tools/etc"

cp -a -- "$overlay/usr/local/bin/." "$tools/usr/local/bin/"
cp -a -- "$overlay/usr/local/lib/playforge" "$tools/usr/local/lib/"
cp -a -- "$overlay/usr/local/share/playforge/packages.target" "$tools/usr/local/share/playforge/"
cp -a -- "$overlay/usr/local/share/playforge/signing" "$tools/usr/local/share/playforge/"
if [[ -d "$overlay/usr/share/playforge" ]]; then
    cp -a -- "$overlay/usr/share/playforge/." "$tools/usr/share/playforge/"
fi
# Der öffentliche Repository-Schlüssel gehört ins Paket, aber nicht in den
# Tar-Update-Payload: alte Updater erlauben usr/share/playforge/ noch nicht.
cp -a -- "$base/pacman-repo.asc" "$tools/usr/share/playforge/"
cp -a -- "$overlay/usr/share/libalpm/hooks" "$tools/usr/share/libalpm/"

cp -a -- "$overlay/etc/environment.d" "$tools/etc/"
cp -a -- "$overlay/etc/gamemode.ini" "$tools/etc/"
cp -a -- "$overlay/etc/modules-load.d" "$tools/etc/"
cp -a -- "$overlay/etc/security" "$tools/etc/"
cp -a -- "$overlay/etc/skel" "$tools/etc/"
cp -a -- "$overlay/etc/sysctl.d" "$tools/etc/"
cp -a -- "$overlay/etc/udev" "$tools/etc/"
mkdir -p "$tools/etc/systemd/system"
cp -a -- "$overlay/etc/systemd/system/playforge-"* "$tools/etc/systemd/system/"
cp -a -- "$overlay/etc/systemd/zram-generator.conf" "$tools/etc/systemd/"
mkdir -p "$tools/etc/pacman.d"
# Auch die Repo-Konfiguration liegt außerhalb von overlay/, damit der alte
# Tar-Updater sie nicht als unbekannten Pfad ablehnt; playforge-repo-enable
# schreibt sie beim Einrichten, das Paket liefert die kanonische Fassung.
cp -a -- "$base/playforge.conf" "$tools/etc/pacman.d/playforge.conf"

# ---------------------------------------------------------------- theme
theme="$base/playforge-theme/stage"
mkdir -p "$theme/etc" "$theme/usr/share" "$theme/usr/local/share/playforge" \
    "$theme/usr/share/color-schemes" "$theme/usr/share/icons" \
    "$theme/usr/share/plasma/look-and-feel" "$theme/usr/share/plymouth/themes" \
    "$theme/usr/share/wallpapers" "$theme/usr/share/pixmaps"

cp -a -- "$overlay/etc/plymouth" "$theme/etc/"
cp -a -- "$overlay/etc/xdg" "$theme/etc/"
cp -a -- "$overlay/usr/share/color-schemes/Playforge.colors" "$theme/usr/share/color-schemes/"
cp -a -- "$overlay/usr/share/icons/Playforge" "$theme/usr/share/icons/"
cp -a -- "$overlay/usr/share/pixmaps/playforge-"* "$theme/usr/share/pixmaps/"
cp -a -- "$overlay/usr/share/plasma/look-and-feel/org.playforge.desktop" "$theme/usr/share/plasma/look-and-feel/"
cp -a -- "$overlay/usr/share/plymouth/themes/playforge" "$theme/usr/share/plymouth/themes/"
cp -a -- "$overlay/usr/share/wallpapers/Playforge" "$theme/usr/share/wallpapers/"
cp -a -- "$overlay/usr/local/share/playforge/branding" "$theme/usr/local/share/playforge/"

# ---------------------------------------------------------------- control center
ccstage="$base/playforge-control-center/stage"
mkdir -p "$ccstage/usr/local/bin" "$ccstage/usr/share/applications" \
    "$ccstage/usr/share/icons/hicolor/scalable/apps"
install -Dm0755 "$cc" "$ccstage/usr/local/bin/playforge-control-center"
install -Dm0644 "$overlay/usr/share/applications/org.playforge.controlcenter.desktop" \
    "$ccstage/usr/share/applications/org.playforge.controlcenter.desktop"
install -Dm0644 "$overlay/usr/share/icons/hicolor/scalable/apps/org.playforge.controlcenter.svg" \
    "$ccstage/usr/share/icons/hicolor/scalable/apps/org.playforge.controlcenter.svg"

printf 'Staging fertig:\n'
for dir in playforge-tools playforge-theme playforge-control-center; do
    printf '  %-26s %s Dateien\n' "$dir" \
        "$(find "$base/$dir/stage" -type f | wc -l)"
done
