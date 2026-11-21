# Playforge

**Playforge** ist eine auf Arch Linux basierende, für Gaming optimierte Distribution.
Sie wird aus dem offiziellen `archiso`-Releng-Profil gebaut und bringt einen
Live-Desktop (KDE Plasma, Wayland), einen Installer und ein auf Spiele
abgestimmtes Paket- und Konfigurationsset mit.

> Kein offizielles Arch-Linux-Projekt. Playforge ist ein eigenständiges Profil
> auf Arch-Basis; Support für das Basissystem gibt es im Arch Wiki.

## Enthalten

- **Kernel:** `linux-zen` (Desktop-/Gaming-Kernel) auf dem installierten System,
  `linux` + `linux-zen` auf dem Live-Medium
- **Desktop:** KDE Plasma (Wayland), SDDM, PipeWire, NetworkManager, BlueZ
- **Gaming:** Steam, Lutris, Wine, Winetricks, GameMode (+ 32-Bit), Gamescope,
  MangoHud (+ 32-Bit), GOverlay, VKD3D
- **Grafik:** Mesa, Vulkan (RADV/ANV/NVK), VA-API, LACT, 32-Bit-Treiber
- **NVIDIA:** optionaler `nvidia-open-dkms` (Turing oder neuer) im Installer
- **Tuning:** Sysctl-Profil, zram, GameMode-Konfiguration, MangoHud-Standardprofil,
  Shader-Cache-Limit, Gamecontroller-udev-Regeln, Leistungsprofile
- **Werkzeuge:** `playforge-install`, `playforge-doctor`, `playforge-tweaks`,
  `playforge-run`, `playforge-update`, `playforge-control-center`

## Voraussetzungen für den Build

- Arch Linux (x86_64), aktuell gehalten
- Paket `archiso` (`pacman -S --needed archiso`)
- aktiviertes `[multilib]`-Repository (für Steam und `lib32-*`):
  in `/etc/pacman.conf` `[multilib]` einkommentieren, danach `sudo pacman -Sy`
- `rust` für das Control Center (`sudo pacman -S --needed rust`); im
  Builder-Container ist Rust enthalten, auf einem Arch-Host ohne Rust wird das
  ISO ohne Control Center gebaut (Warnung am Ende von `build.sh`)
- ca. 25 GB freier Speicher, 8 GB RAM empfohlen
- Internetzugang

Das Profil ist für die aktuelle `archiso`-Version gedacht (Stand 2026). Bei
größeren archiso-Änderungen ggf. das Releng-Profil unter
`/usr/share/archiso/configs/releng` gegenprüfen.

Wer nicht auf einem Arch-Host baut, nutzt den Container bzw. den CI-Workflow
weiter unten; dort ist `archiso` bereits enthalten.

## ISO bauen

```bash
git clone <dieses-repo> playforge && cd playforge
sudo pacman -S --needed archiso
bash build.sh
```

Ergebnis:

```
out/playforge-<datum>-x86_64.iso
out/playforge-<datum>-x86_64.iso.sha256
```

Optionen:

```bash
bash build.sh --keep-work   # Arbeitsverzeichnis für Debugging behalten
bash build.sh --no-check    # Paketprüfung überspringen
```

Der Build dauert je nach CPU und Internet 20–60 Minuten. Das Skript kopiert das
Releng-Profil, legt das Playforge-Overlay darüber, erzeugt Paketliste und
`pacman.conf`, prüft alle Paketnamen gegen die Repos, rebrandet das Bootmenü und
ruft `mkarchiso` auf.

### Automatischer Boot-Smoke-Test

Der Container-/CI-Build prüft das ISO nicht nur auf Boot-Dateien, sondern startet es in QEMU
(q35, TCG) mit OVMF und serieller Konsole. Am Ende von `live-setup` schreibt das
Live-System den Marker `PLAYFORGE_LIVE_READY` auf `/dev/console` und direkt auf
`/dev/ttyS0`. Der Test wartet darauf, erkennt Kernel-Panics und einen frühen
QEMU-Abbruch und bricht nach einem Timeout ab (Standard: 1200 Sekunden). Das
serielle Protokoll liegt unter `out/smoke-test.log`.

