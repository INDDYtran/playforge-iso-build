#!/usr/bin/env bash
# Playforge – erzeugt ein neues Ed25519-Schlüsselpaar für Release-Signaturen.
#
#   bash scripts/generate-signing-key.sh [ziel.pem]
#
# Der private Schlüssel bleibt lokal (Standard: .cache/), der öffentliche
# Schlüssel muss als overlay/usr/local/share/playforge/signing/update-signing.pub
# ins Image. Nach einer Rotation den privaten Schlüssel als Repository-Secret
# PLAYFORGE_SIGNING_KEY hinterlegen:
#
#   gh secret set PLAYFORGE_SIGNING_KEY --repo <owner/repo> < .cache/playforge-signing-key.pem
#
# Wichtig: Erst ein Update mit dem neuen öffentlichen Schlüssel veröffentlichen,
# das noch mit dem alten Schlüssel signiert ist. Sonst können bestehende
# Systeme das Update nicht mehr prüfen.
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-$repo_dir/.cache/playforge-signing-key.pem}"
pub="${target%.pem}.pub"

die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

command -v openssl >/dev/null 2>&1 || die "openssl fehlt."
[[ ! -e "$target" ]] || die "$target existiert bereits – nichts überschrieben."
mkdir -p -- "$(dirname -- "$target")"

openssl genpkey -algorithm ED25519 -out "$target"
chmod 600 -- "$target"
openssl pkey -in "$target" -pubout -out "$pub"

printf '\nPrivater Schlüssel: %s\n' "$target"
printf 'Öffentlicher Schlüssel: %s\n' "$pub"
printf '\nNächste Schritte:\n'
printf '  1. %s nach overlay/usr/local/share/playforge/signing/update-signing.pub kopieren\n' "$pub"
printf '  2. gh secret set PLAYFORGE_SIGNING_KEY --repo <owner/repo> < %s\n' "$target"
printf '  3. Privaten Schlüssel sicher sichern (Passwortmanager/offline).\n'
