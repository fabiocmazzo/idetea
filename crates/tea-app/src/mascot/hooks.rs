//! `idetea mascot setup-hooks` (RFC 0012 §2.3): hooks do Claude Code no projeto, sem mexer nos que já existem.
//!
//! O arquivo é `.claude/settings.local.json`. Cada evento ganha um comando só se ainda não houver um
//! `idetea mascot` daquele estado. O resto do JSON fica. Um arquivo que não é JSON não é reescrito.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// Caminho, relativo à raiz do projeto.
pub const REL_PATH: &str = ".claude/settings.local.json";

const EVENTS: &[(&str, &str, &str)] = &[
    (
        "PreToolUse",
        "idetea mascot working",
        r#"idetea mascot working --text "${TOOL_NAME:-$(sed -n 's/.*"tool_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')}"#,
    ),
    ("Notification", "idetea mascot attention", "idetea mascot attention"),
    ("Stop", "idetea mascot celebrating", "idetea mascot celebrating --ttl 5"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookChange {
    Added,
    Present,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookEdit {
    pub path: PathBuf,
    /// Texto novo, quando houve mudança.
    pub text: Option<String>,
    pub events: Vec<(&'static str, HookChange)>,
    pub created: bool,
}

pub fn plan(root: &Path, existing: Option<&str>) -> Result<HookEdit, String> {
    let path = root.join(REL_PATH);
    let (mut root_val, created) = match existing {
        None => (Value::Object(Map::new()), true),
        Some(text) if text.trim().is_empty() => (Value::Object(Map::new()), true),
        Some(text) => {
            let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
            if !v.is_object() {
                return Err(fl!("mascot-hooks-not-object"));
            }
            (v, false)
        }
    };
    let mut events = Vec::new();
    let mut changed = false;
    for (event, marker, command) in EVENTS {
        match ensure_event(root_val.as_object_mut().expect("objeto"), event, marker, command) {
            HookChange::Added => {
                changed = true;
                events.push((*event, HookChange::Added));
            }
            other => events.push((*event, other)),
        }
    }
    let text = changed.then(|| serde_json::to_string_pretty(&root_val).unwrap_or_default() + "\n");
    Ok(HookEdit { path, text, events, created })
}

/// Acrescenta o comando se o evento for uma lista (ou ainda não existir) e nenhum comando tiver `marker`.
fn ensure_event(root: &mut Map<String, Value>, event: &str, marker: &str, command: &str) -> HookChange {
    let hooks = root.entry("hooks").or_insert_with(|| Value::Object(Map::new()));
    if !hooks.is_object() {
        return HookChange::Skipped;
    }
    let hooks = hooks.as_object_mut().expect("objeto");
    let entry = hooks.entry(event).or_insert_with(|| Value::Array(Vec::new()));
    let Some(list) = entry.as_array_mut() else {
        return HookChange::Skipped;
    };
    if list_has_marker(list, marker) {
        return HookChange::Present;
    }
    list.push(serde_json::json!({
        "matcher": "*",
        "hooks": [{ "type": "command", "command": command }]
    }));
    HookChange::Added
}

fn list_has_marker(list: &[Value], marker: &str) -> bool {
    list.iter().any(|item| {
        item.get("hooks").and_then(Value::as_array).is_some_and(|hooks| {
            hooks.iter().any(|h| h.get("command").and_then(Value::as_str).is_some_and(|c| c.contains(marker)))
        }) || item.get("command").and_then(Value::as_str).is_some_and(|c| c.contains(marker))
    })
}

/// Lê, planeja e, sem `--dry-run`, grava só se algo mudou. Devolve as linhas para imprimir.
pub fn apply(root: &Path, dry_run: bool) -> Result<Vec<String>, String> {
    let path = root.join(REL_PATH);
    let existing = match std::fs::read_to_string(&path) {
        Ok(t) => Some(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let edit = plan(root, existing.as_deref()).map_err(|e| {
        if existing.is_some() { fl!("mascot-hooks-bad-json", path = path.display().to_string(), error = e) } else { e }
    })?;
    let mut lines = vec![fl!("mascot-hooks-where", path = edit.path.display().to_string())];
    for (event, change) in &edit.events {
        let line = match change {
            HookChange::Added if dry_run => fl!("mascot-hooks-would-add", event = (*event).to_string()),
            HookChange::Added => fl!("mascot-hooks-added", event = (*event).to_string()),
            HookChange::Present => fl!("mascot-hooks-present", event = (*event).to_string()),
            HookChange::Skipped => fl!("mascot-hooks-skipped", event = (*event).to_string()),
        };
        lines.push(line);
    }
    match (&edit.text, dry_run) {
        (Some(text), false) => {
            if let Some(dir) = edit.path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            std::fs::write(&edit.path, text).map_err(|e| format!("{}: {e}", edit.path.display()))?;
            let verb = if edit.created { fl!("mascot-hooks-created") } else { fl!("mascot-hooks-updated") };
            lines.push(verb);
        }
        (Some(_), true) => lines.push(fl!("mascot-hooks-dry")),
        (None, _) => lines.push(fl!("mascot-hooks-unchanged")),
    }
    lines.push(fl!("mascot-hooks-note"));
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_is_idempotent_and_keeps_other_hooks() {
        let root = Path::new("/proj");
        let first = plan(root, None).unwrap();
        assert!(first.created);
        assert!(first.text.as_ref().unwrap().contains("PreToolUse"));
        assert!(first.text.as_ref().unwrap().contains("idetea mascot celebrating --ttl 5"));
        let again = plan(root, first.text.as_deref()).unwrap();
        assert!(again.text.is_none(), "a segunda vez não reescreve: {:?}", again.events);
        assert!(again.events.iter().all(|(_, c)| *c == HookChange::Present));

        let foreign = r#"{
            "permissions": {"allow": ["Bash"]},
            "hooks": {
                "PreToolUse": [
                    {"matcher": "Bash", "hooks": [{"type": "command", "command": "echo oi"}]}
                ],
                "PostToolUse": "deixa"
            }
        }"#;
        let edit = plan(root, Some(foreign)).unwrap();
        let v: Value = serde_json::from_str(edit.text.as_ref().unwrap()).unwrap();
        assert_eq!(v["permissions"]["allow"][0], "Bash");
        let pre = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 2, "o hook de Bash fica, o nosso entra depois");
        assert_eq!(pre[0]["hooks"][0]["command"], "echo oi");
        assert!(pre[1]["hooks"][0]["command"].as_str().unwrap().contains("idetea mascot working"));
        assert_eq!(v["hooks"]["PostToolUse"], "deixa", "um evento que não é lista fica como está");
        assert!(edit.events.iter().all(|(e, c)| {
            matches!((e, c), (&"PreToolUse" | &"Notification" | &"Stop", HookChange::Added))
        }));

        let custom = r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"idetea mascot working --text custom"}]}]}}"#;
        let kept = plan(root, Some(custom)).unwrap();
        assert_eq!(kept.events.iter().find(|(e, _)| *e == "PreToolUse").unwrap().1, HookChange::Present);
        let text = kept.text.as_ref().unwrap();
        assert!(text.contains("custom"), "não duplica nem troca o comando que a pessoa escreveu");
        assert_eq!(text.matches("idetea mascot working").count(), 1);

        let bad = plan(root, Some("{"));
        assert!(bad.is_err());
        let not_obj = plan(root, Some("[]"));
        assert!(not_obj.is_err());
    }
}