Das ISO übergibt dafür `console=tty0 console=ttyS0,115200n8`; die grafische
Konsole bleibt über `tty0` erhalten. Der Marker belegt, dass das Live-Setup
(Overlay, Benutzer, Dienste) abgeschlossen wurde, nicht dass die grafische
KDE-Sitzung oder GPU-Treiber funktionieren.

Im Container ist QEMU/OVMF bereits installiert. Auf einem Arch-Host direkt:

```bash
sudo pacman -S --needed qemu-system-x86 edk2-ovmf
bash scripts/boot-smoke-test.sh out/playforge-*-x86_64.iso
```

Timeout und Überspringen:

```bash
PLAYFORGE_BOOT_TIMEOUT=1800 bash scripts/boot-smoke-test.sh out/playforge-*-x86_64.iso
PLAYFORGE_SKIP_BOOT_TEST=1 bash scripts/build-iso-container.sh
bash scripts/boot-smoke-test.sh --help
```

### Live-ISO manuell starten

```bash
sudo pacman -S --needed qemu-system-x86 edk2-ovmf
run_archiso -i out/playforge-*-x86_64.iso
```

### Auf USB schreiben

```bash
sudo dd if=out/playforge-*-x86_64.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

`/dev/sdX` ist der **gesamte** USB-Stick, nicht eine Partition. Alternativ
BalenaEtcher o. Ä. verwenden.

## Reproduzierbarer ISO-Build im Container / CI

Für Windows, macOS und Linux gibt es einen gepinnten Arch-Builder-Container. Er
nutzt einen festen Arch-Linux-Archive-Snapshot statt der tagesaktuellen Mirrors,
führt `build.sh` aus, prüft BIOS-/UEFI-Bootabbild, Kernel, Initramfs und
SquashFS, startet danach den QEMU-Boot-Smoke-Test und legt ISO, SHA256,
`smoke-test.log` und `BUILD-INFO.txt` in `out/` ab.

### Lokal (Docker/Podman)

Voraussetzungen: Docker Desktop mit Linux-Containern oder Podman, ca. 25 GB
freier Platz. `mkarchiso` braucht chroot und Mounts, daher läuft der Container
mit `--privileged`; baue damit nur Quellstände, denen du vertraust.

```bash
bash scripts/build-iso-container.sh
```

Auf Windows Git Bash wandelt das Skript die Pfade für Docker Desktop um. Manuell:

```bash
docker build --platform linux/amd64 -t playforge-builder .
mkdir -p out .cache/pacman
docker run --rm --privileged --platform linux/amd64 \
  -e SOURCE_DATE_EPOCH="$(date -u +%s)" \
  -v "$PWD/out:/build/out" \
  -v "$PWD/.cache/pacman:/var/cache/pacman/pkg" \
  playforge-builder
