//! Privilegierte Aktionen.
//!
//! Das Control Center schreibt selbst nie nach `/etc`. Alle Änderungen laufen
//! über `/usr/local/lib/playforge/control-helper.sh`, ein Bash-Skript mit fester
//! Befehlsliste, das nach Administratorrechten fragt (pkexec).

use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::system::have;

pub const HELPER: &str = "/usr/local/lib/playforge/control-helper.sh";

#[derive(Clone, Debug)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}

impl ActionResult {
    fn success(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
        }
    }

    fn failure(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
        }
    }
}

/// Startet eine Helfer-Aktion im Hintergrund (der pkexec-Dialog blockiert die
/// Oberfläche nicht).
pub fn spawn_helper(args: Vec<String>, label: String) -> Receiver<ActionResult> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(run_helper(&args, &label));
    });
    rx
}

/// Führt ein Benutzerprogramm im Hintergrund aus (z. B. `playforge-theme`).
pub fn spawn_command(program: String, args: Vec<String>) -> Receiver<ActionResult> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(match run_plain(&program, &args) {
            Ok(output) if output.status.success() => {
                ActionResult::success(summarize(&output, "Ausgeführt."))
            }
            Ok(output) => ActionResult::failure(error_summary(&output)),
            Err(err) => {
                ActionResult::failure(format!("{program} konnte nicht gestartet werden: {err}"))
            }
        });
    });
    rx
}

fn run_helper(args: &[String], label: &str) -> ActionResult {
    if !cfg!(target_os = "linux") {
        return ActionResult::failure(format!(
            "{label}: Änderungen am System sind nur unter Playforge Linux möglich."
        ));
    }
    if !Path::new(HELPER).exists() {
        return ActionResult::failure(format!(
            "Der Playforge-Helfer fehlt ({HELPER}). Bitte das System aktualisieren."
        ));
    }

    // Live-Medium und Systeme mit aktivem sudo-Ticket: ohne Rückfrage.
    if have("sudo") {
        if let Ok(output) = run_plain("sudo", &join(&["-n", HELPER], args)) {
            if output.status.success() {
                return ActionResult::success(summarize(&output, "Änderung übernommen."));
            }
        }
    }

    // Regulärer Weg: polkit fragt über den KDE-Agenten nach dem Passwort.
    if have("pkexec") {
        match run_plain("pkexec", &join(&[HELPER], args)) {
            Ok(output) if output.status.success() => {
                return ActionResult::success(summarize(&output, "Änderung übernommen."))
            }
            Ok(output) => return ActionResult::failure(error_summary(&output)),
            Err(err) => {
                return ActionResult::failure(format!(
                    "pkexec konnte nicht gestartet werden: {err}"
                ))
            }
        }
    }

    ActionResult::failure(
        "Weder sudo noch pkexec gefunden – die Änderung kann nicht autorisiert werden.",
    )
}

fn join(prefix: &[&str], args: &[String]) -> Vec<String> {
    prefix
        .iter()
        .map(|value| value.to_string())
        .chain(args.iter().cloned())
        .collect()
}

fn run_plain(program: &str, args: &[String]) -> std::io::Result<Output> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .env("SYSTEMD_COLORS", "0")
        .env("NO_COLOR", "1");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd.output()
}

fn summarize(output: &Output, fallback: &str) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        fallback.to_string()
    } else {
        first_line(&stdout)
    }
}

fn error_summary(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return first_line(&stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return first_line(&stdout);
    }
    match output.status.code() {
        Some(code) => format!("Aktion abgebrochen (Exit-Code {code})."),
        None => "Aktion abgebrochen.".to_string(),
    }
}

fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    if line.chars().count() > 220 {
        format!("{}…", line.chars().take(220).collect::<String>())
    } else {
        line.to_string()
    }
}
