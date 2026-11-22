# syntax=docker/dockerfile:1
#
# Playforge ISO-Builder
#
# Baut das ISO in einem reproduzierbaren Arch-Linux-Container. Basis-Image und
# Paket-Snapshot sind gepinnt; SOURCE_DATE_EPOCH steuert die Zeitstempel von
# mkarchiso. Der Build läuft als root, weil mkarchiso chroot und Mounts
# benötigt; der Container muss deshalb mit --privileged gestartet werden.

ARG BASE_IMAGE=archlinux/archlinux@sha256:66876082fe42b95d8778e581395b4f4e73d7043491470276bb3aa5a1f2009b5b
FROM ${BASE_IMAGE}

ARG BASE_IMAGE
ARG ARCHIVE_DATE=

ENV ARCHIVE_DATE=${ARCHIVE_DATE} \
    TZ=UTC \
    LC_ALL=C.UTF-8

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Wichtig fuer den Docker-Layer-Cache: Die Paketinstallation steht bewusst vor
# COPY und vor SOURCE_DATE_EPOCH. Ein neuer Commit-Zeitstempel oder eine
# Quellcode-Aenderung invalidiert diesen teuren Layer damit nicht. Der Layer
# haengt nur von Basis-Image und ARCHIVE_DATE ab.
#
# Arch Linux Archive für einen festen Paketstand verwenden. Ist ARCHIVE_DATE
# leer, wird ein aktueller Mirror genutzt (Standard, weil Archive-Snapshots
# tageweise unerreichbar sein können).
#
# Das offizielle Arch-Basis-Image enthaelt in /etc/pacman.conf keine
# [multilib]-Sektion. Ohne sie fehlt multilib.db, und build.sh kann die
# 32-Bit-Pakete (steam, lib32-*) nicht prüfen. QEMU und OVMF werden für den
# Boot-Smoke-Test im Container benötigt.
# archive.archlinux.org ist nicht immer erreichbar (Timeouts). Deshalb:
# mehrere Versuche pro Synchronisation und ein aktueller Mirror als Rückfall,
# damit ein einzelner Aussetzer den Build nicht mehr abbricht.
RUN set -eux; \
    archive_mirror=""; \
    fallback_mirror="https://geo.mirror.pkgbuild.com"; \
    if [[ -n "${ARCHIVE_DATE}" ]]; then \
        archive_mirror="https://archive.archlinux.org/repos/${ARCHIVE_DATE}"; \
        printf 'Server = %s/$repo/os/$arch\n' "$archive_mirror" \
            > /etc/pacman.d/mirrorlist; \
    else \
        printf 'Server = %s/$repo/os/$arch\n' "$fallback_mirror" \
            > /etc/pacman.d/mirrorlist; \
    fi; \
    if ! grep -q '^\[multilib\]' /etc/pacman.conf; then \
        printf '\n[multilib]\nInclude = /etc/pacman.d/mirrorlist\n' >> /etc/pacman.conf; \
    fi; \
    sync_repos() { \
        local attempt; \
        for attempt in 1 2 3 4; do \
            if pacman -Syy --noconfirm; then \
                return 0; \
            fi; \
            echo "pacman -Syy fehlgeschlagen (Versuch ${attempt}/4), neuer Versuch in 20s" >&2; \
            sleep 20; \
        done; \
        return 1; \
    }; \
    use_fallback() { \
        printf 'Server = %s/$repo/os/$arch\nServer = %s/$repo/os/$arch\n' \
            "${archive_mirror}" "$fallback_mirror" > /etc/pacman.d/mirrorlist; \
    }; \
    if ! sync_repos; then \
        echo "Archiv-Mirror '${archive_mirror}' nicht erreichbar, nutze $fallback_mirror" >&2; \
        use_fallback; \
        sync_repos; \
    fi; \
    pacman -S --needed --noconfirm archiso qemu-system-x86 edk2-ovmf \
        grub mtools dosfstools \
        rust gcc pkgconf wayland libxkbcommon libx11 libxcursor libxrandr libxi; \
    pacman -Scc --noconfirm; \
    sync_repos

# Erst nach dem Paket-Layer setzen: SOURCE_DATE_EPOCH und der Basis-Digest
# werden nur für den Build-Lauf gebraucht und sollen den Layer nicht treffen.
ARG SOURCE_DATE_EPOCH=1795132800
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH} \
    PLAYFORGE_BASE_IMAGE=${BASE_IMAGE}

COPY . /build

RUN set -eux; \
    install -m 0755 /build/scripts/container-entrypoint.sh /usr/local/bin/playforge-build; \
    install -m 0755 /build/scripts/verify-iso.sh /usr/local/bin/playforge-verify-iso; \
    install -m 0755 /build/scripts/boot-smoke-test.sh /usr/local/bin/playforge-boot-smoke; \
    for script in \
        /build/build.sh \
        /build/scripts/build-file-browser.sh \
        /build/scripts/container-entrypoint.sh \
        /build/scripts/verify-iso.sh \
        /build/scripts/boot-smoke-test.sh \
        /build/scripts/pack-update.sh \
        /build/overlay/usr/local/bin/playforge-update \
        /build/overlay/usr/local/lib/playforge/control-helper.sh \
        /build/profile/airootfs/usr/local/lib/playforge/live-setup; do \
        bash -n "$script"; \
    done

WORKDIR /build
ENTRYPOINT ["/usr/local/bin/playforge-build"]