```

Optionen per Umgebung:

| Variable | Standard | Bedeutung |
| --- | --- | --- |
| `ARCHIVE_DATE` | aktuelle Mirrors | Arch-Linux-Archive-Snapshot (`YYYY/MM/DD`); `latest` oder leer = aktuelle Mirrors |
| `SOURCE_DATE_EPOCH` | Commit-Zeit bzw. `1795132800` | Zeitstempel im ISO |
| `PLAYFORGE_BASE_IMAGE` | gepinnter Digest | Basis-Image |
| `PLAYFORGE_PACMAN_CACHE` | `.cache/pacman` | Paketcache auf dem Host |
| `PLAYFORGE_BOOT_TIMEOUT` | `1200` | Timeout des QEMU-Boot-Smoke-Tests in Sekunden |
| `PLAYFORGE_SKIP_BOOT_TEST` | `0` | `1` überspringt den Boot-Smoke-Test |
| `CONTAINER_ENGINE` | automatisch | `docker` oder `podman` |

Beispiel mit festem Paketstand:

```bash
ARCHIVE_DATE=2026/11/20 bash scripts/build-iso-container.sh
```

### Paket- und Docker-Layer-Cache

Der Wrapper legt `.cache/pacman` an und bindet es in den Container als
`/var/cache/pacman/pkg` ein. Pacman verwendet vorhandene Paketarchive weiter und
lädt nur fehlende Versionen. Der Cache enthält reine Paketdateien, keine
aufgelösten Abhängigkeiten; Auflösung und Signaturprüfung erfolgen weiterhin
gegen den gepinnten Snapshot. Löschen ist gefahrlos:

```bash
rm -rf .cache/pacman
```

Im CI wird derselbe Pfad über `actions/cache` gesichert. Der Cache-Key enthält
Snapshot und Paketliste; `restore-keys` erlaubt eine teilweise Wiederverwendung
bei geänderter Paketliste, ohne die Versionsauflösung zu verändern.

Das Builder-Image installiert `archiso` in einem eigenen Layer **vor** `COPY`
und vor `SOURCE_DATE_EPOCH`. Dadurch nutzt Docker diesen Layer bei wiederholten
lokalen Builds weiter, solange Basis-Image und `ARCHIVE_DATE` gleich bleiben;
ein neuer Commit-Zeitstempel invalidiert ihn nicht. Im CI sind die
Builder-Layer pro Lauf frisch; dort greift der Paketcache über `actions/cache`.

### GitHub Actions

`.github/workflows/build-iso.yml` baut bei einem Tag `v*` oder manuell über
"Run workflow". Das Artefakt `playforge-iso-<commit>` enthält ISO, `.sha256`,
`BUILD-INFO.txt` und `smoke-test.log` (14 Tage Aufbewahrung). Die Standardwerte
lassen sich als Repository-Variablen überschreiben:

- `PLAYFORGE_ARCHIVE_DATE` (Standard `latest`; fester Snapshot z. B. `2026/11/20`)
- `PLAYFORGE_BASE_IMAGE` (Digest oder Tag)

Der Workflow läuft auf `ubuntu-24.04`, gibt vorher Speicherplatz frei und
startet den Build-Container mit `--privileged`. Der Container prüft die
Boot-Struktur (BIOS- und UEFI-El-Torito), Kernel/Initramfs und das SquashFS und
bootet das ISO anschließend in QEMU (TCG, OVMF, serielle Konsole). Der Test
wartet auf `PLAYFORGE_LIVE_READY`, das `live-setup` nach dem Kopieren des
Overlays und dem Anlegen des Live-Benutzers schreibt. Das Artefakt enthält ISO,
`.sha256`, `BUILD-INFO.txt` und `smoke-test.log`; bei einem Fehler wird das
serielle Protokoll zusätzlich als eigenes Artefakt hochgeladen. Geprüft wird
der Abschluss des Live-Setups, nicht die grafische KDE-Sitzung oder GPU/Audio.
Der Boot-Test läuft unter TCG und kann einige Minuten dauern; der Timeout liegt
bei 1200 Sekunden.

### Reproduzierbarkeit

Basis-Image-Digest, Arch-Archive-Snapshot und `SOURCE_DATE_EPOCH` sind gepinnt.
Damit sind Paketstand und Zeitstempel für denselben Quellstand festgelegt.
Bit-identische ISOs sind mit `archiso` derzeit nicht garantiert (u. a.
Dateireihenfolge/Timestamps in SquashFS); `archiso` Issue #247 beschreibt die
Grenzen. Wenn du Basis-Image und Snapshot aktualisierst, tue es gemeinsam, sonst
kann der ältere Schlüsselring des Images neuere Pakete nicht verifizieren.

Die Paketcaches ändern die Versionsauflösung nicht: Bei gepinntem Snapshot
verwendet Pacman nur Cache-Dateien mit passendem Namen und prüft weiterhin die
Signaturen. Sie sind reine Download-Beschleuniger.

**Hinweis zur Artefaktgröße:** GitHub-Actions-Artefakte sind auf 10 GB pro
Artefakt begrenzt; das ISO ist mehrere GB groß (typisch 2–4 GB). Bei privaten Repositories
zählt es gegen das Actions-Speicherkontingent (Free: 500 MB). Deshalb wird
jedes ISO zusätzlich als Release veröffentlicht (siehe Download).

## Download

Fertige ISOs liegen dauerhaft als GitHub-Release `iso-<commit>` bereit, nicht
nur als CI-Artefakt mit Ablaufdatum. GitHub erlaubt höchstens 2 GiB pro
Release-Asset – ein ISO hat rund 3 GB, deshalb besteht jedes Release aus
Teilen:

```bash
gh release download iso-9107dbdfe7a9 -R INDDYtran/playforge-iso-build
cat playforge-*.iso.part-* > playforge.iso
sha256sum -c playforge-*.iso.sha256
sudo dd if=playforge.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Die Teile lassen sich auch einzeln über die Release-Seite herunterladen;
`REJOIN.txt` im Release nennt Reihenfolge und Prüfsummen, `*.parts.sha256`
prueft die Teile einzeln. Veröffentlicht wird automatisch von
`.github/workflows/publish-iso.yml` nach jedem erfolgreichen ISO-Build oder
manuell mit einer Build-ID. Das Release `system-update` bleibt bewusst das
„latest“-Release, weil `playforge-update` seine Dateien von dort holt.

