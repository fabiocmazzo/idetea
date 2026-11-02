//! A pasta `.idetea/ui/` no disco: listar e ler os documentos com cuidado, e gravar as respostas.
//!
//! Regra que evita brigas entre processos: cada arquivo tem um só escritor. O agente escreve e apaga
//! `ui/<nome>.json`; a IDE escreve `ui/responses/<nome>/*.json` e o agente apaga cada resposta depois de ler
//! (é a confirmação). Nada aqui segue symlinks, e nenhum caminho vindo do JSON é usado para abrir arquivo.

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use tea_declarative_ui::{Document, Limits, Response, parse};

/// `.idetea/ui` dentro do projeto.
pub fn ui_dir(root: &Path) -> PathBuf {
    root.join(".idetea").join("ui")
}

fn responses_dir(ui: &Path) -> PathBuf {
    ui.join("responses")
}

/// Nome de interface válido: minúsculas, números, `_` e `-`, começando por letra ou número (até 64).
pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name.len() <= 64
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// É um symlink (ou não deu para saber)?
fn is_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path).map_or(true, |m| m.file_type().is_symlink())
}

/// Cria `.idetea/ui/` e um `.gitignore` com `*` dentro (respostas podem ter o que o usuário digitou; não vão
/// para o git mesmo se o projeto não ignorar a pasta). Recusa se `.idetea` ou `ui` for symlink.
pub fn prepare(root: &Path) -> std::io::Result<()> {
    let ui = ui_dir(root);
    for dir in [root.join(".idetea"), ui.clone()] {
        if dir.exists() && is_link(&dir) {
            return Err(std::io::Error::other(format!("{}: symlink", dir.display())));
        }
    }
    std::fs::create_dir_all(&ui)?;
    let ignore = ui.join(".gitignore");
    if !ignore.exists() && !is_link(&ui) {
        std::fs::write(ignore, "# Criado pela IDE TEA: interfaces e respostas não vão para o git.\n*\n")?;
    }
    Ok(())
}

/// Uma interface na pasta, como a thread de observação viu.
#[derive(Debug, Clone)]
pub struct SurfaceFile {
    pub name: String,
    pub doc: Result<Document, String>,
    /// O arquivo está versionado no git: ignorado por segurança (pode ter vindo de um repositório clonado).
    pub tracked: bool,
    /// Respostas ainda não apagadas pelo agente: (`seq`, `event`).
    pub pending: Vec<(u64, String)>,
}

/// Lê a pasta inteira. Roda fora da thread da interface.
pub fn scan(root: &Path, limits: &Limits) -> Vec<SurfaceFile> {
    let ui = ui_dir(root);
    if is_link(&ui) {
        return Vec::new();
    }
    clean_tmp(&ui);
    let tracked = tracked_files(root);
    let mut out: Vec<SurfaceFile> = list_json(&ui)
        .into_iter()
        .map(|(name, path)| {
            let rel = format!(".idetea/ui/{name}.json");
            SurfaceFile {
                tracked: tracked.contains(&rel),
                doc: read_document(&path, &name, limits),
                pending: pending(&ui, &name),
                name,
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    clean_responses(&ui, &out);
    out
}

/// A pasta de respostas de uma interface já fechada (o agente apagou o documento e leu tudo) sai.
/// `remove_dir` só apaga pasta vazia: uma resposta ainda não lida nunca se perde.
fn clean_responses(ui: &Path, open: &[SurfaceFile]) {
    let dir = responses_dir(ui);
    if is_link(&dir) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    for e in entries.flatten() {
        let name = e.file_name();
        let closed = name.to_str().is_some_and(|n| !open.iter().any(|f| f.name == n));
        if closed && e.file_type().is_ok_and(|t| t.is_dir()) {
            let _ = std::fs::remove_dir(e.path());
        }
    }
}

/// `<nome>.json` de uma pasta (sem ocultos, temporários, symlinks e nomes inválidos).
fn list_json(dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            let file = e.file_name().into_string().ok()?;
            let name = file.strip_suffix(".json")?;
            valid_name(name).then(|| (name.to_string(), e.path()))
        })
        .collect()
}

fn read_document(path: &Path, name: &str, limits: &Limits) -> Result<Document, String> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.file_type().is_file() {
        return Err(fl!("ui-not-a-file"));
    }
    let mut bytes = Vec::new();
    // Lê no máximo um byte além do limite: arquivo que cresce durante a leitura também é recusado.
    std::fs::File::open(path)
        .and_then(|f| f.take(limits.max_bytes as u64 + 1).read_to_end(&mut bytes))
        .map_err(|e| e.to_string())?;
    let doc = parse(&bytes, limits)?;
    match &doc.surface {
        Some(s) if s != name => Err(fl!("ui-surface-mismatch", found = s.clone(), expected = name)),
        _ => Ok(doc),
    }
}

