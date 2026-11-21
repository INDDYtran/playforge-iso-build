#!/usr/bin/env bash
# Playforge – konfiguriert das frisch installierte Zielsystem (läuft im chroot).
set -Eeuo pipefail

info() { printf '\n==> %s\n' "$*"; }
warn() { printf 'Warnung: %s\n' "$*" >&2; }

config=/root/playforge-config.sh
[[ -f "$config" ]] || {
    printf 'Konfiguration %s fehlt.\n' "$config" >&2
    exit 1
}
# shellcheck disable=SC1090
source "$config"

# Overlay erneut anwenden: pacstrap hat Dateien wie /etc/pacman.conf oder
# /etc/gamemode.ini sonst mit Paketversionen überschrieben. /usr/local/lib
# wird ausgelassen, damit dieses laufende Skript nicht überschrieben wird.
if [[ -d /usr/local/share/playforge/overlay ]]; then
    cp -a /usr/local/share/playforge/overlay/etc/. /etc/
    cp -a /usr/local/share/playforge/overlay/usr/local/bin/. /usr/local/bin/
    cp -a /usr/local/share/playforge/overlay/usr/local/share/. /usr/local/share/
    cp -a /usr/local/share/playforge/overlay/usr/share/. /usr/share/
fi
chmod 0755 /usr/local/bin/playforge-* 2>/dev/null || true
chmod 0755 /usr/local/lib/playforge/*.sh 2>/dev/null || true

# Playforge-Theme systemweit einrichten (SDDM, Plasma-Stil).
/usr/local/lib/playforge/apply-theme.sh || warn "Playforge-Theme konnte nicht vollständig eingerichtet werden."

info "Zeitzone"
ln -sf "/usr/share/zoneinfo/$TIMEZONE" /etc/localtime
hwclock --systohc 2>/dev/null || warn "Hardware-Uhr konnte nicht gesetzt werden."

info "Locale"
locale_base="${LOCALE_NAME%%.*}"
if [[ ! -f "/usr/share/i18n/locales/$locale_base" ]]; then
    printf 'Locale %s ist nicht verfügbar.\n' "$LOCALE_NAME" >&2
    exit 1
fi
sed -i "s|^#[[:space:]]*\(${LOCALE_NAME} UTF-8\)|\1|" /etc/locale.gen
grep -q "^${LOCALE_NAME} UTF-8" /etc/locale.gen || printf '%s UTF-8\n' "$LOCALE_NAME" >>/etc/locale.gen
grep -q '^en_US.UTF-8 UTF-8' /etc/locale.gen || printf 'en_US.UTF-8 UTF-8\n' >>/etc/locale.gen
locale-gen
printf 'LANG=%s\n' "$LOCALE_NAME" >/etc/locale.conf

info "Tastatur"
printf 'KEYMAP=%s\n' "$KEYMAP" >/etc/vconsole.conf
install -d /etc/X11/xorg.conf.d
{
    printf 'Section "InputClass"\n'
    printf '    Identifier "system-keyboard"\n'
    printf '    MatchIsKeyboard "on"\n'
    printf '    Option "XkbLayout" "%s"\n' "${KEYMAP%%-*}"
    printf 'EndSection\n'
} >/etc/X11/xorg.conf.d/00-keyboard.conf

info "Hostname"
printf '%s\n' "$HOSTNAME" >/etc/hostname
cat >/etc/hosts <<EOF
127.0.0.1   localhost
::1         localhost
127.0.1.1   $HOSTNAME.localdomain $HOSTNAME
EOF

info "Benutzerkonto"
if ! getent group gamemode >/dev/null 2>&1; then
    groupadd -r gamemode 2>/dev/null || true
fi
user_groups="wheel,video,render,audio,input,storage"
if getent group gamemode >/dev/null 2>&1; then
    user_groups+=",gamemode"
fi
if ! id -u "$USERNAME" >/dev/null 2>&1; then
    useradd -m -G "$user_groups" -s /bin/bash "$USERNAME"
fi
install -d -m 0700 -o "$USERNAME" -g "$USERNAME" "/home/$USERNAME/.config"
if [[ -f /usr/share/pixmaps/playforge-logo.png ]]; then
    install -m 0644 -o "$USERNAME" -g "$USERNAME" \
        /usr/share/pixmaps/playforge-logo.png "/home/$USERNAME/.face.icon"
fi
printf '%s:%s\n' "$USERNAME" "$USER_HASH" | chpasswd -e
if [[ -n "$ROOT_HASH" ]]; then
    printf 'root:%s\n' "$ROOT_HASH" | chpasswd -e
else
    passwd -l root >/dev/null
fi
if grep -qE '^#[[:space:]]*%wheel[[:space:]]+ALL=\(ALL:ALL\)[[:space:]]+ALL' /etc/sudoers; then
    sed -i 's|^#[[:space:]]*\(%wheel[[:space:]]\+ALL=(ALL:ALL)[[:space:]]\+ALL\)|\1|' /etc/sudoers
fi
grep -qE '^%wheel[[:space:]]+ALL=\(ALL:ALL\)[[:space:]]+ALL' /etc/sudoers ||
    printf '%s\n' '%wheel ALL=(ALL:ALL) ALL' >>/etc/sudoers
visudo -cf /etc/sudoers >/dev/null

info "Grafiktreiber"
case "$GPU" in
    nvidia)
        printf 'NVIDIA (open-dkms) wird installiert – der DKMS-Build kann einige Minuten dauern.\n'
        if pacman -S --noconfirm --needed \
            nvidia-open-dkms nvidia-utils lib32-nvidia-utils nvidia-settings \
            libva-nvidia-driver egl-wayland nvidia-prime; then
            sed -i 's/^MODULES=.*/MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)/' /etc/mkinitcpio.conf
        else
            warn "NVIDIA-Treiber konnten nicht installiert werden – das System nutzt nouveau/NVK."
            pacman -Rns --noconfirm nvidia-open-dkms nvidia-utils lib32-nvidia-utils \
                nvidia-settings libva-nvidia-driver egl-wayland nvidia-prime 2>/dev/null || true
        fi
        ;;
    amd | intel | none)
        printf 'Mesa-/Vulkan-Treiber sind bereits installiert.\n'
        ;;
    *)
        warn "Unbekannte GPU-Auswahl: $GPU"
        ;;
