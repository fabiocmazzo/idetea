//! "Consertar este projeto" (`idetea ui fix`): acha o que ficou para trás em `.idetea/ui/` e nas instruções, e
//! conserta só depois de mostrar. Uma resposta que o agente não leu nunca é apagada: pode ser a única cópia de uma
//! decisão do usuário.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::{store, teach};

/// Documento sem resposta por ler, cujo agente parou de esperar há mais que isto: abandonado.
const STOPPED_FOR: Duration = Duration::from_secs(10 * 60);
/// Documento sem `.wait` (o agente não avisa que espera), sem mudar há mais que isto: abandonado.
const UNTOUCHED_FOR: Duration = Duration::from_secs(24 * 3600);
/// Temporário (`.tmp`) mais velho que isto: o agente caiu no meio da escrita.
const TMP_FOR: Duration = Duration::from_secs(10 * 60);

/// Uma coisa achada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// Interface que ninguém mais espera (o agente parou ou sumiu), sem resposta por ler: apagar o documento.
    Abandoned { name: String, minutes: u64 },
    /// Respostas que o agente não leu: só avisa (nunca apaga).
    Unread { name: String, count: usize },
    /// Temporário esquecido, ou `.wait` de uma interface que não existe mais: apagar.
    Leftover { path: String },
    /// O `.idetea/ui/check` chama um executável que não existe mais: regravar com este.
    BrokenCheck { target: String },
    /// Instruções do `AGENTS.md`/`CLAUDE.md` mais velhas que as desta IDE: regravar (o mesmo que "ensinar").
    Instructions { file: String, state: teach::Stale },
}

impl Problem {
    /// Dá para consertar daqui (os outros só são mostrados).
    pub fn fixable(&self) -> bool {
        match self {
            Problem::Unread { .. } => false,
            Problem::Instructions { state, .. } => !matches!(state, teach::Stale::Newer(_)),
            Problem::BrokenCheck { .. } => this_ide().is_some(),
            _ => true,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Problem::Abandoned { name, minutes } => {
                // Até duas horas em minutos; depois, em horas.
                let since = if *minutes < 120 { format!("{minutes} min") } else { format!("{} h", minutes / 60) };
                fl!("ui-fix-abandoned", path = format!(".idetea/ui/{name}.json"), since = since)
            }
            Problem::Unread { name, count } => {
                let count = *count;
                fl!("ui-fix-unread", path = format!(".idetea/ui/responses/{name}/"), count = count)
            }
            Problem::Leftover { path } => fl!("ui-fix-leftover", path = path.clone()),
            Problem::BrokenCheck { target } => fl!("ui-fix-check", target = target.clone()),
            Problem::Instructions { file, state } => match state {
                teach::Stale::Older(v) => fl!("ui-fix-instructions-older", file = file.clone(), found = v.clone()),
                teach::Stale::Unversioned => fl!("ui-fix-instructions-unversioned", file = file.clone()),
                teach::Stale::Newer(v) => fl!("ui-fix-instructions-newer", file = file.clone(), found = v.clone()),
            },
        }
    }
}

/// O executável desta IDE, quando é uma IDE (nos testes é o binário de teste: não serve para o `check`).
fn this_ide() -> Option<PathBuf> {
    std::env::current_exe().ok().filter(|e| e.file_stem().is_some_and(|n| n == "idetea"))
}

fn age(path: &Path, now: SystemTime) -> Option<Duration> {
    let modified = std::fs::symlink_metadata(path).ok()?.modified().ok()?;
    Some(now.duration_since(modified).unwrap_or_default())
}

