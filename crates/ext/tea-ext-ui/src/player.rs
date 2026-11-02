//! Tocar os sons do componente `audio` pelo tocador do sistema (`pw-play`, `paplay`, `aplay` ou `afplay`).
//!
//! Um som de cada vez. O tocador roda num grupo de processos próprio e morre inteiro ao parar, ao trocar a
//! revisão do documento e quando a extensão sai. O `loop` é daqui: quando o tocador termina bem, roda de novo.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Os tocadores, na ordem de preferência.
#[cfg(target_os = "macos")]
const PLAYERS: &[&str] = &["afplay"];
#[cfg(not(target_os = "macos"))]
const PLAYERS: &[&str] = &["pw-play", "paplay", "aplay"];

/// O primeiro tocador que existe no `PATH`.
pub fn find() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    PLAYERS.iter().find_map(|name| std::env::split_paths(&path).map(|dir| dir.join(name)).find(|p| executable(p)))
}

fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// O que está tocando: qual interface, qual componente e em que revisão (outra revisão para o som).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playing {
    pub name: String,
    pub id: String,
    pub revision: u64,
}

#[derive(Default)]
pub struct Player {
    program: Option<PathBuf>,
    current: Option<(Playing, PathBuf, bool, Child)>,
}

/// O que aconteceu com o tocador desde a última olhada.
#[derive(Debug, PartialEq, Eq)]
pub enum Tick {
    /// Continua tocando (ou nada tocando).
    Same,
    /// Terminou (sem `loop`) ou parou com erro.
    Ended,
    /// O tocador saiu com erro: provavelmente não conhece o formato.
    Failed(String),
}

impl Player {
    pub fn new(program: Option<PathBuf>) -> Self {
        Self { program, current: None }
    }

    pub fn available(&self) -> bool {
        self.program.is_some()
    }

    pub fn playing(&self) -> Option<&Playing> {
        self.current.as_ref().map(|c| &c.0)
    }

    /// Toca `path` (para o que estiver tocando antes).
    pub fn play(&mut self, what: Playing, path: PathBuf, looped: bool) -> std::io::Result<()> {
        self.stop();
        let program = self.program.as_ref().ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
        let child = spawn(program, &path)?;
        self.current = Some((what, path, looped, child));
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some((_, _, _, mut child)) = self.current.take() {
            kill(&mut child);
        }
    }

    /// Olha o tocador: com `loop`, roda de novo quando termina bem.
    pub fn tick(&mut self) -> Tick {
        let Some((_, path, looped, child)) = &mut self.current else { return Tick::Same };
        let status = match child.try_wait() {
            Ok(None) => return Tick::Same,
            Ok(Some(status)) => status,
            Err(e) => {
                self.current = None;
                return Tick::Failed(e.to_string());
            }
        };
        if !status.success() {
            self.current = None;
            return Tick::Failed(status.to_string());
        }
        if *looped
            && let Some(program) = &self.program
            && let Ok(next) = spawn(program, path)
        {
            *child = next;
            return Tick::Same;
        }
        self.current = None;
        Tick::Ended
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

fn spawn(program: &Path, path: &Path) -> std::io::Result<Child> {
    let mut cmd = Command::new(program);
    cmd.arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    cmd.spawn()
}

/// Mata o grupo inteiro (o tocador e o que ele tiver aberto) e espera, para não deixar zumbi.
fn kill(child: &mut Child) {
    if let Ok(pid) = i32::try_from(child.id()) {
        // SAFETY: `kill` só manda um sinal; o grupo é o do filho (`process_group(0)` em `spawn`).
        unsafe { libc::kill(-pid, libc::SIGKILL) };
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn script(name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("idetea-player-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tocador");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn what() -> Playing {
        Playing { name: "sons".into(), id: "vento".into(), revision: 1 }
    }

    fn wait_for(player: &mut Player, want: Tick) {
        for _ in 0..200 {
            let t = player.tick();
            if t == want {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("o tocador não chegou a {want:?}");
    }

    #[test]
    fn plays_loops_and_stops() {
        // Um "tocador" que anota cada vez que roda e termina logo.
        let count = std::env::temp_dir().join(format!("idetea-player-vezes-{}", std::process::id()));
        let _ = std::fs::remove_file(&count);
        let program = script("loop", &format!("echo \"$1\" >> {}", count.display()));
        let mut player = Player::new(Some(program.clone()));
        player.play(what(), "som.wav".into(), true).unwrap();
        for _ in 0..300 {
            player.tick();
            if std::fs::read_to_string(&count).map_or(0, |t| t.lines().count()) >= 3 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(std::fs::read_to_string(&count).unwrap().lines().all(|l| l == "som.wav"));
        assert!(std::fs::read_to_string(&count).unwrap().lines().count() >= 3, "com loop, roda de novo");
        assert_eq!(player.playing(), Some(&what()));
        player.stop();
        assert_eq!(player.playing(), None);

        // Sem loop: termina sozinho.
        player.play(what(), "som.wav".into(), false).unwrap();
        wait_for(&mut player, Tick::Ended);
        assert_eq!(player.playing(), None);
        let _ = std::fs::remove_file(&count);
    }

    #[test]
    fn stop_kills_the_whole_group() {
        // O tocador abre um filho que fica rodando; parar mata os dois.
        let pidfile = std::env::temp_dir().join(format!("idetea-player-filho-{}", std::process::id()));
        let _ = std::fs::remove_file(&pidfile);
        let program = script("grupo", &format!("sleep 30 & echo $! > {}\nwait", pidfile.display()));
        let mut player = Player::new(Some(program));
        player.play(what(), "som.wav".into(), false).unwrap();
        let mut pid = None;
        for _ in 0..200 {
            if let Some(p) = std::fs::read_to_string(&pidfile).ok().and_then(|t| t.trim().parse::<i32>().ok()) {
                pid = Some(p);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let pid = pid.expect("o filho começou");
        player.stop();
        let mut gone = false;
        for _ in 0..200 {
            // SAFETY: sinal 0 só confere se o processo existe.
            if unsafe { libc::kill(pid, 0) } != 0 {
                gone = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(gone, "o filho do tocador morreu junto");
        let _ = std::fs::remove_file(&pidfile);
    }

    #[test]
    fn failure_and_missing_player() {
        let mut player = Player::new(Some(script("falha", "exit 3")));
        player.play(what(), "som.xyz".into(), true).unwrap();
        for _ in 0..200 {
            if let Tick::Failed(_) = player.tick() {
                assert_eq!(player.playing(), None, "erro não entra em loop");
                let mut none = Player::new(None);
                assert!(!none.available());
                assert!(none.play(what(), "som.wav".into(), false).is_err());
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("o erro não apareceu");
    }
}
