//! Dateisystem-Zugriffe: Verzeichnisse lesen, Vorschauen und Aktionen.
//!
//! Der Browser löscht nie leichtfertig: Wo `gio` oder `trash-put` vorhanden
//! sind, wandern Dateien in den Papierkorb. Erst wenn beides fehlt, wird
//! endgültig gelöscht – der Dialog weist darauf hin.

use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

const TEXT_PREVIEW_BYTES: usize = 96 * 1024;
const IMAGE_PREVIEW_LIMIT: u64 = 64 * 1024 * 1024;

/// Grobe Dateigattung – bestimmt Symbolfarbe und Vorschau.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Folder,
    Image,
    Text,
    Code,
    Archive,
    Audio,
    Video,
    Pdf,
    Other,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Folder => "Ordner",
            Kind::Image => "Bild",
            Kind::Text => "Textdatei",
            Kind::Code => "Quelltext",
            Kind::Archive => "Archiv",
            Kind::Audio => "Audiodatei",
            Kind::Video => "Videodatei",
            Kind::Pdf => "PDF-Dokument",
            Kind::Other => "Datei",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub kind: Kind,
    pub hidden: bool,
    pub mode: Option<u32>,
}

pub fn kind_for(path: &Path, is_dir: bool) -> Kind {
    if is_dir {
        return Kind::Folder;
    }
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tif" | "tiff"
        | "avif" | "heic" => Kind::Image,
        "txt" | "md" | "markdown" | "log" | "ini" | "conf" | "cfg" | "toml" | "yaml" | "yml"
        | "json" | "xml" | "csv" | "rtf" | "nfo" => Kind::Text,
        "rs" | "c" | "h" | "cpp" | "hpp" | "cc" | "py" | "js" | "ts" | "jsx" | "tsx" | "css"
        | "html" | "htm" | "sh" | "bash" | "zsh" | "fish" | "go" | "java" | "kt" | "lua" | "rb"
        | "php" | "sql" | "desktop" | "service" | "rules" | "patch" | "diff" => Kind::Code,
        "zip" | "tar" | "gz" | "bz2" | "xz" | "zst" | "7z" | "rar" | "iso" | "deb" | "rpm"
        | "pkg" | "jar" => Kind::Archive,
        "mp3" | "flac" | "ogg" | "wav" | "m4a" | "aac" | "opus" | "wma" => Kind::Audio,
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "wmv" | "flv" | "m4v" => Kind::Video,
        "pdf" => Kind::Pdf,
        _ => Kind::Other,
    }
}

/// Liest ein Verzeichnis. Nicht lesbare Einträge werden übersprungen, damit ein
/// einzelner defekter Link nicht den ganzen Ordner blockiert.
pub fn list(dir: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for item in fs::read_dir(dir)? {
        let Ok(item) = item else { continue };
        let path = item.path();
        let name = item.file_name().to_string_lossy().into_owned();
        let is_symlink = item
            .file_type()
            .map(|kind| kind.is_symlink())
            .unwrap_or(false);
        let meta = item
            .metadata()
            .or_else(|_| fs::symlink_metadata(&path))
            .ok();
        let is_dir = meta.as_ref().map(|meta| meta.is_dir()).unwrap_or(false);
        let size = if is_dir {
            0
        } else {
            meta.as_ref().map(|meta| meta.len()).unwrap_or(0)
        };
        let modified = meta.as_ref().and_then(|meta| meta.modified().ok());
        let mode = meta.as_ref().and_then(mode_of);
        entries.push(Entry {
            path: path.clone(),
            name: name.clone(),
            is_dir,
            is_symlink,
            size,
            modified,
            kind: kind_for(&path, is_dir),
            hidden: name.starts_with('.'),
            mode,
        });
    }
    Ok(entries)
}

#[cfg(unix)]
fn mode_of(meta: &fs::Metadata) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    Some(meta.permissions().mode() & 0o777)
}

#[cfg(not(unix))]
fn mode_of(_meta: &fs::Metadata) -> Option<u32> {
    None
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn format_time(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "—".to_string();
    };
    let local: chrono::DateTime<chrono::Local> = time.into();
    local.format("%d.%m.%Y %H:%M").to_string()
}

// --------------------------------------------------------------- Orte

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaceKind {
    Home,
    Desktop,
    Documents,
    Downloads,
    Music,
    Pictures,
    Videos,
    Root,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    Disk,
}

#[derive(Clone, Debug)]
pub struct Place {
    pub label: String,
    pub path: PathBuf,
    pub kind: PlaceKind,
}

