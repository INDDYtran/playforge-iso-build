# Playforge-Paketquelle

Die Paketquelle liefert die Playforge-eigenen Bestandteile als echte
pacman-Pakete:

| Paket | Inhalt |
|---|---|
| `playforge-tools` | Skripte, systemd-Units, Konfiguration, Snapshot-Hook, Repository-Schlüssel |
| `playforge-theme` | KDE-Theme, Wallpaper, Plymouth-, SDDM- und Bootmenü-Grafik |
| `playforge-control-center` | Rust-Binary, Desktop-Eintrag und Icon |
| `playforge-files` | Dateibrowser-Binary, Desktop-Eintrag und Icon |

Damit gibt es Versionen, Abhängigkeiten, `pacman -Syu`, Downgrade über den
Paketcache oder eine ältere Datei aus der Quelle — und keine Dateien mehr, die
nur per Tar-Overlay ersetzt werden.

## Quelle

- URL: `https://inddytran.github.io/playforge-iso-build/x86_64`
- Konfiguration: `/etc/pacman.d/playforge.conf` (Quelle: `packaging/pacman/playforge.conf`)
- Signatur: `SigLevel = Required DatabaseRequired`
- Öffentlicher Schlüssel: `packaging/pacman/pacman-repo.asc` (installiert als `/usr/share/playforge/pacman-repo.asc`, zusätzlich in `playforge-repo-enable` eingebettet)

`playforge-repo-enable` bringt den öffentlichen Schlüssel eingebettet mit (damit die Migration auch mit dem alten Tar-Updater klappt), importiert ihn, hängt
`Include = /etc/pacman.d/playforge.conf` an `/etc/pacman.conf` an und
installiert die vier Pakete. Auf einer frischen Installation ruft
`configure-target.sh` das automatisch auf. Auf bestehenden Systemen einmalig:

```bash
sudo playforge-repo-enable
```

Danach:

```bash
sudo pacman -Syu
```

`playforge-update` erkennt installierte `playforge-tools` und verweist auf
pacman, statt die Paketdateien erneut per Tar zu überschreiben. Für den
Notfall lässt sich der alte Weg mit `PLAYFORGE_UPDATE_LEGACY=1` erzwingen.

## Downgrade

Alte Paketversionen bleiben in der Quelle und im pacman-Cache. Beispiele:

```bash
ls /var/cache/pacman/pkg/playforge-tools-*
sudo pacman -U /var/cache/pacman/pkg/playforge-tools-<ältere-version>-1-any.pkg.tar.zst

# oder direkt aus der Quelle
sudo pacman -U https://inddytran.github.io/playforge-iso-build/x86_64/playforge-tools-<version>-1-any.pkg.tar.zst
```

## Bauen und veröffentlichen

`.github/workflows/publish-pacman.yml` baut bei Änderungen an `overlay/`,
`control-center/`, `file-browser/` oder `packaging/pacman/`:

1. Control Center und Dateibrowser mit Cargo,
2. die vier Pakete mit `makepkg`,
3. die signierte Datenbank mit `repo-add --sign`,
4. prüft DB- und Paketsignaturen lokal mit pacman,
5. schreibt die Pakete und die Datenbank in den Branch `gh-pages`,
6. prüft die veröffentlichte Quelle noch einmal über HTTPS.

Versionen haben die Form `2026.11.22.r123.gabcdef0` (Datum, Commit-Anzahl,
Kurzhash) und steigen mit jedem Commit.

Lokal in einem Arch-Container:

```bash
PLAYFORGE_PKGVER=2026.11.22.r123.gabcdef0 \
PLAYFORGE_GPG_KEY=/pfad/repo-key.asc \
PLAYFORGE_OUT=$PWD/out/pacman \
    bash packaging/pacman/build-repo.sh
```

Vor dem Commit prüfen, ob jede Overlay-Datei genau einem Paket zugeordnet ist:

```bash
git add overlay packaging/pacman
bash packaging/pacman/check-coverage.sh
```

## Schlüssel

Der öffentliche Schlüssel liegt in `packaging/pacman/pacman-repo.asc` und noch
einmal eingebettet in `playforge-repo-enable`; bei einer Rotation müssen beide
aktualisiert werden. Der private Repository-Schlüssel liegt nur als GitHub-Secret
`PLAYFORGE_REPO_GPG_KEY` und lokal unter
`.cache/playforge-repo-signing-key.asc` (nicht versioniert). Er ist **nicht**
derselbe Schlüssel wie der Update-Schlüssel (`update-signing.pub`); pacman
braucht OpenPGP, der Updater nutzt Ed25519 per OpenSSL.

Rotation: neuen Schlüssel erzeugen, öffentlichen Schlüssel ersetzen, mit dem
alten Schlüssel signieren, veröffentlichen und erst danach das Secret tauschen.
`playforge-repo-enable` importiert den neuen Schlüssel beim nächsten Lauf.