## Installation

Das Live-System startet direkt in KDE Plasma (Benutzer `gamer`, ohne Passwort).
Im Terminal oder über das Desktop-Symbol:

```bash
playforge-install
```

Der Installer:

1. zeigt alle Datenträger und verlangt eine doppelte Bestätigung (der Ziel-Datenträger wird **komplett gelöscht**),
2. richtet UEFI (GPT + ESP) oder BIOS (GPT + BIOS-Boot-Partition) ein,
3. formatiert ext4 oder btrfs (Subvolumes `@` und `@home`, zstd-Komprimierung),
4. installiert das Paketset aus `packages.target` (Internet erforderlich),
5. legt Benutzer, Locale, Tastatur, Hostname, zram und Dienste an,
6. installiert systemd-boot (UEFI) bzw. GRUB (BIOS),
7. installiert bei Bedarf den NVIDIA-Treiber.

**Voraussetzungen:** 32 GB Datenträger (mehr für Spiele), Internetzugang,
UEFI- oder BIOS-System. Eine vorhandene Windows-Installation auf derselben
Platte wird nicht erhalten – der Installer partitioniert die ganze Platte neu.

### NVIDIA

Der Installer bietet NVIDIA nur für Turing (RTX 20xx) oder neuer an
(`nvidia-open-dkms`). Ältere Karten laufen mit Mesa/NVK (Live-Medium und
Basissystem nutzen `nouveau`/NVK). Bei aktivem Secure Boot müssen die per DKMS
gebauten Module signiert werden, sonst laden sie nicht; im Zweifel Secure Boot
im UEFI deaktivieren. Der DKMS-Build dauert bei der Installation einige Minuten.

## Nach der Installation

```bash
playforge-doctor                      # Systemcheck: Treiber, Vulkan, 32-Bit, Tuning
playforge-tweaks status               # aktuelles Leistungsprofil
sudo playforge-tweaks performance     # dauerhaft maximale CPU-Leistung
sudo playforge-tweaks balanced        # ausgewogen (Standard)
sudo playforge-tweaks reset           # Profil entfernen
```

Das Profil wird in `/etc/playforge/profile` gespeichert und beim Booten von
`playforge-profile.service` angewendet.

### Systemupdate ohne neues ISO

