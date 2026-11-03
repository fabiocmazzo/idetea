//! `idetea skills`: instala skills de agentes no projeto (RFC 0005, etapa 2).
//!
//! Uma skill é uma pasta com um `SKILL.md` (cabeçalho com `name` e `description`). A fonte é um repositório git;
//! cada skill instalada fica fixada no commit de onde veio, anotado em `.idetea/skills.toml`. A cópia vai para
//! `.agents/skills/<nome>/` (Codex, OpenCode e outros) e para `.claude/skills/<nome>/` (Claude Code). A IDE só
//! copia arquivos: nada da skill é executado, symlinks e submódulos ficam de fora, e uma cópia mexida à mão não é
//! sobrescrita sem `--force`.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

pub mod repo;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use repo::{Entry, Repo, Source};

/// Onde as skills ficam no projeto (cada uma recebe uma cópia).
pub const TARGETS: [&str; 2] = [".agents/skills", ".claude/skills"];
/// O registro do que foi instalado, relativo à raiz.
pub const LOCK: &str = ".idetea/skills.toml";
/// A fonte quando o usuário não diz outra.
pub const DEFAULT_SOURCE: &str = "anthropics/skills";

/// Uma skill de uma fonte, num commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    /// A pasta da skill no repositório.
    pub path: String,
    pub files: usize,
    pub bytes: u64,
    /// Tem arquivos que o agente pode rodar (executáveis ou scripts).
    pub scripts: bool,
    /// Symlinks e submódulos: não são copiados.
    pub skipped: usize,
}

/// Uma skill instalada no projeto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installed {
    pub name: String,
    pub source: String,
    pub commit: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    #[serde(default, rename = "skill")]
    pub skills: Vec<Installed>,
}

const LOCK_HEADER: &str = "# Escrito pela IDE TEA (idetea skills): de onde veio cada skill instalada em .agents/skills/ e\n# .claude/skills/, e em que commit. Vai para o git junto com as skills.\n\n";

impl Lock {
    pub fn load(root: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(root.join(LOCK)) {
            Ok(text) => toml::from_str(&text).map_err(|e| format!("{LOCK}: {e}")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("{LOCK}: {e}")),
        }
    }

    pub fn save(&self, root: &Path) -> Result<(), String> {
        let path = root.join(LOCK);
        if self.skills.is_empty() {
            return match std::fs::remove_file(&path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!("{LOCK}: {e}")),
                _ => Ok(()),
            };
        }
        let text = toml::to_string(self).map_err(|e| format!("{LOCK}: {e}"))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&path, format!("{LOCK_HEADER}{text}")).map_err(|e| format!("{LOCK}: {e}"))
    }

    pub fn get(&self, name: &str) -> Option<&Installed> {
        self.skills.iter().find(|s| s.name == name)
    }

    fn put(&mut self, skill: Installed) {
        self.skills.retain(|s| s.name != skill.name);
        self.skills.push(skill);
        self.skills.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Nome que vira pasta: minúsculas, dígitos, `-` e `_`, até 64 (nada de `..` nem `/`).
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        && name.starts_with(|c: char| c.is_ascii_alphanumeric())
}

/// `name` e `description` do cabeçalho (`---` … `---`) de um `SKILL.md`. Aceita aspas e os blocos `|` e `>`.
pub fn front_matter(text: &str) -> (Option<String>, Option<String>) {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (None, None);
    }
    let head: Vec<&str> = lines.take_while(|l| l.trim() != "---").collect();
    let field = |key: &str| -> Option<String> {
        let i = head.iter().position(|l| l.strip_prefix(key).is_some_and(|r| r.starts_with(':')))?;
        let value = head[i][key.len() + 1..].trim();
        if value == "|" || value == ">" || value == "|-" || value == ">-" {
            let block: Vec<&str> = head[i + 1..].iter().take_while(|l| l.starts_with([' ', '\t'])).map(|l| l.trim()).collect();
            return Some(block.join(" ")).filter(|s| !s.is_empty());
        }
        let unquoted = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')).or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')));
        Some(unquoted.unwrap_or(value).to_string()).filter(|s| !s.is_empty())
    };
    (field("name"), field("description"))
}

