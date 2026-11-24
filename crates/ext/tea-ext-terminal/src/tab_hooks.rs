//! O estado que um agente manda para a aba onde roda (RFC 0024, M1.1): "pediu sua atenção", "terminou a vez".
//!
//! Cada aba do terminal recebe `IDETEA_TERMINAL=<pid da IDE>-<id do terminal>` no ambiente. Os hooks do Claude Code
//! que o mascote instala (`idetea mascot attention`, `idetea mascot celebrating`) herdam essa variável e, além do
//! mascote, gravam `.idetea/terminals/<valor>.json`. A IDE dona do pid lê, apaga e marca a aba. Sem a variável
//! (o agente rodando fora da IDE), nada muda.

use std::path::{Path, PathBuf};

/// A variável que cada aba recebe.
pub const ENV: &str = "IDETEA_TERMINAL";
/// A raiz do projeto da IDE dona da aba (desde a 0.18.0): diz à CLI se um pedido é para esta IDE.
pub const ROOT_ENV: &str = "IDETEA_ROOT";

/// A IDE de onde sai um pedido feito em `root` (RFC 0024, 2b, passo 2): o pid da aba onde a CLI roda, mas só se a
/// IDE dessa aba tem `root` aberto. Depois de um `cd` para outro projeto, ou numa aba de uma IDE sem `IDETEA_ROOT`
/// (até a 0.17.0), `None`: o pedido sairia com o pid de uma IDE que não olha esta pasta, e as outras o deixariam
/// esperando por ela.
pub fn origin_ide(root: &Path) -> Option<u32> {
    origin_ide_from(std::env::var(ENV).ok().as_deref(), std::env::var_os(ROOT_ENV).as_deref().map(Path::new), root)
}

fn origin_ide_from(tab: Option<&str>, ide_root: Option<&Path>, root: &Path) -> Option<u32> {
    let (pid, _) = parse_value(tab?)?;
    let same = |a: &Path, b: &Path| match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    (pid > 0 && same(ide_root?, root)).then_some(pid)
}

/// O que o agente disse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Pediu permissão ou uma resposta (hook `Notification`).
    Asked,
    /// Terminou a vez e espera você (hook `Stop`).
    TurnDone,
}

impl Signal {
    /// O estado do mascote que vira um sinal para a aba. Os outros (`working`…) não mudam a aba: a saída já mostra.
    pub fn from_mood(mood: &str) -> Option<Self> {
        match mood {
            "attention" => Some(Self::Asked),
            "celebrating" => Some(Self::TurnDone),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Asked => "asked",
            Self::TurnDone => "turn-done",
        }
    }

    fn parse(word: &str) -> Option<Self> {
        match word {
            "asked" => Some(Self::Asked),
            "turn-done" => Some(Self::TurnDone),
            _ => None,
        }
    }
}

pub fn dir(root: &Path) -> PathBuf {
    root.join(".idetea").join("terminals")
}

/// O valor da variável de uma aba.
pub fn env_value(pid: u32, id: u64) -> String {
    format!("{pid}-{id}")
}

/// `<pid>-<id>`, só dígitos: o valor vira nome de arquivo e não pode sair da pasta.
fn parse_value(value: &str) -> Option<(u32, u64)> {
    let (pid, id) = value.split_once('-')?;
    let digits = |s: &str| !s.is_empty() && s.len() <= 20 && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(pid) || !digits(id) {
        return None;
    }
    Some((pid.parse().ok()?, id.parse().ok()?))
}

/// Do lado do agente (`idetea mascot …`): grava o sinal para a aba de `value`. Um valor que não vale é ignorado.
pub fn write(root: &Path, value: &str, signal: Signal) -> std::io::Result<()> {
    if parse_value(value).is_none() {
        return Ok(());
    }
    let dir = dir(root);
    std::fs::create_dir_all(&dir)?;
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        let _ = std::fs::write(ignore, "*\n");
    }
    let body = serde_json::json!({"format": "idetea-terminal-signal", "version": 1, "signal": signal.as_str()});
    let tmp = dir.join(format!(".{value}.json.tmp"));
    std::fs::write(&tmp, body.to_string() + "\n")?;
    std::fs::rename(&tmp, dir.join(format!("{value}.json")))
}

/// Do lado da IDE: os sinais para as abas desta IDE (`pid`), por id do terminal. Cada arquivo lido é apagado.
pub fn take(root: &Path, pid: u32) -> Vec<(u64, Signal)> {
    let Ok(entries) = std::fs::read_dir(dir(root)) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(value) = name.to_str().and_then(|n| n.strip_suffix(".json")) else { continue };
        let Some((owner, id)) = parse_value(value) else { continue };
        if owner != pid {
            continue;
        }
        let path = entry.path();
        let signal = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| v.get("signal").and_then(|s| s.as_str()).and_then(Signal::parse));
        let _ = std::fs::remove_file(&path);
        if let Some(signal) = signal {
            out.push((id, signal));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_ide_only_for_the_ides_own_project() {
        let base = std::env::temp_dir().join(format!("idetea-origin-{}", std::process::id()));
        let (a, b) = (base.join("a"), base.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        assert_eq!(origin_ide_from(Some("4242-3"), Some(&a), &a), Some(4242));
        assert_eq!(origin_ide_from(Some("4242-3"), Some(&a.join(".")), &a), Some(4242), "o mesmo caminho escrito de outro jeito");
        assert_eq!(origin_ide_from(Some("4242-3"), Some(&a), &b), None, "cd para outro projeto");
        assert_eq!(origin_ide_from(Some("4242-3"), None, &a), None, "aba de uma IDE até a 0.17.0");
        assert_eq!(origin_ide_from(None, Some(&a), &a), None, "fora da IDE");
        assert_eq!(origin_ide_from(Some("x-3"), Some(&a), &a), None);
        assert_eq!(origin_ide_from(Some("0-3"), Some(&a), &a), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn round_trip_only_for_this_ide() {
        let root = std::env::temp_dir().join(format!("idetea-tab-hooks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        write(&root, "42-7", Signal::Asked).unwrap();
        write(&root, "43-1", Signal::TurnDone).unwrap();
        assert_eq!(take(&root, 42), vec![(7, Signal::Asked)]);
        assert!(take(&root, 42).is_empty(), "lido uma vez só");
        assert_eq!(take(&root, 43), vec![(1, Signal::TurnDone)]);
        assert!(dir(&root).join(".gitignore").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn values_that_could_escape_are_ignored() {
        let root = std::env::temp_dir().join(format!("idetea-tab-hooks-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for bad in ["../x", "1-2/../3", "a-1", "1-", "-1", "1-2-3", ""] {
            write(&root, bad, Signal::Asked).unwrap();
            assert!(parse_value(bad).is_none(), "{bad}");
        }
        assert!(!dir(&root).exists(), "nada foi gravado");
    }

    #[test]
    fn only_attention_and_celebrating_reach_the_tab() {
        assert_eq!(Signal::from_mood("attention"), Some(Signal::Asked));
        assert_eq!(Signal::from_mood("celebrating"), Some(Signal::TurnDone));
        assert_eq!(Signal::from_mood("working"), None);
    }
}