`playforge-update` holt die Playforge-Systemdateien – Skripte, Theme,
Konfiguration und das Control Center – aus einem GitHub-Release und ersetzt
nur diese Dateien. Kernel, Pakete, Bootloader und Benutzerdaten bleiben
unangetastet; es wird also kein neues ISO gebraucht.

```bash
playforge-update --check      # gibt es etwas Neues? (Exit 10 = ja)
sudo playforge-update         # anwenden
sudo playforge-update --dry-run   # zeigen, was getauscht würde
sudo playforge-update --rollback  # letzte Sicherung zurückholen
```

Ablauf: Das Skript liest `playforge-update.version` aus dem Release
`system-update` (`releases/latest` von `INDDYtran/playforge-iso-build`),
vergleicht sie mit `/etc/playforge/version`, lädt `playforge-update.tar.gz`
samt Prüfsumme, prüft **jede** Datei gegen das mitgelieferte Manifest und
schreibt erst dann. Vorher sichert es die betroffenen Dateien samt Rechten
unter `/var/lib/playforge/backups` (die letzten drei Stände). Dateien, die aus
dem Projekt verschwunden sind, werden entfernt.

Geschrieben wird nur innerhalb fester Bereiche (`/usr/local/bin`,
`/usr/local/lib/playforge`, `/usr/local/share/playforge`, die
Playforge-Dateien unter `/usr/share`, `/etc/…`). Alles andere lehnt der
Updater ab. Quelle und Ziel lassen sich für Tests umlenken:

```bash
PLAYFORGE_UPDATE_REPO=owner/repo
PLAYFORGE_UPDATE_BASE=file:///tmp/release
PLAYFORGE_UPDATE_ROOT=/tmp/testwurzel
```

Damit man nicht selbst nachsehen muss, prüft `playforge-update-check.timer` das
Update einmal täglich (plus 10 Minuten nach dem Booten) und schreibt das
Ergebnis still nach `/var/lib/playforge/update-status`. Der Tab „Updates“ im
Control Center zeigt darauf „Update verfügbar“ mit installierter und neuer
Version, wie lange die Prüfung her ist und ob der Timer läuft. Installiert
wird dabei nichts: Wer das Update will, führt `sudo playforge-update` aus.
Der Timer wird bei der Installation eingeschaltet und von `playforge-update`
nachgeholt, falls er auf einem älteren System noch fehlt.

Veröffentlicht wird das Update von `.github/workflows/publish-update.yml`:
bei Änderungen an `overlay/` oder `control-center/` und manuell über
"Run workflow". Der Lauf baut das Control Center im Builder-Container, packt
Overlay und Binary zu `playforge-update.tar.gz` (mit `manifest.tsv`) und lädt
Archiv, Prüfsumme und Version in das Release `system-update`. Paketupdates
(pacman, Flatpak) macht dieses Werkzeug bewusst nicht.

### Playforge-Theme

Live-System und installiertes System verwenden das globale KDE-Theme
`org.playforge.desktop` mit dunklem Farbschema, Playforge-Wallpaper,
eigenem Startbildschirm (Splash), angepasstem SDDM-Anmeldebildschirm und
einem Icon-Theme auf Breeze-Basis. Die Einrichtung übernimmt
`/usr/local/lib/playforge/apply-theme.sh`; beim ersten Login wendet
`/usr/local/lib/playforge/apply-user-theme.sh` das Theme zusätzlich auf die
Plasma-Sitzung an (u. a. Wallpaper).

Theme in der laufenden Sitzung erneut anwenden:

```bash
playforge-theme
```

Das Theme lässt sich wie jedes globale KDE-Theme unter
*Systemeinstellungen → Erscheinungsbild → Globales Design* wechseln; die
Playforge-Varianten bleiben dort auswählbar.

### Playforge Control Center

`playforge-control-center` bündelt Systemstatus und Einstellungen in einer
Rust-Applikation (egui). Ein einzelnes Binary ohne Laufzeitabhängigkeiten,
also ohne GTK/Qt-Bibliotheken und ohne Webserver. Start über das
Anwendungsmenü oder `playforge-control-center`.

