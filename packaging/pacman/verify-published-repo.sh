#!/usr/bin/env bash
# Playforge – prüft die veröffentlichte Paketquelle mit pacman.
#
#   PLAYFORGE_REPO_URL=https://…/x86_64 \
#   PLAYFORGE_REPO_KEY=/key.asc \
#   PLAYFORGE_EXPECTED_VERSION=2026.11.22.r123.gabcdef0 \
#       bash packaging/pacman/verify-published-repo.sh
set -Eeuo pipefail

url="${PLAYFORGE_REPO_URL:?PLAYFORGE_REPO_URL fehlt}"
key="${PLAYFORGE_REPO_KEY:-/key.asc}"
expected="${PLAYFORGE_EXPECTED_VERSION:-}"
die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

[[ -f "$key" ]] || die "Schlüssel fehlt: $key"
command -v pacman >/dev/null 2>&1 || die "pacman fehlt."

pacman-key --init >/dev/null
pacman-key --add "$key" >/dev/null
fpr="$(gpg --batch --with-colons --import-options show-only --import "$key" |
    awk -F: '/^fpr:/{print $10; exit}')"
[[ -n "$fpr" ]] || die "Fingerprint nicht lesbar."
pacman-key --lsign-key "$fpr" >/dev/null

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT
cat >"$tmp/pacman.conf" <<EOF
[options]
Architecture = auto
SigLevel = Required DatabaseRequired
DBPath = $tmp/db
CacheDir = $tmp/cache
[playforge]
Server = $url
EOF

printf 'Warte auf die veröffentlichte Datenbank'
ok=0
for _ in $(seq 1 30); do
    rm -rf -- "$tmp/db" "$tmp/cache"
    mkdir -p "$tmp/db" "$tmp/cache"
    if pacman -Sy --config "$tmp/pacman.conf" --noconfirm >/dev/null 2>&1 &&
        pacman -Si --config "$tmp/pacman.conf" playforge-tools >"$tmp/info" 2>/dev/null; then
        if [[ -z "$expected" ]]; then
            ok=1
            break
        fi
        version="$(awk -F': ' '/^Version[[:space:]]*:/{print $2; exit}' "$tmp/info")"
        if [[ "$version" == "$expected"-* ]]; then
            ok=1
            break
        fi
        printf '\n  noch %s statt %s' "$version" "$expected"
    else
        printf '.'
    fi
    sleep 15
done
printf '\n'
(( ok )) || die "playforge.db ist unter $url nicht mit Version $expected verfügbar."

cat "$tmp/info"
pacman -Sw --config "$tmp/pacman.conf" --nodeps --nodeps --noconfirm \
    playforge-tools playforge-theme playforge-control-center playforge-files

printf '\nVeröffentlichte Paketquelle ist signiert und nutzbar.\n'
