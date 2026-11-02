//! `idetea ui check <arquivo>…`: confere um documento pela linha de comando, com o mesmo leitor do painel.
//!
//! O agente valida antes de publicar sem precisar de ferramenta de fora (um validador de JSON Schema pode não
//! estar instalado nem ser permitido). Erros fazem o painel recusar o documento; avisos (campo desconhecido,
//! imagem que falta) aparecem no painel, que mostra o resto.

use std::path::{Path, PathBuf};

use tea_declarative_ui::{Limits, Media, MediaLimits};

use crate::store;

/// O resultado de um arquivo: linhas para mostrar e se o painel aceitaria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub ok: bool,
    pub lines: Vec<String>,
}

/// A raiz do projeto de um documento: acima de `.idetea/ui/` (as imagens e sons são relativos a ela), senão a
/// pasta atual.
fn project_root(file: &Path, cwd: &Path) -> PathBuf {
    let dir = file.parent().unwrap_or(Path::new(""));
    let ui = dir.file_name().is_some_and(|n| n == "ui") && dir.parent().and_then(Path::file_name).is_some_and(|n| n == ".idetea");
    match dir.parent().and_then(Path::parent) {
        Some(root) if ui => root.to_path_buf(),
        _ => cwd.to_path_buf(),
    }
}

pub fn check(file: &Path, cwd: &Path) -> Report {
    let shown = file.display().to_string();
    let full = if file.is_absolute() { file.to_path_buf() } else { cwd.join(file) };
    // O temporário que o agente grava antes do `mv` (`.<nome>.json.tmp`) vale pelo nome final.
    let file_name = full.file_name().and_then(|s| s.to_str()).unwrap_or_default();
    let base = file_name.strip_prefix('.').and_then(|f| f.strip_suffix(".tmp")).unwrap_or(file_name);
    let name = base.strip_suffix(".json").unwrap_or(base).to_string();
    let mut lines = Vec::new();
    if !base.ends_with(".json") || !store::valid_name(&name) {
        lines.push(fl!("ui-check-name", name = name.clone()));
    }
    let (doc, _) = store::read_document(&full, &name, &Limits::default());
    let doc = match doc {
        Ok(doc) => doc,
        Err(e) => {
            lines.insert(0, fl!("ui-check-error", file = shown, error = e));
            return Report { ok: false, lines };
        }
    };
    let ok = lines.is_empty();
    let header = if ok {
        fl!("ui-check-ok", file = shown, count = doc.components.len())
    } else {
        fl!("ui-check-error", file = shown, error = lines.remove(0))
    };
    lines.insert(0, header);
    for w in &doc.warnings {
        lines.push(fl!("ui-check-warning", warning = w.clone()));
    }
    let root = project_root(&full, cwd);
    for (path, problem) in Media::load(&root, &doc, &MediaLimits::default()).problems() {
        lines.push(fl!("ui-check-media", path = path, problem = problem));
    }
    Report { ok, lines }
}

/// Confere cada arquivo e mostra o resultado. Código de saída: 0 se todos passam, 1 se algum tem erro, 2 sem
/// arquivos.
pub fn run(files: &[PathBuf], cwd: &Path) -> i32 {
    if files.is_empty() {
        eprintln!("{}", fl!("ui-check-usage"));
        return 2;
    }
    let mut code = 0;
    for f in files {
        let report = check(f, cwd);
        for (i, line) in report.lines.iter().enumerate() {
            println!("{}{line}", if i == 0 { "" } else { "  " });
        }
        if !report.ok {
            code = 1;
        }
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("idetea-check-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(store::ui_dir(&root)).unwrap();
        root
    }

    #[test]
    fn checks_like_the_panel() {
        let root = project("doc");
        let ui = store::ui_dir(&root);
        std::fs::write(root.join("a.png"), include_bytes!("../testdata/folha-6x2.png")).unwrap();
        std::fs::write(
            ui.join("boa.json"),
            r#"{"format": "idetea-ui", "version": 1, "components": [
                {"id": "i", "component": "image", "path": "a.png", "cor": "azul"},
                {"id": "s", "component": "sprite", "path": "a.png", "frames": 2, "frame_width": 9, "frame_height": 9},
                {"id": "x", "component": "audio", "path": "falta.wav"}]}"#,
        )
        .unwrap();
        let r = check(&ui.join("boa.json"), &root);
        assert!(r.ok, "{:?}", r.lines);
        assert_eq!(r.lines[0], fl!("ui-check-ok", file = ui.join("boa.json").display().to_string(), count = 3));
        assert_eq!(r.lines.len(), 4, "um aviso de campo e dois de mídia: {:?}", r.lines);
        assert!(r.lines.iter().any(|l| l.contains("cor")));
        assert!(r.lines.iter().any(|l| l.contains("falta.wav")));
        assert!(r.lines.iter().any(|l| l.contains("#s")), "o sprite maior que a folha");

        std::fs::write(ui.join("ruim.json"), r#"{"format": "idetea-ui", "version": 1}"#).unwrap();
        std::fs::write(ui.join("Nome.json"), r#"{"format": "idetea-ui", "version": 1, "components": []}"#).unwrap();
        std::fs::write(ui.join("outra.json"), r#"{"format": "idetea-ui", "version": 1, "surface": "x", "components": []}"#).unwrap();
        for bad in ["ruim.json", "Nome.json", "outra.json"] {
            let r = check(&ui.join(bad), &root);
            assert!(!r.ok, "{bad}: {:?}", r.lines);
        }
        assert!(!check(Path::new("nao-existe.json"), &root).ok);
        std::fs::copy(ui.join("boa.json"), ui.join(".nova.json.tmp")).unwrap();
        assert!(check(&ui.join(".nova.json.tmp"), &root).ok, "o temporário antes do mv vale pelo nome final");
        assert_eq!(run(&[ui.join("boa.json")], &root), 0);
        assert_eq!(run(&[ui.join("boa.json"), ui.join("ruim.json")], &root), 1);
        assert_eq!(run(&[], &root), 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn root_is_above_the_ui_folder() {
        let cwd = Path::new("/outro");
        assert_eq!(project_root(Path::new("/p/.idetea/ui/a.json"), cwd), PathBuf::from("/p"));
        assert_eq!(project_root(Path::new("/p/rascunho/a.json"), cwd), PathBuf::from("/outro"));
    }
}
