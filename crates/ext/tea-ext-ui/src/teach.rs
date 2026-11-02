//! "Ensinar os agentes": grava no `AGENTS.md` (e no `CLAUDE.md`, se houver) o texto que explica a um agente de
//! código como perguntar pelo painel. O bloco fica entre marcadores; rodar de novo só troca o que está entre eles.

use std::path::{Path, PathBuf};

pub const BEGIN: &str = "<!-- idetea-ui:begin (IDE TEA: \"Interfaces\" command; edit outside these markers) -->";
pub const END: &str = "<!-- idetea-ui:end -->";
const BEGIN_PREFIX: &str = "<!-- idetea-ui:begin";
/// O `CLAUDE.md` criado junto com um `AGENTS.md` novo: só importa o outro (o texto fica num lugar só).
const CLAUDE_IMPORT: &str = "@AGENTS.md\n";

/// O texto para os agentes. Sempre em inglês (é para agentes lerem); ele pede para conversar no idioma do
/// usuário, então não depende do idioma da IDE.
pub const INSTRUCTIONS: &str = include_str!("../agents/instructions.md");

/// O arquivo com o bloco novo: troca o bloco que já existir ou acrescenta no fim.
pub fn merge(existing: &str, text: &str) -> String {
    let block = format!("{BEGIN}\n{}\n{END}", text.trim_end());
    if let Some(b) = existing.find(BEGIN_PREFIX)
        && let Some(e) = existing[b..].find(END).map(|e| b + e)
    {
        return format!("{}{block}{}", &existing[..b], &existing[e + END.len()..]);
    }
    let head = existing.trim_end();
    if head.is_empty() { format!("{block}\n") } else { format!("{head}\n\n{block}\n") }
}

/// Um arquivo a gravar.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// Relativo à raiz, para mostrar.
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    /// O arquivo ainda não existe.
    pub new: bool,
}

/// O que gravar no projeto: os `AGENTS.md`/`CLAUDE.md` que existirem (um symlink para o outro conta uma vez; um
/// `CLAUDE.md` que já importa o `AGENTS.md` com `@AGENTS.md` fica de fora). Sem nenhum dos dois: um `AGENTS.md`
/// novo com o texto e um `CLAUDE.md` novo que só importa o `AGENTS.md` (o Claude Code lê o `CLAUDE.md`; os outros
/// agentes, o `AGENTS.md`). Arquivos que já estão com o texto atual não entram.
pub fn plan(root: &Path, text: &str) -> std::io::Result<Vec<Change>> {
    let real_root = root.canonicalize()?;
    let agents = root.join("AGENTS.md");
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut out = Vec::new();
    let mut any = false;
    for name in ["AGENTS.md", "CLAUDE.md"] {
        let path = root.join(name);
        if std::fs::symlink_metadata(&path).is_err() {
            continue;
        }
        let real = path.canonicalize()?;
        if !real.starts_with(&real_root) || !real.is_file() || seen.contains(&real) {
            continue; // fora do projeto, não é arquivo, ou já contado
        }
        seen.push(real.clone());
        let existing = std::fs::read_to_string(&real)?;
        if name == "CLAUDE.md" && agents.exists() && existing.contains("@AGENTS.md") {
            continue;
        }
        any = true;
        let content = merge(&existing, text);
        if content != existing {
            out.push(Change { name: name.to_string(), path: real, content, new: false });
        }
    }
    if !any {
        out.push(Change { name: "AGENTS.md".to_string(), path: agents, content: merge("", text), new: true });
        let claude = root.join("CLAUDE.md");
        out.push(Change { name: "CLAUDE.md".to_string(), path: claude, content: CLAUDE_IMPORT.to_string(), new: true });
    }
    Ok(out)
}

/// Grava. Escrita direta (não `rename`): um `AGENTS.md` que é symlink continua symlink.
pub fn apply(changes: &[Change]) -> std::io::Result<()> {
    for c in changes {
        std::fs::write(&c.path, &c.content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("idetea-teach-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn merge_appends_then_replaces() {
        let first = merge("# Projeto\n\nRegras.\n", "texto 1\n");
        assert_eq!(first, format!("# Projeto\n\nRegras.\n\n{BEGIN}\ntexto 1\n{END}\n"));
        let edited = format!("{first}\n## Depois\nmais coisas\n");
        let second = merge(&edited, "texto 2");
        assert_eq!(second, format!("# Projeto\n\nRegras.\n\n{BEGIN}\ntexto 2\n{END}\n\n## Depois\nmais coisas\n"));
        assert_eq!(merge(&second, "texto 2"), second, "rodar de novo não muda nada");
        assert_eq!(merge("", "x"), format!("{BEGIN}\nx\n{END}\n"));
    }

    #[test]
    fn plan_picks_the_right_files() {
        let root = tmp_root("plan");
        // Nada: cria o AGENTS.md com o texto e um CLAUDE.md que só importa o AGENTS.md.
        let p = plan(&root, "t").unwrap();
        let names: Vec<(&str, bool)> = p.iter().map(|c| (c.name.as_str(), c.new)).collect();
        assert_eq!(names, [("AGENTS.md", true), ("CLAUDE.md", true)]);
        apply(&p).unwrap();
        assert_eq!(std::fs::read_to_string(root.join("CLAUDE.md")).unwrap(), "@AGENTS.md\n");
        assert!(plan(&root, "t").unwrap().is_empty(), "já atualizado (o CLAUDE.md importa o AGENTS.md)");

        // CLAUDE.md que importa o AGENTS.md: fica de fora. Sem o import: entra.
        std::fs::write(root.join("CLAUDE.md"), "@AGENTS.md\n").unwrap();
        let p = plan(&root, "t2").unwrap();
        assert_eq!(p.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["AGENTS.md"]);
        std::fs::write(root.join("CLAUDE.md"), "# Claude\n").unwrap();
        let p = plan(&root, "t2").unwrap();
        assert_eq!(p.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["AGENTS.md", "CLAUDE.md"]);

        // AGENTS.md é symlink para o CLAUDE.md: conta uma vez, e continua symlink depois de gravar.
        #[cfg(unix)]
        {
            std::fs::remove_file(root.join("AGENTS.md")).unwrap();
            std::os::unix::fs::symlink("CLAUDE.md", root.join("AGENTS.md")).unwrap();
            let p = plan(&root, "t3").unwrap();
            assert_eq!(p.len(), 1);
            apply(&p).unwrap();
            assert!(std::fs::symlink_metadata(root.join("AGENTS.md")).unwrap().file_type().is_symlink());
            assert!(std::fs::read_to_string(root.join("CLAUDE.md")).unwrap().contains("t3"));
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// O texto é o mesmo que a documentação mostra.
    #[test]
    fn docs_show_the_same_text() {
        let docs = include_str!("../../../../docs/interfaces/agentes.md");
        assert!(docs.contains(INSTRUCTIONS), "atualize docs/interfaces/agentes.md junto com agents/instructions.md");
    }
}