/// O que há para consertar no projeto `root`, agora.
pub fn find(root: &Path, now: SystemTime) -> Vec<Problem> {
    let ui = store::ui_dir(root);
    let mut out = Vec::new();
    let waits = store::wait_files(root);
    let docs: Vec<String> = std::fs::read_dir(&ui)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| e.file_name().into_string().ok()?.strip_suffix(".json").filter(|n| store::valid_name(n)).map(str::to_string))
        .collect();
    let mut sorted = docs.clone();
    sorted.sort();
    for name in &sorted {
        let unread = store::pending(&ui, name).len();
        if unread > 0 {
            out.push(Problem::Unread { name: name.clone(), count: unread });
            continue;
        }
        let abandoned = match waits.get(name) {
            Some(t) => now.duration_since(*t).ok().filter(|a| *a >= STOPPED_FOR),
            None => age(&ui.join(format!("{name}.json")), now).filter(|a| *a >= UNTOUCHED_FOR),
        };
        if let Some(a) = abandoned {
            out.push(Problem::Abandoned { name: name.clone(), minutes: a.as_secs() / 60 });
        }
    }
    // Respostas de uma interface que já não existe (o agente apagou o documento sem ler tudo).
    if let Ok(entries) = std::fs::read_dir(ui.join("responses")) {
        let mut orphans: Vec<String> = entries
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| store::valid_name(n) && !docs.contains(n))
            .collect();
        orphans.sort();
        for name in orphans {
            let count = store::pending(&ui, &name).len();
            if count > 0 {
                out.push(Problem::Unread { name, count });
            }
        }
    }
    for name in waits.keys().filter(|n| !docs.contains(n)) {
        out.push(Problem::Leftover { path: format!(".idetea/ui/{name}.wait") });
    }
    if let Ok(entries) = std::fs::read_dir(&ui) {
        let mut tmps: Vec<String> = entries
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "tmp") && e.file_type().is_ok_and(|t| t.is_file()))
            .filter(|e| age(&e.path(), now).is_some_and(|a| a >= TMP_FOR))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        tmps.sort();
        out.extend(tmps.into_iter().map(|f| Problem::Leftover { path: format!(".idetea/ui/{f}") }));
    }
    if let Some(target) = check_target(&ui.join("check"))
        && !Path::new(&target).exists()
    {
        out.push(Problem::BrokenCheck { target });
    }
    let ours = teach::version(teach::INSTRUCTIONS);
    for (file, state) in teach::stale(root, &ours) {
        out.push(Problem::Instructions { file, state });
    }
    out
}

/// O executável que o `.idetea/ui/check` chama (a linha `exec '<caminho>' ui check`).
fn check_target(script: &Path) -> Option<String> {
    let text = std::fs::read_to_string(script).ok()?;
    let line = text.lines().find_map(|l| l.strip_prefix("exec '"))?;
    let end = line.find("' ui check")?;
    Some(line[..end].replace(r"'\''", "'"))
}

/// Conserta o que dá (os [`Problem::fixable`]). Devolve o que foi feito, uma linha por item.
pub fn apply(root: &Path, problems: &[Problem]) -> std::io::Result<Vec<String>> {
    let ui = store::ui_dir(root);
    let mut done = Vec::new();
    let mut teach_files = Vec::new();
    for p in problems.iter().filter(|p| p.fixable()) {
        match p {
            Problem::Abandoned { name, .. } => {
                remove(&ui.join(format!("{name}.json")))?;
                remove(&ui.join(format!("{name}.wait")))?;
            }
            Problem::Leftover { path } => remove(&root.join(path))?,
            Problem::BrokenCheck { .. } => store::prepare(root)?,
            Problem::Instructions { file, .. } => teach_files.push(file.clone()),
            Problem::Unread { .. } => continue,
        }
        done.push(p.describe());
    }
    if !teach_files.is_empty() {
        // Só os arquivos velhos: um bloco de uma IDE mais nova não volta atrás.
        let plan: Vec<teach::Change> =
            teach::plan(root, teach::INSTRUCTIONS)?.into_iter().filter(|c| c.newer.is_none() && teach_files.contains(&c.name)).collect();
        teach::apply(&plan)?;
    }
    Ok(done)
}