esac

info "Dienste aktivieren"
systemctl enable NetworkManager.service
systemctl enable sddm.service
systemctl enable bluetooth.service
systemctl enable fstrim.timer
systemctl enable power-profiles-daemon.service
systemctl enable systemd-timesyncd.service
systemctl enable reflector.timer
systemctl enable playforge-profile.service
systemctl enable playforge-update-check.timer
systemctl set-default graphical.target

info "Initramfs erzeugen"
mkinitcpio -P

info "Bootloader einrichten"
if [[ "$UEFI" == "1" ]]; then
    if ! bootctl --esp-path=/boot install; then
        warn "bootctl konnte keinen NVRAM-Eintrag anlegen – installiere ohne EFI-Variable."
        bootctl --esp-path=/boot --no-variables install
    fi
    install -d /boot/loader/entries
    cat >/boot/loader/loader.conf <<'EOF'
default playforge.conf
timeout 4
EOF
    ucode=""
    if grep -qm1 'GenuineIntel' /proc/cpuinfo; then
        ucode="intel-ucode.img"
    fi
    if grep -qm1 'AuthenticAMD' /proc/cpuinfo; then
        ucode="amd-ucode.img"
    fi
    kernel_options="root=UUID=$ROOT_UUID rw nowatchdog"
    if [[ "$BTRFS" == "1" ]]; then
        kernel_options+=" rootflags=subvol=@"
    fi
    {
        printf 'title Playforge Linux\n'
        printf 'linux /vmlinuz-linux-zen\n'
        if [[ -n "$ucode" ]]; then
            printf 'initrd /%s\n' "$ucode"
        fi
        printf 'initrd /initramfs-linux-zen.img\n'
        printf 'options %s\n' "$kernel_options"
    } >/boot/loader/entries/playforge.conf
else
    if grep -q '^GRUB_TIMEOUT=' /etc/default/grub; then
        sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=3/' /etc/default/grub
    else
        printf 'GRUB_TIMEOUT=3\n' >>/etc/default/grub
    fi
    if grep -q '^GRUB_CMDLINE_LINUX_DEFAULT=' /etc/default/grub; then
        sed -i 's|^GRUB_CMDLINE_LINUX_DEFAULT=.*|GRUB_CMDLINE_LINUX_DEFAULT="nowatchdog"|' /etc/default/grub
    else
        printf 'GRUB_CMDLINE_LINUX_DEFAULT="nowatchdog"\n' >>/etc/default/grub
    fi
    if [[ "$BTRFS" == "1" ]]; then
        if grep -q '^GRUB_CMDLINE_LINUX=' /etc/default/grub; then
            sed -i 's|^GRUB_CMDLINE_LINUX=.*|GRUB_CMDLINE_LINUX="rootflags=subvol=@"|' /etc/default/grub
        else
            printf 'GRUB_CMDLINE_LINUX="rootflags=subvol=@"\n' >>/etc/default/grub
        fi
    fi
    if grep -qE '^#?GRUB_DISABLE_OS_PROBER=' /etc/default/grub; then
        sed -i 's/^#\?GRUB_DISABLE_OS_PROBER=.*/GRUB_DISABLE_OS_PROBER=false/' /etc/default/grub
    else
        printf 'GRUB_DISABLE_OS_PROBER=false\n' >>/etc/default/grub
    fi
    grub-install --target=i386-pc --recheck "$DISK"
    grub-mkconfig -o /boot/grub/grub.cfg
fi

rm -f "$config"
printf '\nZielsystem ist eingerichtet.\n'
