#!/usr/bin/env bash
# Baut das Playforge-ISO lokal in einem Linux-Container (Docker oder Podman).
set -Eeuo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$script_dir/.." && pwd)"

die() {
    printf 'Fehler: %s\n' "$*" >&2
    exit 1
}
info() { printf '\n==> %s\n' "$*"; }

# Standard: aktuelle Mirrors. Ein fester Archive-Snapshot (YYYY/MM/DD) ist
# reproduzierbarer, aber archive.archlinux.org ist nicht immer erreichbar.
archive_date="${ARCHIVE_DATE-}"
if [[ "$archive_date" == "latest" ]]; then
    archive_date=""
fi

epoch="${SOURCE_DATE_EPOCH:-}"
if [[ -z "$epoch" ]]; then
    epoch="$(git -C "$repo_dir" log -1 --format=%ct 2>/dev/null || true)"
fi
epoch="${epoch:-1795132800}"

if [[ -n "$archive_date" && ! "$archive_date" =~ ^[0-9]{4}/[0-9]{2}/[0-9]{2}$ ]]; then
    die "ARCHIVE_DATE muss YYYY/MM/DD oder 'latest' sein: $archive_date"
fi
if [[ ! "$epoch" =~ ^[0-9]+$ ]]; then
    die "SOURCE_DATE_EPOCH muss eine Unix-Zeit sein: $epoch"
fi

engine="${CONTAINER_ENGINE:-}"
if [[ -z "$engine" ]]; then
    if command -v docker >/dev/null 2>&1; then
        engine="docker"
    elif command -v podman >/dev/null 2>&1; then
        engine="podman"
    else
        die "Docker oder Podman nicht gefunden. Docker Desktop bzw. Podman installieren."
    fi
fi
command -v "$engine" >/dev/null 2>&1 || die "Container-Engine '$engine' nicht gefunden."

base_image="${PLAYFORGE_BASE_IMAGE:-archlinux/archlinux@sha256:66876082fe42b95d8778e581395b4f4e73d7043491470276bb3aa5a1f2009b5b}"

# Paketcache: Pacman laedt nur fehlende Pakete. Der Cache enthaelt reine
# Paketarchive; Aufloesung und Signaturpruefung bleiben beim gepinnten
# Snapshot. Der Pfad kann per PLAYFORGE_PACMAN_CACHE ueberschrieben werden.
cache_dir="${PLAYFORGE_PACMAN_CACHE:-$repo_dir/.cache/pacman}"
mkdir -p "$cache_dir" "$repo_dir/out"
cache_dir="$(cd -- "$cache_dir" && pwd)"

# Auf Windows/macOS liefert cygpath einen Pfad, den Docker Desktop versteht.
context="$repo_dir"
host_repo="$repo_dir"
host_cache="$cache_dir"
if command -v cygpath >/dev/null 2>&1; then
    context="$(cygpath -m "$repo_dir")"
    host_repo="$(cygpath -m "$repo_dir")"
    host_cache="$(cygpath -m "$cache_dir")"
fi

info "Baue Builder-Image mit $engine"
"$engine" build \
    --platform linux/amd64 \
    --build-arg "BASE_IMAGE=$base_image" \
    --build-arg "ARCHIVE_DATE=$archive_date" \
    --build-arg "SOURCE_DATE_EPOCH=$epoch" \
    -t playforge-builder "$context"

info "Baue ISO (kann 20–60 Minuten dauern)"
info "Paket-Cache: $cache_dir"
# mkarchiso benötigt chroot und Mounts; deshalb --privileged.
MSYS_NO_PATHCONV=1 "$engine" run --rm --privileged --platform linux/amd64 \
    -e "SOURCE_DATE_EPOCH=$epoch" \
    -e "ARCHIVE_DATE=$archive_date" \
    -e "PLAYFORGE_BOOT_TIMEOUT=${PLAYFORGE_BOOT_TIMEOUT:-1200}" \
    -e "PLAYFORGE_SKIP_BOOT_TEST=${PLAYFORGE_SKIP_BOOT_TEST:-0}" \
    -v "$host_repo/out:/build/out" \
    -v "$host_cache:/var/cache/pacman/pkg" \
    playforge-builder "$@"

info "Ergebnis in $repo_dir/out"
ls -lh "$repo_dir/out"
