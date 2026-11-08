//! A pasta `.idetea/ui/` no disco: listar e ler os documentos com cuidado, e gravar as respostas.
//!
//! Regra que evita brigas entre processos: cada arquivo tem um só escritor. O agente escreve e apaga
//! `ui/<nome>.json`; a IDE escreve `ui/responses/<nome>/*.json` e o agente apaga cada resposta depois de ler
//! (é a confirmação). Nada aqui segue symlinks, e nenhum caminho vindo do JSON é usado para abrir arquivo.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use tea_declarative_ui::{Document, Limits, Media, MediaLimits, Response, media_stamps, parse};

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

/// O formato completo, da versão desta IDE. Fica em `.idetea/ui/FORMAT.md` para o agente ler sem internet (o
/// repositório pode ser privado) e sem pegar uma versão diferente da IDE que está aberta.
pub const FORMAT_DOC: &str = include_str!("../../../../docs/interfaces/formato.md");
/// O JSON Schema do documento, ao lado do `FORMAT.md` (para o agente validar antes de publicar).
pub const SCHEMA: &str = include_str!("../../../../docs/interfaces/idetea-ui.schema.json");
const FORMAT_HEADER: &str = "<!-- Written by the TEA IDE (the version that is open): the full interface format. Do not edit; the IDE rewrites it. -->\n\n";

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
    write_if_changed(&ui.join("FORMAT.md"), &format!("{FORMAT_HEADER}{}", format_for_project(FORMAT_DOC)))?;
    write_if_changed(&ui.join("idetea-ui.schema.json"), SCHEMA)?;
    if let Ok(exe) = std::env::current_exe()
        && exe.file_stem().is_some_and(|n| n == "idetea")
    {
        write_check_script(&ui.join("check"), &exe)?;
    }
    Ok(())
}

/// `.idetea/ui/check`: roda `idetea ui check` com o executável desta IDE (o `idetea` do `PATH` pode ser outra
/// versão, ou uma antiga que abriria uma janela com arquivos chamados `ui` e `check`).
fn write_check_script(path: &Path, exe: &Path) -> std::io::Result<()> {
    let quoted = exe.to_string_lossy().replace('\'', r"'\''");
    let script = format!("#!/bin/sh\n# Written by the TEA IDE: checks interface documents like the panel reads them.\nexec '{quoted}' ui check \"$@\"\n");
    write_if_changed(path, &script)?;
    #[cfg(unix)]
    if !is_link(path) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(windows)]
    {
        let cmd_path = path.with_extension("cmd");
        let cmd_script = format!("@echo off\r\n\"{}\" ui check %*\r\n", exe.display());
        write_if_changed(&cmd_path, &cmd_script)?;
    }
    Ok(())
}

