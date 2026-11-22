#!/usr/bin/env bash
# Playforge – baut die Pakete und die signierte pacman-Repository-Datenbank.
#
# Läuft in einem Arch-Container als root. Voraussetzungen: base-devel, rust,
# gnupg, pacman-contrib, pkgconf, wayland/libxkbcommon/libx11/… für das
# Control Center.
#
#   PLAYFORGE_PKGVER=2026.11.22.r123.gabcdef0 \
#   PLAYFORGE_GPG_KEY=/pfad/repo-key.asc \
#   PLAYFORGE_OUT=/out/pacman \
#       bash packaging/pacman/build-repo.sh
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
out="${PLAYFORGE_OUT:-$repo_dir/out/pacman}"
pkgdir="$out/x86_64"
ver="${PLAYFORGE_PKGVER:?PLAYFORGE_PKGVER fehlt}"
key="${PLAYFORGE_GPG_KEY:?PLAYFORGE_GPG_KEY fehlt}"
epoch="${SOURCE_DATE_EPOCH:-$(date -u +%s)}"

die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}
msg() { printf '\n==> %s\n' "$*"; }

[[ -f "$key" ]] || die "GPG-Schlüssel nicht gefunden: $key"
[[ "$ver" =~ ^[0-9][A-Za-z0-9._]*$ ]] || die "Ungültige Paketversion: $ver"
for cmd in makepkg repo-add gpg pacman-key cargo; do
    command -v "$cmd" >/dev/null 2>&1 || die "$cmd fehlt."
done

export GNUPGHOME="${PLAYFORGE_GNUPGHOME:-$(mktemp -d)}"
chmod 700 "$GNUPGHOME"
gpg --batch --import "$key" >/dev/null 2>&1 || die "GPG-Schlüssel konnte nicht importiert werden."
fpr="$(gpg --batch --with-colons --list-secret-keys |
    awk -F: '/^fpr:/{print $10; exit}')"
[[ -n "$fpr" ]] || die "Kein GPG-Fingerprint gefunden."
msg "Signaturschlüssel $fpr"

if ! id builder >/dev/null 2>&1; then
    useradd -m builder
fi
chown -R builder "$repo_dir"
mkdir -p "$pkgdir"
chown -R builder "$out"

msg "Baue Control Center"
runuser -u builder -- env HOME=/home/builder CARGO_HOME=/home/builder/.cargo \
    bash -c "cd '$repo_dir/control-center' && cargo build --release --locked"
cc="$repo_dir/control-center/target/release/playforge-control-center"
[[ -x "$cc" ]] || die "Control-Center-Binary fehlt nach dem Build."

msg "Stelle Paketinhalte zusammen"
PLAYFORGE_CONTROL_CENTER="$cc" bash "$repo_dir/packaging/pacman/stage.sh"
chown -R builder "$repo_dir/packaging/pacman"

for pkg in playforge-tools playforge-theme playforge-control-center; do
    msg "Baue $pkg"
    (
        cd "$repo_dir/packaging/pacman/$pkg"
        runuser -u builder -- env HOME=/home/builder \
            PLAYFORGE_PKGVER="$ver" \
            SOURCE_DATE_EPOCH="$epoch" \
            PKGDEST="$pkgdir" \
            makepkg --noconfirm --nodeps --force
    )
done

shopt -s nullglob
packages=("$pkgdir"/*.pkg.tar.zst)
(( ${#packages[@]} > 0 )) || die "Keine Pakete gebaut."
for file in "${packages[@]}"; do
    gpg --batch --yes --detach-sign --no-armor -u "$fpr" "$file"
done

msg "Erzeuge Repository-Datenbank"
cd "$pkgdir"
rm -f playforge.db playforge.db.* playforge.files playforge.files.*
repo-add --sign playforge.db.tar.gz ./*.pkg.tar.zst
[[ -f playforge.db.tar.gz.sig ]] || die "Datenbank wurde nicht signiert."

# pacman fragt playforge.db (ohne .tar.gz) an; Signaturen gelten für identische
# Bytes und werden mitkopiert.
rm -f playforge.db playforge.files
cp playforge.db.tar.gz playforge.db
cp playforge.files.tar.gz playforge.files
cp playforge.db.tar.gz.sig playforge.db.sig
cp playforge.files.tar.gz.sig playforge.files.sig

msg "Prüfe Repository lokal"
tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT
mkdir -p "$tmp/db" "$tmp/cache" "$tmp/gnupg"
chmod 700 "$tmp/gnupg"
pacman-key --gpgdir "$tmp/gnupg" --init >/dev/null
pacman-key --gpgdir "$tmp/gnupg" --add "$repo_dir/packaging/pacman/pacman-repo.asc" >/dev/null
pacman-key --gpgdir "$tmp/gnupg" --lsign-key "$fpr" >/dev/null
cat >"$tmp/pacman.conf" <<EOF
[options]
Architecture = auto
SigLevel = Required DatabaseRequired
GPGDir = $tmp/gnupg
DBPath = $tmp/db
CacheDir = $tmp/cache
[playforge]
Server = file://$pkgdir
EOF
pacman -Sy --config "$tmp/pacman.conf" --noconfirm >/dev/null
pacman -Si --config "$tmp/pacman.conf" playforge-tools >/dev/null
pacman -Sw --config "$tmp/pacman.conf" --nodeps --noconfirm \
    playforge-tools playforge-theme playforge-control-center >/dev/null

msg "Fertig"
printf '  Version: %s\n' "$ver"
printf '  Pakete:\n'
for file in "${packages[@]}"; do
    printf '    %s\n' "$(basename "$file")"
done
printf '  Datenbank: %s/playforge.db\n' "$pkgdir"