/// Apaga um arquivo comum (um symlink só perde o link, nunca o alvo); já apagado não é erro.
fn remove(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// `idetea ui fix [--apply]`: mostra o que acharia; com `--apply`, conserta. Saída 0 (nada, ou consertado),
/// 1 (falta consertar ou deu erro).
pub fn run(args: &[String], cwd: &Path) -> i32 {
    let apply_too = args.iter().any(|a| a == "--apply");
    if let Some(other) = args.iter().find(|a| *a != "--apply") {
        eprintln!("{}", fl!("ui-fix-usage", arg = other.clone()));
        return 2;
    }
    let problems = find(cwd, SystemTime::now());
    if problems.is_empty() {
        println!("{}", fl!("ui-fix-nothing"));
        return 0;
    }
    for p in &problems {
        let mark = if p.fixable() { "-" } else { "!" };
        println!("{mark} {}", p.describe());
    }
    if !apply_too {
        if problems.iter().any(Problem::fixable) {
            println!("{}", fl!("ui-fix-hint"));
        }
        return 1;
    }
    match apply(cwd, &problems) {
        Ok(done) => {
            println!("{}", fl!("ui-fix-done", count = done.len()));
            i32::from(problems.iter().any(|p| !p.fixable()))
        }
        Err(e) => {
            eprintln!("{}", fl!("ui-fix-failed", error = e.to_string()));
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"{"format": "idetea-ui", "version": 1, "components": []}"#;

    fn project(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("idetea-fix-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(store::ui_dir(&root)).unwrap();
        root
    }

    fn touch(path: &Path, ago: Duration) {
        let t = SystemTime::now() - ago;
        std::fs::File::options().write(true).open(path).unwrap().set_modified(t).unwrap();
    }

    #[test]
    fn finds_and_fixes_what_was_left_behind() {
        let root = project("tudo");
        let ui = store::ui_dir(&root);
        let write = |f: &str, text: &str, ago: u64| {
            std::fs::write(ui.join(f), text).unwrap();
            touch(&ui.join(f), Duration::from_secs(ago));
        };
        write("esperando.json", DOC, 3600);
        write("esperando.wait", "", 1); // o agente ainda espera
        write("parou.json", DOC, 3600);
        write("parou.wait", "", 20 * 60); // parou há 20 min
        write("velha.json", DOC, 2 * 24 * 3600); // sem .wait, dois dias parada
        write("nova.json", DOC, 60);
        write("respondida.json", DOC, 2 * 24 * 3600);
        std::fs::create_dir_all(ui.join("responses/respondida")).unwrap();
        std::fs::write(ui.join("responses/respondida/000001-e1.json"), "{}").unwrap();
        std::fs::create_dir_all(ui.join("responses/sumiu")).unwrap();
        std::fs::write(ui.join("responses/sumiu/000001-e1.json"), "{}").unwrap();
        write("sobrou.wait", "", 3600);
        write(".meio.json.tmp", "{", 3600);
        write(".agora.json.tmp", "{", 5);
        write("check", "#!/bin/sh\nexec '/nao/existe/idetea' ui check \"$@\"\n", 0);
        std::fs::write(root.join("AGENTS.md"), format!("{}\nantigo\n{}\n", teach::BEGIN, teach::END)).unwrap();

        let found = find(&root, SystemTime::now());
        let has = |p: &Problem| found.contains(p);
        assert!(has(&Problem::Abandoned { name: "parou".into(), minutes: 20 }), "{found:?}");
        assert!(has(&Problem::Abandoned { name: "velha".into(), minutes: 2 * 24 * 60 }), "{found:?}");
        assert!(has(&Problem::Unread { name: "respondida".into(), count: 1 }), "a resposta não lida protege o documento");
        assert!(has(&Problem::Unread { name: "sumiu".into(), count: 1 }), "resposta de interface apagada");
        assert!(has(&Problem::Leftover { path: ".idetea/ui/sobrou.wait".into() }));
        assert!(has(&Problem::Leftover { path: ".idetea/ui/.meio.json.tmp".into() }));
        assert!(has(&Problem::BrokenCheck { target: "/nao/existe/idetea".into() }));
        assert!(has(&Problem::Instructions { file: "AGENTS.md".into(), state: teach::Stale::Unversioned }));
        assert_eq!(found.len(), 8, "esperando, nova e o temporário recente ficam: {found:?}");

        let done = apply(&root, &found).unwrap();
        assert_eq!(done.len(), 5, "o check não (binário de teste) e as respostas não lidas não: {done:?}");
        for gone in ["parou.json", "parou.wait", "velha.json", "sobrou.wait", ".meio.json.tmp"] {
            assert!(!ui.join(gone).exists(), "{gone}");
        }
        for kept in ["esperando.json", "nova.json", "respondida.json", ".agora.json.tmp", "responses/sumiu/000001-e1.json"] {
            assert!(ui.join(kept).exists(), "{kept}");
        }
        let agents = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert_eq!(teach::block_version(&agents), Some(teach::version(teach::INSTRUCTIONS).as_str()));
        let again = find(&root, SystemTime::now());
        assert!(again.iter().all(|p| !p.fixable()), "só sobra o que não se conserta daqui: {again:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn newer_instructions_are_not_overwritten() {
        let root = project("nova");
        let ours = teach::version(teach::INSTRUCTIONS);
        let newer = teach::merge("", teach::INSTRUCTIONS).replace(&format!("version: `{ours}`"), "version: `ffff0000`");
        std::fs::write(root.join("AGENTS.md"), &newer).unwrap();
        let found = find(&root, SystemTime::now());
        assert_eq!(found, vec![Problem::Instructions { file: "AGENTS.md".into(), state: teach::Stale::Newer("ffff0000".into()) }]);
        assert!(!found[0].fixable());
        apply(&root, &found).unwrap();
        assert_eq!(std::fs::read_to_string(root.join("AGENTS.md")).unwrap(), newer);
        let _ = std::fs::remove_dir_all(&root);
    }
}