const SCRIPT_EXTENSIONS: [&str; 10] = ["sh", "bash", "py", "js", "mjs", "ts", "rb", "pl", "ps1", "bat"];

fn is_script(e: &Entry) -> bool {
    e.mode == "100755" || Path::new(&e.path).extension().and_then(|x| x.to_str()).is_some_and(|x| SCRIPT_EXTENSIONS.contains(&x))
}

/// As skills de `commit`, por nome. Pastas sem nome válido ficam de fora (`warnings`).
pub fn skills_at(repo: &Repo, commit: &str) -> Result<(Vec<Skill>, Vec<String>), String> {
    let entries = repo.entries(commit, None)?;
    let mut out: Vec<Skill> = Vec::new();
    let mut warnings = Vec::new();
    for e in entries.iter().filter(|e| e.is_file() && (e.path == "SKILL.md" || e.path.ends_with("/SKILL.md"))) {
        let dir = e.path.strip_suffix("SKILL.md").unwrap_or("").trim_end_matches('/').to_string();
        let text = String::from_utf8_lossy(&repo.read(&e.object)?).into_owned();
        let (name, description) = front_matter(&text);
        let fallback = dir.rsplit('/').next().unwrap_or("").to_string();
        let name = name.unwrap_or(fallback);
        if !valid_name(&name) {
            warnings.push(fl!("skills-bad-name", path = e.path.clone(), name = name));
            continue;
        }
        if out.iter().any(|s| s.name == name) {
            warnings.push(fl!("skills-duplicate", path = e.path.clone(), name = name));
            continue;
        }
        let inside: Vec<&Entry> = entries.iter().filter(|f| dir.is_empty() || f.path.starts_with(&format!("{dir}/"))).collect();
        let files: Vec<&&Entry> = inside.iter().filter(|f| f.is_file()).collect();
        out.push(Skill {
            name,
            description: description.unwrap_or_default(),
            files: files.len(),
            bytes: files.iter().map(|f| f.size).sum(),
            scripts: files.iter().any(|f| is_script(f)),
            skipped: inside.len() - files.len(),
            path: dir,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok((out, warnings))
}

/// Os arquivos de uma skill num commit: caminho dentro da skill → (conteúdo, executável).
fn skill_files(repo: &Repo, commit: &str, path: &str) -> Result<Vec<(String, Vec<u8>, bool)>, String> {
    let prefix = if path.is_empty() { None } else { Some(path) };
    let mut out = Vec::new();
    for e in repo.entries(commit, prefix)?.into_iter().filter(Entry::is_file) {
        let rel = if path.is_empty() { e.path.clone() } else { e.path[path.len() + 1..].to_string() };
        // Nada que saia da pasta da skill.
        if rel.split('/').any(|c| c.is_empty() || c == "." || c == "..") {
            continue;
        }
        out.push((rel, repo.read(&e.object)?, e.mode == "100755"));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Os arquivos de uma cópia instalada (caminho relativo → conteúdo). Symlink no caminho: erro.
fn read_tree(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> std::io::Result<()> {
        for e in std::fs::read_dir(dir)? {
            let e = e?;
            let t = e.file_type()?;
            if t.is_symlink() {
                return Err(std::io::Error::other(format!("{}: symlink", e.path().display())));
            } else if t.is_dir() {
                walk(base, &e.path(), out)?;
            } else {
                let rel = e.path().strip_prefix(base).unwrap_or(&e.path()).to_string_lossy().replace('\\', "/");
                out.push((rel, std::fs::read(e.path())?));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out).map_err(|e| e.to_string())?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Alguma cópia da skill instalada foi mexida desde que a IDE copiou (`None`: igual nas duas). Uma cópia que
/// sumiu não conta como mexida: instalar de novo a recria.
fn modified(root: &Path, repo: &Repo, inst: &Installed) -> Result<Option<String>, String> {
    let want: Vec<(String, Vec<u8>)> = skill_files(repo, &inst.commit, &inst.path)?.into_iter().map(|(p, c, _)| (p, c)).collect();
    for target in TARGETS {
        let dir = root.join(target).join(&inst.name);
        if std::fs::symlink_metadata(&dir).is_err() {
            continue;
        }
        if read_tree(&dir).ok().as_ref() != Some(&want) {
            return Ok(Some(format!("{target}/{}", inst.name)));
        }
    }
    Ok(None)
}

/// Copia a skill para cada destino: grava ao lado e troca, então uma queda no meio não deixa metade.
fn copy(root: &Path, name: &str, files: &[(String, Vec<u8>, bool)]) -> Result<(), String> {
    for target in TARGETS {
        let base = root.join(target);
        std::fs::create_dir_all(&base).map_err(|e| format!("{}: {e}", base.display()))?;
        let dest = base.join(name);
        let tmp = base.join(format!(".{name}.tmp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        for (rel, content, exec) in files {
            let path = tmp.join(rel);
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            std::fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
            #[cfg(unix)]
            if *exec {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
            }
            #[cfg(not(unix))]
            let _ = exec;
        }
        remove_copy(&dest)?;
        std::fs::rename(&tmp, &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    }
    Ok(())
}

/// Apaga uma cópia instalada. Um symlink no lugar perde só o link (nunca o que ele aponta).
fn remove_copy(dir: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(dir) {
        Err(_) => Ok(()),
        Ok(m) if m.file_type().is_symlink() || m.is_file() => std::fs::remove_file(dir).map_err(|e| format!("{}: {e}", dir.display())),
        Ok(_) => std::fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display())),
    }
}

/// Pasta do cache das fontes (`$XDG_CACHE_HOME/idetea/skills`, ou `~/.cache/idetea/skills`).
pub fn cache_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("idetea").join("skills"))
}

/// Por que um comando não terminou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// Não fiz de propósito, para não perder algo do usuário (uma cópia mexida à mão): `--force` faz.
    Refused(String),
    /// Deu errado (rede, git, disco, nome que não existe).
    Failed(String),
}

impl From<String> for Failure {
    fn from(e: String) -> Self {
        Failure::Failed(e)
    }
}

impl Failure {
    pub fn message(&self) -> &str {
        match self {
            Failure::Refused(m) | Failure::Failed(m) => m,
        }
    }
}

/// O projeto e o cache: o que os comandos usam.
pub struct Project<'a> {
    pub root: &'a Path,
    pub cache: &'a Path,
}

fn short(commit: &str) -> &str {
    &commit[..commit.len().min(8)]
}

fn size(bytes: u64) -> String {
    if bytes < 1024 { format!("{bytes} B") } else { format!("{} KB", bytes.div_ceil(1024)) }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() } else { format!("{}…", s.chars().take(max - 1).collect::<String>()) }
}

impl Project<'_> {
    fn source(&self, spec: &str) -> Result<Source, String> {
        Source::parse(spec, self.root).ok_or_else(|| fl!("skills-bad-source", source = spec.to_string()))
    }

    /// As skills instaladas.
    pub fn status(&self, out: &mut Vec<String>) -> Result<bool, Failure> {
        let lock = Lock::load(self.root)?;
        if lock.skills.is_empty() {
            out.push(fl!("skills-none-installed"));
        }
        for s in &lock.skills {
            out.push(fl!("skills-installed-line", name = s.name.clone(), source = s.source.clone(), commit = short(&s.commit).to_string()));
        }
        Ok(true)
    }

    /// As skills de uma fonte, no commit mais novo.
    pub fn list(&self, spec: &str, out: &mut Vec<String>) -> Result<bool, Failure> {
        let source = self.source(spec)?;
        let repo = Repo::fetch(&source, self.cache)?;
        let head = repo.head()?;
        let (skills, warnings) = skills_at(&repo, &head)?;
        let lock = Lock::load(self.root)?;
        out.push(fl!("skills-list-header", source = source.spec.clone(), commit = short(&head).to_string(), count = skills.len()));
        for s in &skills {
            let mut tags = vec![fl!("skills-files", count = s.files), size(s.bytes)];
            if s.scripts {
                tags.push(fl!("skills-tag-scripts"));
            }
            if s.skipped > 0 {
                tags.push(fl!("skills-tag-skipped", count = s.skipped));
            }
            match lock.get(&s.name) {
                Some(i) if i.commit == head => tags.push(fl!("skills-tag-installed")),
                Some(i) => tags.push(fl!("skills-tag-installed-at", commit = short(&i.commit).to_string())),
                None => {}
            }
            out.push(format!("  {} — {} ({})", s.name, clip(&s.description, 100), tags.join(", ")));
        }
        out.extend(warnings.into_iter().map(|w| format!("  ! {w}")));
        Ok(true)
    }

    /// Instala `names` da fonte, no commit mais novo.
    pub fn add(&self, spec: &str, names: &[String], force: bool, out: &mut Vec<String>) -> Result<bool, Failure> {
        if names.is_empty() {
            return Err(fl!("skills-add-usage").into());
        }
        let source = self.source(spec)?;
        let repo = Repo::fetch(&source, self.cache)?;
        let head = repo.head()?;
        let (skills, _) = skills_at(&repo, &head)?;
        let mut lock = Lock::load(self.root)?;
        // Confere tudo antes de copiar qualquer uma.
        let mut chosen = Vec::new();
        for name in names {
            let skill = skills.iter().find(|s| &s.name == name).ok_or_else(|| fl!("skills-not-found", name = name.clone(), source = source.spec.clone()))?;
            match lock.get(name) {
                Some(inst) if !force => {
                    if inst.source != source.spec {
                        return Err(Failure::Refused(fl!("skills-other-source", name = name.clone(), source = inst.source.clone())));
                    }
                    let inst_repo = Repo::fetch(&self.source(&inst.source)?, self.cache)?;
                    if let Some(copy) = modified(self.root, &inst_repo, inst)? {
                        return Err(Failure::Refused(fl!("skills-modified", path = copy)));
                    }
                }
                None if !force => {
                    if let Some(t) = TARGETS.iter().find(|t| std::fs::symlink_metadata(self.root.join(t).join(name)).is_ok()) {
                        return Err(Failure::Refused(fl!("skills-exists", path = format!("{t}/{name}"))));
                    }
                }
                _ => {}
            }
            chosen.push(skill.clone());
        }
        for skill in chosen {
            copy(self.root, &skill.name, &skill_files(&repo, &head, &skill.path)?)?;
            lock.put(Installed { name: skill.name.clone(), source: source.spec.clone(), commit: head.clone(), path: skill.path.clone() });
            lock.save(self.root)?;
            out.push(fl!("skills-added", name = skill.name.clone(), commit = short(&head).to_string()));
            if skill.scripts {
                out.push(format!("  ! {}", fl!("skills-has-scripts", name = skill.name)));
            }
        }
        Ok(true)
    }

    /// O que mudou nas fontes desde a instalação; com `apply`, instala o commit novo. Saída "falso" se há
    /// atualização por aplicar.
    pub fn update(&self, names: &[String], apply: bool, out: &mut Vec<String>) -> Result<bool, Failure> {
        let mut lock = Lock::load(self.root)?;
        if let Some(n) = names.iter().find(|n| lock.get(n).is_none()) {
            return Err(fl!("skills-not-installed", name = n.clone()).into());
        }
        let mut pending = false;
        let chosen: Vec<Installed> = lock.skills.iter().filter(|s| names.is_empty() || names.contains(&s.name)).cloned().collect();
        for inst in chosen {
            let repo = Repo::fetch(&self.source(&inst.source)?, self.cache)?;
            let head = repo.head()?;
            let (skills, _) = skills_at(&repo, &head)?;
            let Some(now) = skills.iter().find(|s| s.name == inst.name) else {
                out.push(fl!("skills-gone", name = inst.name.clone(), source = inst.source.clone()));
                continue;
            };
            if head == inst.commit || repo.diff(&inst.commit, &head, &inst.path)?.0.trim().is_empty() && now.path == inst.path {
                out.push(fl!("skills-up-to-date", name = inst.name.clone()));
                continue;
            }
            let (stat, full) = repo.diff(&inst.commit, &head, &inst.path)?;
            out.push(fl!("skills-update", name = inst.name.clone(), old = short(&inst.commit).to_string(), new = short(&head).to_string()));
            out.extend(stat.lines().map(|l| format!("  {l}")));
            if !apply {
                out.extend(full.lines().map(|l| format!("    {l}")));
                pending = true;
                continue;
            }
            if let Some(copy) = modified(self.root, &repo, &inst)? {
                return Err(Failure::Refused(fl!("skills-modified", path = copy)));
            }
            copy(self.root, &inst.name, &skill_files(&repo, &head, &now.path)?)?;
            lock.put(Installed { commit: head.clone(), path: now.path.clone(), ..inst.clone() });
            lock.save(self.root)?;
            out.push(fl!("skills-updated", name = inst.name.clone(), commit = short(&head).to_string()));
        }
        if pending {
            out.push(fl!("skills-update-hint"));
        }
        Ok(!pending)
    }

    /// Tira as skills do projeto (as duas cópias e o registro).
    pub fn remove(&self, names: &[String], force: bool, out: &mut Vec<String>) -> Result<bool, Failure> {
        if names.is_empty() {
            return Err(fl!("skills-remove-usage").into());
        }
        let mut lock = Lock::load(self.root)?;
        for name in names {
            let inst = lock.get(name).cloned().ok_or_else(|| fl!("skills-not-installed", name = name.clone()))?;
            if !force {
                let repo = Repo::fetch(&self.source(&inst.source)?, self.cache)?;
                if let Some(copy) = modified(self.root, &repo, &inst)? {
                    return Err(Failure::Refused(fl!("skills-modified", path = copy)));
                }
            }
            for target in TARGETS {
                remove_copy(&self.root.join(target).join(name))?;
            }
            lock.skills.retain(|s| &s.name != name);
            lock.save(self.root)?;
            out.push(fl!("skills-removed", name = name.clone()));
        }
        Ok(true)
    }
}

/// `idetea skills …`. Saída: 0 (feito), 1 (erro, ou atualização por aplicar), 2 (uso).
pub fn run(args: &[String], cwd: &Path) -> i32 {
    let force = args.iter().any(|a| a == "--force");
    let apply = args.iter().any(|a| a == "--apply");
    let rest: Vec<String> = args.iter().filter(|a| *a != "--force" && *a != "--apply").cloned().collect();
    if rest.iter().any(|a| a == "-h" || a == "--help" || a == "help") {
        println!("{}", fl!("skills-usage"));
        return 0;
    }
    if let Some(flag) = rest.iter().find(|a| a.starts_with('-')) {
        eprintln!("{}\n{}", fl!("skills-bad-arg", arg = flag.clone()), fl!("skills-usage"));
        return 2;
    }
    let Some(cache) = cache_dir() else {
        eprintln!("{}", fl!("skills-no-cache"));
        return 1;
    };
    let project = Project { root: cwd, cache: &cache };
    let mut out = Vec::new();
    let result = match rest.first().map(String::as_str) {
        None | Some("status") => project.status(&mut out),
        Some("list") => project.list(rest.get(1).map_or(DEFAULT_SOURCE, String::as_str), &mut out),
        Some("add") if rest.len() >= 2 => project.add(&rest[1], &rest[2..], force, &mut out),
        Some("update") => project.update(&rest[1..], apply, &mut out),
        Some("remove") => project.remove(&rest[1..], force, &mut out),
        Some(other) => {
            eprintln!("{}\n{}", fl!("skills-bad-arg", arg = other.to_string()), fl!("skills-usage"));
            return 2;
        }
    };
    for line in &out {
        println!("{line}");
    }
    match result {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(Failure::Refused(e)) => {
            eprintln!("{}", fl!("skills-refused", reason = e));
            1
        }
        Err(Failure::Failed(e)) => {
            eprintln!("{}", fl!("skills-error", error = e));
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("idetea-skills-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn git(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main"])
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    fn write(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// Uma fonte com três skills: uma simples, uma com script e um symlink, e uma com nome inválido.
    fn source(dir: &Path) {
        git(dir, &["init", "-q"]);
        write(dir, "skills/pdf/SKILL.md", "---\nname: pdf\ndescription: \"Lê PDFs\"\n---\n# PDF\n");
        write(dir, "skills/pdf/ref.md", "referência\n");
        write(dir, "skills/web/SKILL.md", "---\nname: web\ndescription: >\n  Monta sites\n  estáticos\n---\n");
        write(dir, "skills/web/tools/build.sh", "#!/bin/sh\necho oi\n");
        std::os::unix::fs::symlink("/etc/passwd", dir.join("skills/web/fora")).unwrap();
        write(dir, "skills/Ruim/SKILL.md", "---\nname: Ruim Nome\n---\n");
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "um"]);
    }

    #[test]
    fn reads_the_front_matter() {
        assert_eq!(front_matter("---\nname: a\ndescription: 'b c'\n---\n"), (Some("a".into()), Some("b c".into())));
        assert_eq!(front_matter("---\nname: a\ndescription: |\n  um\n  dois\nother: x\n---\n"), (Some("a".into()), Some("um dois".into())));
        assert_eq!(front_matter("# sem cabeçalho\nname: a"), (None, None));
        assert!(valid_name("skill-creator") && !valid_name("../x") && !valid_name("A") && !valid_name("-a"));
    }

    #[test]
    fn installs_updates_and_removes() {
        let base = temp("ciclo");
        let (src, root, cache) = (base.join("fonte"), base.join("projeto"), base.join("cache"));
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        source(&src);
        let p = Project { root: &root, cache: &cache };
        let spec = src.to_string_lossy().into_owned();
        let mut out = Vec::new();

        assert!(p.list(&spec, &mut out).unwrap());
        let text = out.join("\n");
        assert!(text.contains("pdf — Lê PDFs"), "{text}");
        assert!(text.contains("web — Monta sites estáticos"), "{text}");
        assert!(text.contains(&fl!("skills-tag-scripts")), "o script aparece: {text}");
        assert!(text.contains("Ruim Nome"), "nome inválido vira aviso: {text}");

        out.clear();
        p.add(&spec, &["pdf".into(), "web".into()], false, &mut out).unwrap();
        for t in TARGETS {
            assert_eq!(std::fs::read_to_string(root.join(t).join("pdf/ref.md")).unwrap(), "referência\n");
            let script = root.join(t).join("web/tools/build.sh");
            assert!(script.is_file());
            assert!(std::fs::symlink_metadata(root.join(t).join("web/fora")).is_err(), "o symlink não é copiado");
        }
        let lock = Lock::load(&root).unwrap();
        assert_eq!(lock.skills.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["pdf", "web"]);
        assert!(std::fs::read_to_string(root.join(LOCK)).unwrap().starts_with("# Escrito pela IDE TEA"));
        assert!(matches!(p.add(&spec, &["nenhuma".into()], false, &mut out), Err(Failure::Failed(_))));

        // Nada mudou na fonte: em dia.
        out.clear();
        assert!(p.update(&[], false, &mut out).unwrap());
        // Muda a pdf na fonte: o update mostra o diff e só aplica com --apply.
        write(&src, "skills/pdf/ref.md", "referência nova\n");
        git(&src, &["commit", "-qam", "dois"]);
        out.clear();
        assert!(!p.update(&[], false, &mut out).unwrap(), "há atualização por aplicar");
        let text = out.join("\n");
        assert!(text.contains("+referência nova"), "o diff aparece: {text}");
        assert!(text.contains(&fl!("skills-up-to-date", name = "web")), "{text}");
        assert_eq!(std::fs::read_to_string(root.join(".claude/skills/pdf/ref.md")).unwrap(), "referência\n", "sem --apply, nada muda");
        out.clear();
        assert!(p.update(&["pdf".into()], true, &mut out).unwrap());
        assert_eq!(std::fs::read_to_string(root.join(".agents/skills/pdf/ref.md")).unwrap(), "referência nova\n");

        // Uma cópia mexida à mão não é sobrescrita nem apagada sem --force.
        write(&root, ".claude/skills/web/notas.md", "minhas notas\n");
        assert!(matches!(p.remove(&["web".into()], false, &mut out), Err(Failure::Refused(m)) if m.contains(".claude/skills/web")));
        assert!(p.add(&spec, &["web".into()], false, &mut out).is_err());
        out.clear();
        p.remove(&["web".into(), "pdf".into()], true, &mut out).unwrap();
        for t in TARGETS {
            assert!(!root.join(t).join("web").exists() && !root.join(t).join("pdf").exists());
        }
        assert!(!root.join(LOCK).exists(), "sem skills, sem registro");

        // Uma pasta que já existia e não veio da IDE: não é trocada.
        write(&root, ".agents/skills/pdf/SKILL.md", "minha\n");
        assert!(matches!(p.add(&spec, &["pdf".into()], false, &mut out), Err(Failure::Refused(m)) if m.contains(".agents/skills/pdf")));
        let _ = std::fs::remove_dir_all(&base);
    }
}