pub fn places() -> Vec<Place> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };

    let mut places = vec![Place {
        label: "Persönlicher Ordner".to_string(),
        path: home.clone(),
        kind: PlaceKind::Home,
    }];

    let user_dirs = user_dirs(&home);
    let standard: [(&str, &str, PlaceKind); 6] = [
        ("Schreibtisch", "DESKTOP_DIR", PlaceKind::Desktop),
        ("Dokumente", "DOCUMENTS_DIR", PlaceKind::Documents),
        ("Downloads", "DOWNLOAD_DIR", PlaceKind::Downloads),
        ("Musik", "MUSIC_DIR", PlaceKind::Music),
        ("Bilder", "PICTURES_DIR", PlaceKind::Pictures),
        ("Videos", "VIDEOS_DIR", PlaceKind::Videos),
    ];
    for (label, key, kind) in standard {
        let fallback = match kind {
            PlaceKind::Desktop => "Desktop",
            PlaceKind::Documents => "Documents",
            PlaceKind::Downloads => "Downloads",
            PlaceKind::Music => "Music",
            PlaceKind::Pictures => "Pictures",
            PlaceKind::Videos => "Videos",
            _ => continue,
        };
        let path = user_dirs
            .get(key)
            .cloned()
            .unwrap_or_else(|| home.join(fallback));
        if path.is_dir() && path != home {
            places.push(Place {
                label: label.to_string(),
                path,
                kind,
            });
        }
    }

    places.push(Place {
        label: "Dateisystem".to_string(),
        path: root_dir(),
        kind: PlaceKind::Root,
    });

    places
}

/// Eingehängte Datenträger aus `/proc/self/mounts` (nur Linux). Der Eintrag für
/// `/` fehlt bewusst, weil ihn bereits „Dateisystem“ abdeckt.
pub fn mounts() -> Vec<Place> {
    #[cfg(target_os = "linux")]
    {
        let Ok(text) = fs::read_to_string("/proc/self/mounts") else {
            return Vec::new();
        };
        let mut places = Vec::new();
        let mut seen: HashSet<PathBuf> = HashSet::new();
        for line in text.lines() {
            let mut fields = line.split_whitespace();
            let (Some(device), Some(mount), Some(fstype)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            if !device.starts_with("/dev/") {
                continue;
            }
            if matches!(fstype, "squashfs" | "iso9660" | "udf") {
                continue;
            }
            let mount = unescape_mount(mount);
            if mount == Path::new("/") || !mount.is_dir() {
                continue;
            }
            if !seen.insert(mount.clone()) {
                continue;
            }
            let label = mount
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| mount.display().to_string());
            places.push(Place {
                label,
                path: mount,
                kind: PlaceKind::Disk,
            });
        }
        places.sort_by(|a, b| a.path.cmp(&b.path));
        places
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
}

fn root_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::current_dir()
            .ok()
            .and_then(|path| path.ancestors().last().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("C:\\"))
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/")
    }
}

/// Liest `~/.config/user-dirs.dirs`, damit auch lokalisierte XDG-Ordner
/// (etwa `~/Schreibtisch`) im Seitenbereich auftauchen.
fn user_dirs(home: &Path) -> HashMap<String, PathBuf> {
    let mut dirs = HashMap::new();
    let Ok(text) = fs::read_to_string(home.join(".config/user-dirs.dirs")) else {
        return dirs;
    };
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("XDG_") else {
            continue;
        };
        let Some((key, value)) = rest.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        let path = if let Some(relative) = value.strip_prefix("$HOME/") {
            home.join(relative)
        } else if value == "$HOME" {
            home.to_path_buf()
        } else {
            PathBuf::from(value)
        };
        dirs.insert(key.to_string(), path);
    }
    dirs
}

#[cfg(target_os = "linux")]
fn unescape_mount(raw: &str) -> PathBuf {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let oct: String = chars.by_ref().take(3).collect();
        match u8::from_str_radix(&oct, 8) {
            Ok(value) => out.push(value as char),
            Err(_) => {
                out.push('\\');
                out.push_str(&oct);
            }
        }
    }
    PathBuf::from(out)
}

// --------------------------------------------------------------- Vorschau

pub struct TextPreview {
    pub content: String,
    pub truncated: bool,
}

pub fn text_preview(path: &Path) -> Option<TextPreview> {
    let mut file = File::open(path).ok()?;
    let mut buffer = Vec::with_capacity(32 * 1024);
    file.by_ref()
        .take((TEXT_PREVIEW_BYTES + 1) as u64)
        .read_to_end(&mut buffer)
        .ok()?;
    let truncated = buffer.len() > TEXT_PREVIEW_BYTES;
    if truncated {
        buffer.truncate(TEXT_PREVIEW_BYTES);
    }
    if buffer.iter().take(2048).any(|byte| *byte == 0) {
        return None;
    }
    Some(TextPreview {
        content: String::from_utf8_lossy(&buffer).into_owned(),
        truncated,
    })
}

pub fn decode_image(path: &Path) -> Option<image::DynamicImage> {
    let meta = fs::metadata(path).ok()?;
    if meta.len() > IMAGE_PREVIEW_LIMIT {
        return None;
    }
    image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
}

// --------------------------------------------------------------- Aktionen

pub fn open_external(path: &Path) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut command = Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");

    command.arg(path).spawn().map(|_| ())
}

/// Verschiebt in den Papierkorb, wenn ein Werkzeug dafür vorhanden ist.
pub fn trash(path: &Path) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        for (program, prefix) in [("gio", Some("trash")), ("trash-put", None)] {
            let mut command = Command::new(program);
            if let Some(prefix) = prefix {
                command.arg(prefix);
            }
            command.arg("--").arg(path);
            if command
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
            {
                return Ok(());
            }
        }
    }
    remove(path)
}

fn remove(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}