/// Respostas esperando o agente: (`seq`, `event`) pelo nome `000007-<event>.json`.
pub fn pending(ui: &Path, name: &str) -> Vec<(u64, String)> {
    let dir = responses_dir(ui).join(name);
    if is_link(&dir) {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out: Vec<(u64, String)> = entries
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().into_string().ok()?;
            let stem = file.strip_suffix(".json")?;
            let (seq, event) = stem.split_once('-')?;
            Some((seq.parse().ok()?, event.to_string()))
        })
        .collect();
    out.sort();
    out
}

/// Grava a resposta: arquivo novo (nunca segue um link deixado lá) e `rename` para o nome final, para o agente
/// nunca ver um arquivo pela metade.
pub fn write_response(root: &Path, response: &Response) -> std::io::Result<PathBuf> {
    let ui = ui_dir(root);
    let responses = responses_dir(&ui);
    let dir = responses.join(&response.surface);
    if !valid_name(&response.surface) {
        return Err(std::io::Error::other("nome inválido"));
    }
    for d in [&ui, &responses, &dir] {
        if d.exists() && is_link(d) {
            return Err(std::io::Error::other(format!("{}: symlink", d.display())));
        }
    }
    std::fs::create_dir_all(&dir)?;
    let name = tea_declarative_ui::file_name(response.seq, &response.event);
    let path = dir.join(&name);
    // Só a IDE escreve aqui; mesmo assim, uma resposta nunca substitui outra.
    if std::fs::symlink_metadata(&path).is_ok() {
        return Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
    }
    let tmp = dir.join(format!(".{name}.tmp"));
    let text = serde_json::to_string_pretty(response).map_err(std::io::Error::other)?;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    file.write_all(text.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}

/// Apaga temporários esquecidos há mais de uma hora (queda no meio de uma escrita, nossa ou do agente).
fn clean_tmp(ui: &Path) {
    let old = |p: &Path| {
        std::fs::symlink_metadata(p)
            .and_then(|m| m.modified())
            .is_ok_and(|t| SystemTime::now().duration_since(t).unwrap_or_default() > Duration::from_secs(3600))
    };
    let mut dirs = vec![ui.to_path_buf()];
    if let Ok(entries) = std::fs::read_dir(responses_dir(ui)) {
        dirs.extend(entries.flatten().filter(|e| e.file_type().is_ok_and(|t| t.is_dir())).map(|e| e.path()));
    }
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().is_some_and(|x| x == "tmp") && e.file_type().is_ok_and(|t| t.is_file()) && old(&path) {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

/// Arquivos de `.idetea/ui/` versionados no git (caminhos relativos à raiz). Sem git ou fora de um
/// repositório: nenhum.
fn tracked_files(root: &Path) -> BTreeSet<String> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "--", ".idetea/ui"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    match out {
        Ok(o) if o.status.success() => o
            .stdout
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect(),
        _ => BTreeSet::new(),
    }
}

/// Assinatura barata da pasta (nomes, tamanhos e datas): se não mudou, não precisa ler de novo.
pub fn signature(root: &Path) -> Vec<(PathBuf, u64, Option<SystemTime>)> {
    let ui = ui_dir(root);
    let mut dirs = vec![ui.clone()];
    if let Ok(entries) = std::fs::read_dir(responses_dir(&ui)) {
        dirs.extend(entries.flatten().map(|e| e.path()));
    }
    let mut out = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            if let Ok(m) = e.metadata() {
                out.push((e.path(), m.len(), m.modified().ok()));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("idetea-ui-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    const DOC: &str = r#"{"format":"idetea-ui","version":1,"components":[{"id":"t","component":"text","text":"oi"}]}"#;

    #[test]
    fn names() {
        assert!(valid_name("revisar-migracao") && valid_name("a1_b"));
        assert!(!valid_name("") && !valid_name("-a") && !valid_name("A") && !valid_name("../x") && !valid_name("a.b"));
    }

    #[test]
    fn scan_reads_and_protects() {
        let root = tmp_root("scan");
        prepare(&root).unwrap();
        let ui = ui_dir(&root);
        assert!(std::fs::read_to_string(ui.join(".gitignore")).unwrap().contains('*'));
        std::fs::write(ui.join("boa.json"), DOC).unwrap();
        std::fs::write(ui.join("quebrada.json"), "{ nada").unwrap();
        std::fs::write(ui.join("outro-nome.json"), DOC.replace("{\"format\"", "{\"surface\":\"x\",\"format\"")).unwrap();
        std::fs::write(ui.join("Maiuscula.json"), DOC).unwrap(); // nome inválido: ignorado
        std::fs::write(ui.join(".oculta.json"), DOC).unwrap();
        std::fs::write(ui.join("meio.json.tmp"), "{").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(ui.join("boa.json"), ui.join("link.json")).unwrap();

        let files = scan(&root, &Limits::default());
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["boa", "outro-nome", "quebrada"], "sem symlink, oculto, temporário ou nome inválido");
        assert!(files[0].doc.is_ok());
        assert!(files[1].doc.as_ref().unwrap_err().contains("outro-nome"), "surface diferente do nome do arquivo");
        assert!(files[2].doc.is_err());
        assert!(files.iter().all(|f| !f.tracked && f.pending.is_empty()));

        let small = Limits { max_bytes: 10, ..Limits::default() };
        assert!(scan(&root, &small)[0].doc.is_err(), "grande demais");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn responses_are_atomic_and_listed_in_order() {
        let root = tmp_root("resp");
        prepare(&root).unwrap();
        let r = |seq, event: &str| {
            Response::new("pergunta", 1, seq, event.into(), "ok".into(), "b", "2026-11-01T00:00:00Z".into(), Default::default())
        };
        let p2 = write_response(&root, &r(2, "e2")).unwrap();
        write_response(&root, &r(10, "e10")).unwrap();
        assert!(p2.ends_with("responses/pergunta/000002-e2.json"));
        let back: Response = serde_json::from_str(&std::fs::read_to_string(&p2).unwrap()).unwrap();
        assert_eq!(back, r(2, "e2"));
        assert_eq!(pending(&ui_dir(&root), "pergunta"), [(2, "e2".to_string()), (10, "e10".to_string())]);
        assert!(write_response(&root, &r(2, "e2")).is_err(), "nunca sobrescreve");
        std::fs::remove_file(p2).unwrap(); // o agente leu
        assert_eq!(pending(&ui_dir(&root), "pergunta").len(), 1);
        let dir = root.join(".idetea/ui/responses/pergunta");
        assert!(std::fs::read_dir(&dir).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().ends_with(".tmp")));
        scan(&root, &Limits::default()); // sem documento, mas com uma resposta não lida: a pasta fica
        assert!(dir.exists());
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            std::fs::remove_file(e.path()).unwrap();
        }
        scan(&root, &Limits::default()); // fechada e vazia: sai
        assert!(!dir.exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn tracked_documents_are_flagged() {
        let root = tmp_root("git");
        let git = |args: &[&str]| Command::new("git").args(args).current_dir(&root).output().is_ok_and(|o| o.status.success());
        if !git(&["init", "-q"]) {
            return; // sem git na máquina
        }
        std::fs::create_dir_all(ui_dir(&root)).unwrap();
        std::fs::write(ui_dir(&root).join("clonada.json"), DOC).unwrap();
        assert!(git(&["add", "-f", ".idetea/ui/clonada.json"]));
        prepare(&root).unwrap();
        std::fs::write(ui_dir(&root).join("local.json"), DOC).unwrap();
        let files = scan(&root, &Limits::default());
        assert_eq!(files.iter().map(|f| (f.name.as_str(), f.tracked)).collect::<Vec<_>>(), [("clonada", true), ("local", false)]);
        let _ = std::fs::remove_dir_all(&root);
    }
}
