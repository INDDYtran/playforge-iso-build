#!/usr/bin/env bash
# Playforge – baut das Control Center (Rust) für einen lokalen Test.
# Für das ISO übernimmt build.sh diesen Schritt automatisch.
set -Eeuo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cc_dir="$repo_dir/control-center"
out_dir="$repo_dir/out/control-center"

msg() { printf '\n==> %s\n' "$*"; }
die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}

command -v cargo >/dev/null 2>&1 ||
    die "cargo fehlt. Installieren: pacman -S --needed rust (oder rustup)."
[[ -f "$cc_dir/Cargo.toml" ]] || die "control-center/Cargo.toml fehlt."

msg "Baue playforge-control-center"
cargo build --release --locked --manifest-path "$cc_dir/Cargo.toml"

binary="$cc_dir/target/release/playforge-control-center"
[[ -x "$binary" ]] || die "Binary wurde nicht erzeugt: $binary"

mkdir -p "$out_dir"
install -m 0755 "$binary" "$out_dir/playforge-control-center"

msg "Fertig"
printf 'Binary: %s\n' "$out_dir/playforge-control-center"
printf 'Größe:  %s\n' "$(du -h "$out_dir/playforge-control-center" | cut -f1)"