/// O formato como fica em `.idetea/ui/`: os links para o repositório da IDE (`../rfc/…`, `../../examples/…`) não
/// existem no projeto e viram texto com o caminho no repositório; `agentes.md` aponta para o `AGENTS.md` do
/// projeto (onde o "ensinar" grava as instruções). O schema está ao lado, e as âncoras continuam.
pub fn format_for_project(doc: &str) -> String {
    let mut out = String::with_capacity(doc.len());
    let mut rest = doc;
    while let Some(i) = rest.find("](") {
        let open = rest[..i].rfind('[');
        let close = rest[i + 2..].find(')').map(|c| i + 2 + c);
        let link = open.zip(close).filter(|&(o, c)| !rest[o..c].contains('\n'));
        // Não é link (um `](` no meio de código, por exemplo): segue adiante.
        let Some((open, close)) = link else {
            out.push_str(&rest[..i + 2]);
            rest = &rest[i + 2..];
            continue;
        };
        let (label, target) = (&rest[open + 1..i], &rest[i + 2..close]);
        out.push_str(&rest[..open]);
        if target == "agentes.md" {
            out.push_str(&format!("[{label}](../../AGENTS.md)"));
        } else if let Some(repo) = target.strip_prefix("../../").or_else(|| target.strip_prefix("../").map(|_| "")) {
            let path = if repo.is_empty() { format!("docs/{}", &target[3..]) } else { repo.to_string() };
            if label.trim_matches('`') == path {
                out.push_str(&format!("{label} (no repositório da IDE TEA)"));
            } else {
                out.push_str(&format!("{label} (`{path}`, no repositório da IDE TEA)"));
            }
        } else {
            out.push_str(&rest[open..=close]);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Grava só se o conteúdo mudou. Um symlink no lugar não é seguido (poderia apontar para fora do projeto).
fn write_if_changed(path: &Path, text: &str) -> std::io::Result<()> {
    let symlink = std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink());
    if !symlink && std::fs::read_to_string(path).ok().as_deref() != Some(text) {
        std::fs::write(path, text)?;
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
    /// As imagens do documento (`image`, `gallery`), já lidas.
    pub media: Media,
    /// Hash do conteúdo do arquivo: o mesmo nome recriado com outro conteúdo é outra interface.
    pub hash: u64,
    /// O arquivo como estava (só de um documento válido), para o histórico guardar a revisão respondida.
    pub raw: Option<std::sync::Arc<[u8]>>,
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
            let tracked = tracked.contains(&rel);
            let (doc, hash, raw) = read_document_raw(&path, &name, limits);
            let raw = doc.is_ok().then(|| std::sync::Arc::from(raw));
            // Um documento versionado é ignorado: nem as imagens dele são lidas.
            let media = match &doc {
                Ok(d) if !tracked => Media::load(root, d, &MediaLimits::default()),
                _ => Media::default(),
            };
            SurfaceFile { tracked, doc, pending: pending(&ui, &name), media, name, hash, raw }
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

pub(crate) fn read_document(path: &Path, name: &str, limits: &Limits) -> (Result<Document, String>, u64) {
    let (doc, hash, _) = read_document_raw(path, name, limits);
    (doc, hash)
}

/// Como `read_document`, devolvendo também os bytes lidos.
fn read_document_raw(path: &Path, name: &str, limits: &Limits) -> (Result<Document, String>, u64, Vec<u8>) {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => return (Err(e.to_string()), 0, Vec::new()),
    };
    if !meta.file_type().is_file() {
        return (Err(fl!("ui-not-a-file")), 0, Vec::new());
    }
    let mut bytes = Vec::new();
    // Lê no máximo um byte além do limite: arquivo que cresce durante a leitura também é recusado.
    if let Err(e) = std::fs::File::open(path).and_then(|f| f.take(limits.max_bytes as u64 + 1).read_to_end(&mut bytes)) {
        return (Err(e.to_string()), 0, Vec::new());
    }
    let hash = fnv1a(&bytes);
    let doc = parse(&bytes, limits).and_then(|doc| match &doc.surface {
        Some(s) if s != name => Err(fl!("ui-surface-mismatch", found = s.clone(), expected = name)),
        _ => Ok(doc),
    });
    (doc, hash, bytes)
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3))
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
    tracked_under(root, ".idetea/ui")
}

/// Arquivos versionados no git debaixo de `dir` (relativo à raiz).
pub(crate) fn tracked_under(root: &Path, dir: &str) -> BTreeSet<String> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "--", dir])
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

/// Um agente esperando a resposta de `<nome>.json` mantém `<nome>.wait` com a data em dia (`touch` a cada
/// segundo). Mais velho que isto: ele parou de esperar.
pub const WAIT_FRESH: Duration = Duration::from_secs(5);

/// Se o agente de uma interface espera a resposta, pelo `<nome>.wait`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wait {
    /// O agente está esperando (o arquivo foi tocado há pouco).
    Waiting,
    /// Parou de esperar há tantos minutos.
    Stopped { minutes: u64 },
}

/// Os `<nome>.wait` da pasta, pela data. Sem ocultos, symlinks e nomes inválidos.
pub fn wait_files(root: &Path) -> BTreeMap<String, SystemTime> {
    let Ok(entries) = std::fs::read_dir(ui_dir(root)) else { return BTreeMap::new() };
    entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            let file = e.file_name().into_string().ok()?;
            let name = file.strip_suffix(".wait").filter(|n| valid_name(n))?;
            Some((name.to_string(), e.metadata().ok()?.modified().ok()?))
        })
        .collect()
}

/// Como está cada agente que usa o `.wait`, agora.
pub fn waits(root: &Path, now: SystemTime) -> BTreeMap<String, Wait> {
    wait_files(root)
        .into_iter()
        .map(|(name, t)| {
            let age = now.duration_since(t).unwrap_or_default();
            let wait = if age < WAIT_FRESH { Wait::Waiting } else { Wait::Stopped { minutes: age.as_secs() / 60 } };
            (name, wait)
        })
        .collect()
}