- **Übersicht** – Kernel, CPU, RAM/zram, GPU, Vulkan, Paketstand und eine
  Checkliste, was für Steam und Proton eingerichtet ist
- **Leistung** – Profil `performance`/`balanced`/`reset` über
  `playforge-tweaks`, Live-Werte (Governor, EPP, `vm.max_map_count`, BBR) und
  das Shader-Cache-Limit
- **Gaming** – GameMode-Parameter in `/etc/gamemode.ini`, MangoHud-Profile und
  ein Konfigurator für `playforge-run`-Startoptionen
- **Grafik** – GPUs mit Kernel-Treiber, Vulkan-ICDs, 32-Bit-Unterstützung,
  `vulkaninfo --summary` und `nvidia-smi`
- **Feintuning** – CPU-Governor und Energy-Performance-Preference (dauerhaft
  über `playforge-cpu-tune.service`), `vm.swappiness`,
  `vm.vfs_cache_pressure`, `vm.max_map_count` und Inotify-Limits,
  Datenträger-Scheduler per udev-Regel sowie `MESA_GLTHREAD`,
  `AMD_VULKAN_ICD` und `RADV_PERFTEST`
- **Updates** – pacman-, AUR- und Flatpak-Prüfung, `pacman -Syu` mit
  Live-Ausgabe, optional Flatpak und reflector-Spiegel sowie ein
  systemd-Timer für tägliche oder wöchentliche Automatik
- **Dienste** – Start/Stopp und Autostart der Playforge-relevanten
  systemd-Units
- **Diagnose** – `playforge-doctor` aufbereitet nach OK, Warnung und Fehler
- **Protokolle** – `journalctl -b`, gefiltert nach Playforge-Diensten

Systemänderungen schreibt die App niemals selbst: Sie ruft
`/usr/local/lib/playforge/control-helper.sh` über `pkexec` auf (auf dem
Live-Medium über das passwortlose sudo von `gamer`). Der Helfer prüft Befehle
und Werte gegen feste Listen; freie Pfade oder Unit-Namen nimmt er nicht an.
Das Feintuning landet in `/etc/playforge/cpu.conf`,
`/etc/sysctl.d/99-playforge-zz-tuning.conf`,
`/etc/udev/rules.d/60-playforge-io-scheduler.rules` und
`/etc/environment.d/90-playforge.conf`. Der Updater schreibt
`/etc/playforge/updates.conf` sowie
`/etc/systemd/system/playforge-autoupdate.{service,timer}` und merkt sich den
letzten Lauf in `/var/lib/playforge/last-update`.

Selbst bauen und testen:

```bash
bash scripts/build-control-center.sh   # out/control-center/playforge-control-center
./out/control-center/playforge-control-center
```

`build.sh` baut die App automatisch, sofern `cargo` vorhanden ist, und legt sie
in das Live-System und in die Installation.

### Spiele mit GameMode/MangoHud/Gamescope starten

In Steam als Startoption:

```
playforge-run --hud -- %command%
```

Weitere Varianten:

```bash
playforge-run --hud -- ./spiel            # GameMode + MangoHud
playforge-run --no-gamemode --hud -- ...  # nur MangoHud
playforge-run --gamescope --resolution 2560x1440 -- %command%
```

### Was das Tuning macht

