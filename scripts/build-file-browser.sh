#!/usr/bin/env bash
# Playforge – baut den Dateibrowser (Rust) für einen lokalen Test.
# Für das ISO übernimmt build.sh diesen Schritt automatisch.
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
fb_dir="$repo_dir/file-browser"
out_dir="$repo_dir/out/file-browser"

msg() { printf '\n==> %s\n' "$*"; }
die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

command -v cargo >/dev/null 2>&1 ||
    die "cargo fehlt. Installieren: pacman -S --needed rust (oder rustup)."
[[ -f "$fb_dir/Cargo.toml" ]] || die "file-browser/Cargo.toml fehlt."

msg "Baue playforge-files"
cargo build --release --locked --manifest-path "$fb_dir/Cargo.toml"

binary="$fb_dir/target/release/playforge-files"
[[ -x "$binary" ]] || die "Binary wurde nicht erzeugt: $binary"

mkdir -p "$out_dir"
install -m 0755 "$binary" "$out_dir/playforge-files"

msg "Fertig"
printf 'Binary: %s\n' "$out_dir/playforge-files"
printf 'Größe:  %s\n' "$(du -h "$out_dir/playforge-files" | cut -f1)"
