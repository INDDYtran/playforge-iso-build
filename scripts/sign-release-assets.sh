#!/usr/bin/env bash
# Playforge – signiert Release-Dateien mit dem Playforge-Ed25519-Schlüssel.
#
#   PLAYFORGE_SIGNING_KEY_FILE=/pfad/key.pem \
#       bash scripts/sign-release-assets.sh datei1 datei2 ...
#
# Zu jeder Datei entsteht <datei>.sig. Der private Schlüssel wird nur gelesen,
# nie ausgegeben. Die Verifikation auf dem Zielsystem macht playforge-update
# mit dem eingebauten öffentlichen Schlüssel.
set -Eeuo pipefail

die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

key="${PLAYFORGE_SIGNING_KEY_FILE:-}"
[[ -n "$key" ]] || die "PLAYFORGE_SIGNING_KEY_FILE ist nicht gesetzt."
[[ -f "$key" ]] || die "Signaturschlüssel nicht gefunden: $key"
command -v openssl >/dev/null 2>&1 || die "openssl fehlt."
(( $# > 0 )) || die "Keine Dateien zum Signieren angegeben."

for file in "$@"; do
    [[ -f "$file" ]] || die "Datei nicht gefunden: $file"
    openssl pkeyutl -sign -inkey "$key" -rawin -in "$file" -out "$file.sig"
    printf '  signiert: %s.sig\n' "$file"
done