| Datei | Wirkung |
| --- | --- |
| `/etc/sysctl.d/90-playforge.conf` | `vm.max_map_count` für Proton-Spiele, niedriges `swappiness`, Inotify-Limits, BBR |
| `/etc/systemd/zram-generator.conf` | zram-Swap (zstd, max. 16 GB) statt Platten-Swap |
| `/etc/gamemode.ini` | GameMode setzt während des Spiels den Performance-Governor |
| `/etc/security/limits.d/20-playforge.conf` | hohe Datei-Limits (Steam/Proton) |
| `/etc/environment.d/90-playforge.conf` | großer Mesa-Shader-Cache, NVIDIA-Cache ohne Auto-Löschung |
| `/etc/udev/rules.d/60-playforge-gamepads.rules` | kein USB-Ruhezustand für gängige Controller |
| `/etc/skel/.config/MangoHud/MangoHud.conf` | Standard-HUD (FPS, Frametime, Temperaturen) |
| `/etc/skel/.config/gamemode.ini` | benutzerspezifische GameMode-Config |
| `/etc/sysctl.d/99-playforge-zz-tuning.conf` | vom Control Center gesetzte Kernelwerte (überschreiben das Standard-Tuning) |
| `/etc/playforge/cpu.conf` + `playforge-cpu-tune.service` | Governor/EPP beim Booten wiederherstellen |
| `/etc/udev/rules.d/60-playforge-io-scheduler.rules` | Datenträger-Scheduler für SSD/NVMe |
| `/etc/playforge/updates.conf` + `playforge-autoupdate.timer` | automatische System- und Flatpak-Updates |

## Projektstruktur

```
build.sh                     Build-Skript (Arch-Host)
Dockerfile                   Gepinnter Arch-Builder-Container
.dockerignore                Build-Kontext des Containers
.github/workflows/           GitHub Actions: ISO bauen und als Artefakt ablegen
scripts/
  build-iso-container.sh     Lokaler Docker-/Podman-Build
  build-control-center.sh    Control Center (Rust) lokal bauen
  pack-update.sh             Systemupdate-Archiv für das GitHub-Release bauen
  container-entrypoint.sh    Build, Verify, Boot-Test und BUILD-INFO im Container
  verify-iso.sh              BIOS-/UEFI-/Kernel-/SquashFS-Prüfung
  boot-smoke-test.sh         QEMU/OVMF-Boottest mit serieller Konsole
control-center/              Control Center für Linux (Rust, egui)
  src/                       Fenster, Seiten, Systemabfragen, Aktionen
  Cargo.toml                 Abhängigkeiten und Release-Profil
profile/                     Overlay auf das archiso-Releng-Profil
  profiledef.sh              ISO-Name, Label, Bootmodi, Rechte
  airootfs/                  Nur Live-System (SDDM-Autologin, Live-Setup)
overlay/                     Dateien für Live-System *und* Installation
  etc/                       pacman.conf (multilib), Sysctl, zram, GameMode, udev, skel
  etc/xdg/                   KDE-Standardwerte und Theme-Autostart
  usr/local/bin/             playforge-install/-doctor/-tweaks/-run/-theme
  usr/local/lib/playforge/   configure-target.sh, control-helper.sh, Theme-Skripte
  usr/local/share/playforge/ Paketliste des Zielsystems
  usr/share/                 Theme, Wallpaper, Farbschema, Icons, Logo,
                             Menüeintrag des Control Centers
```

Der Paketcache unter `.cache/pacman/` wird beim Container-Build angelegt und ist
nicht versioniert.

## Bekannte Grenzen

- Nur x86_64.
- Installation benötigt Internet (kein Offline-Installer).
- Keine LUKS-Verschlüsselung und kein Dual-Boot-Resize im Installer (gesamte Platte).
- Kein Secure-Boot-Signing, keine Snapshots/Rollback-Einrichtung.
- Kein eigenes Paket-Repository; AUR ist nicht vorkonfiguriert. Für AUR-Helfer
  (z. B. `paru`) und AUR-Pakete wie `protonup-qt`, `heroic-games-launcher-bin`
  oder `vkbasalt` (nicht mehr in den offiziellen Repos) muss der AUR manuell
  eingerichtet werden.
- Der Live-Desktop läuft bei NVIDIA ohne proprietären Treiber (nouveau/NVK).
- Der automatische Boot-Test prüft nur den Abschluss des Live-Setups über einen
  seriellen Marker; Grafik, GPU-Treiber, Audio und der Installer werden in QEMU
  nicht getestet.
- Arch ist ein Rolling Release: Updates können manuelle Eingriffe erfordern.