/// Assinatura barata da pasta (nomes, tamanhos e datas): se não mudou, não precisa ler de novo.
/// O que muda quando é preciso ler a pasta de novo: a pasta e as respostas ([`signature`]) e as imagens que os
/// documentos da última leitura usam (regerar um sprite atualiza o painel).
pub fn signature_with_media(root: &Path, last: &[SurfaceFile]) -> Vec<(PathBuf, u64, Option<SystemTime>)> {
    let mut out = signature(root);
    for f in last {
        if let Ok(doc) = &f.doc {
            out.extend(media_stamps(root, doc));
        }
    }
    out
}

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
            // O `.wait` muda a cada segundo enquanto o agente espera: tem observação própria ([`waits`]).
            if e.path().extension().is_some_and(|x| x == "wait") {
                continue;
            }
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
        let format = std::fs::read_to_string(ui.join("FORMAT.md")).unwrap();
        assert!(format.ends_with(&format_for_project(FORMAT_DOC)) && format.contains("## A resposta"), "o formato da versão desta IDE");
        assert_eq!(std::fs::read_to_string(ui.join("idetea-ui.schema.json")).unwrap(), SCHEMA);
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

    #[test]
    fn image_changes_are_noticed() {
        let root = tmp_root("imagens");
        prepare(&root).unwrap();
        std::fs::write(root.join("sprite.png"), b"um").unwrap();
        let doc = r#"{"format":"idetea-ui","version":1,"components":[{"id":"i","component":"image","path":"sprite.png"}]}"#;
        std::fs::write(ui_dir(&root).join("ver.json"), doc).unwrap();
        let files = scan(&root, &Limits::default());
        assert!(files[0].media.picture("sprite.png").is_some(), "a imagem foi lida (aqui, com erro: não é PNG)");
        let before = signature_with_media(&root, &files);
        std::fs::write(root.join("sprite.png"), b"outro maior").unwrap();
        assert_ne!(signature_with_media(&root, &files), before, "regerar a imagem pede uma leitura nova");
        assert_eq!(signature(&root), signature(&root), "a pasta em si não mudou");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn format_links_work_in_the_project() {
        let doc = format_for_project(FORMAT_DOC);
        assert!(doc.contains("`docs/rfc/0001-interfaces-declarativas.md` (no repositório da IDE TEA)"));
        assert!(doc.contains("RFC 0003 (`docs/rfc/0003-interfaces-pelo-uso.md`, no repositório da IDE TEA)"));
        assert!(doc.contains("`examples/interfaces/pergunta.sh` (no repositório da IDE TEA)"));
        assert!(doc.contains("[`agentes.md`](../../AGENTS.md)"));
        assert!(doc.contains("[`idetea-ui.schema.json`](idetea-ui.schema.json)"), "o schema fica ao lado");
        assert!(doc.contains("](#imagens)"), "âncoras continuam");
        let left: Vec<&str> = doc.lines().filter(|l| l.replace("](../../AGENTS.md)", "").contains("](../")).collect();
        assert!(left.is_empty(), "nenhum link para fora do projeto: {left:?}");
        assert_eq!(format_for_project("a [b](c) d"), "a [b](c) d");
    }

    #[test]
    fn check_script_runs_this_ide() {
        let root = tmp_root("check-script");
        let script = root.join("check");
        write_check_script(&script, Path::new("/opt/it's/idetea")).unwrap();
        let text = std::fs::read_to_string(&script).unwrap();
        assert!(text.contains(r#"exec '/opt/it'\''s/idetea' ui check "$@""#), "{text}");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&script).unwrap().permissions().mode() & 0o777, 0o755);
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_wait_file_says_who_is_waiting() {
        let root = std::env::temp_dir().join(format!("idetea-wait-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let ui = ui_dir(&root);
        std::fs::create_dir_all(&ui).unwrap();
        let ago = |f: &str, secs: u64| {
            std::fs::write(ui.join(f), "").unwrap();
            let t = SystemTime::now() - Duration::from_secs(secs);
            std::fs::File::options().write(true).open(ui.join(f)).unwrap().set_modified(t).unwrap();
        };
        ago("agora.wait", 1);
        ago("parou.wait", 3 * 60 + 5);
        ago("Ruim.wait", 1); // nome inválido: ignorado
        let w = waits(&root, SystemTime::now());
        assert_eq!(w.get("agora"), Some(&Wait::Waiting));
        assert_eq!(w.get("parou"), Some(&Wait::Stopped { minutes: 3 }));
        assert_eq!(w.len(), 2);
        let before = signature(&root);
        ago("agora.wait", 0);
        assert_eq!(signature(&root), before, "tocar o .wait não faz ler a pasta de novo");
        let _ = std::fs::remove_dir_all(&root);
    }
}
