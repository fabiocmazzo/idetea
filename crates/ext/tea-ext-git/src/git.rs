//! Chamadas ao binário `git` e leitura da saída. Sem UI.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Onde fica o repositório.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Repo {
    /// Pasta de trabalho (raiz do repositório).
    pub root: PathBuf,
    /// Pasta `.git` (absoluta; pode ficar fora da raiz em worktrees).
    pub git_dir: PathBuf,
}

/// Roda `git` na pasta e devolve a saída. Códigos de saída em `ok_codes` contam como sucesso.
pub fn run(dir: &Path, args: &[&str], ok_codes: &[i32]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "color.ui=false", "-c", "core.quotepath=false"])
        .args(args)
        // Nunca pedir senha ou abrir editor: não há terminal para responder.
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("não foi possível rodar git: {e}"))?;
    let code = out.status.code().unwrap_or(-1);
    if code == 0 || ok_codes.contains(&code) {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let msg = err.lines().rfind(|l| !l.trim().is_empty()).unwrap_or("erro desconhecido");
        Err(msg.trim().to_string())
    }
}

/// Repositório que contém `dir`, se houver (e se o `git` existir).
pub fn discover(dir: &Path) -> Option<Repo> {
    let out = run(dir, &["rev-parse", "--show-toplevel", "--absolute-git-dir"], &[]).ok()?;
    let mut lines = out.lines();
    let root = PathBuf::from(lines.next()?);
    let git_dir = PathBuf::from(lines.next()?);
    Some(Repo { root, git_dir })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    /// `None` = HEAD solto (detached); aí `oid` diz onde está.
    pub branch: Option<String>,
    pub oid: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Relativo à raiz do repositório.
    pub path: String,
    /// Nome anterior, em renomeações.
    pub orig: Option<String>,
    /// Letra do lado do stage (`M`, `A`, `D`, `R`...), `None` se nada no stage.
    pub staged: Option<char>,
    /// Letra do lado da pasta de trabalho, `None` se nada fora do stage.
    pub unstaged: Option<char>,
    pub untracked: bool,
    pub conflict: bool,
}

impl Status {
    pub fn is_staged_empty(&self) -> bool {
        !self.entries.iter().any(|e| e.staged.is_some())
    }

    /// Nome curto para a barra: branch ou começo do commit.
    pub fn head_label(&self) -> String {
        match (&self.branch, &self.oid) {
            (Some(b), _) => b.clone(),
            (None, Some(oid)) => oid.chars().take(8).collect(),
            (None, None) => "?".into(),
        }
    }
}

pub fn status(repo: &Repo) -> Result<Status, String> {
    // `--no-optional-locks`: não disputar o index com o git que o usuário roda no terminal.
    let out = run(&repo.root, &["--no-optional-locks", "status", "--porcelain=v2", "--branch", "-z"], &[])?;
    Ok(parse_status(&out))
}

fn side(c: char) -> Option<char> {
    (c != '.').then_some(c)
}

/// Lê `git status --porcelain=v2 --branch -z`.
pub fn parse_status(out: &str) -> Status {
    let mut st = Status::default();
    let mut records = out.split('\0');
    while let Some(rec) = records.next() {
        if let Some(header) = rec.strip_prefix("# ") {
            let (key, value) = header.split_once(' ').unwrap_or((header, ""));
            match key {
                "branch.oid" if value != "(initial)" => st.oid = Some(value.to_string()),
                "branch.head" if value != "(detached)" => st.branch = Some(value.to_string()),
                "branch.upstream" => st.upstream = Some(value.to_string()),
                "branch.ab" => {
                    for part in value.split(' ') {
                        if let Some(n) = part.strip_prefix('+') {
                            st.ahead = n.parse().unwrap_or(0);
                        } else if let Some(n) = part.strip_prefix('-') {
                            st.behind = n.parse().unwrap_or(0);
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        let kind = rec.chars().next();
        match kind {
            // 1 XY sub mH mI mW hH hI path
            Some('1') => {
                let f: Vec<&str> = rec.splitn(9, ' ').collect();
                if let [_, xy, .., path] = f.as_slice()
                    && f.len() == 9
                {
                    st.entries.push(changed(xy, path, None, false));
                }
            }
            // 2 XY sub mH mI mW hH hI Xscore path \0 origPath
            Some('2') => {
                let f: Vec<&str> = rec.splitn(10, ' ').collect();
                let orig = records.next().map(str::to_string);
                if let [_, xy, .., path] = f.as_slice()
                    && f.len() == 10
                {
                    st.entries.push(changed(xy, path, orig, false));
                }
            }
            // u XY sub m1 m2 m3 mW h1 h2 h3 path
            Some('u') => {
                let f: Vec<&str> = rec.splitn(11, ' ').collect();
                if let [_, xy, .., path] = f.as_slice()
                    && f.len() == 11
                {
                    st.entries.push(changed(xy, path, None, true));
                }
            }
            Some('?') => st.entries.push(Entry {
                path: rec[2..].to_string(),
                orig: None,
                staged: None,
                unstaged: Some('?'),
                untracked: true,
                conflict: false,
            }),
            _ => {}
        }
    }
    st.entries.sort_by(|a, b| a.path.cmp(&b.path));
    st
}

fn changed(xy: &str, path: &str, orig: Option<String>, conflict: bool) -> Entry {
    let mut c = xy.chars();
    let (x, y) = (c.next().unwrap_or('.'), c.next().unwrap_or('.'));
    if conflict {
        return Entry { path: path.into(), orig, staged: None, unstaged: Some('U'), untracked: false, conflict };
    }
    Entry { path: path.into(), orig, staged: side(x), unstaged: side(y), untracked: false, conflict }
}

// ---- Diff ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Header,
    Hunk,
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub text: String,
    pub kind: LineKind,
    /// Linha no arquivo novo (começa em 1), para "clique abre aqui".
    pub new_line: Option<usize>,
}

/// Diff de um arquivo: do stage (`staged`) ou da pasta de trabalho. Arquivo novo aparece inteiro como adicionado.
pub fn diff(repo: &Repo, entry: &Entry, staged: bool) -> Result<String, String> {
    let path = entry.path.as_str();
    if entry.untracked {
        // `--no-index` sai com 1 quando há diferença.
        return run(&repo.root, &["diff", "--no-index", "--", "/dev/null", path], &[1]);
    }
    if staged {
        run(&repo.root, &["diff", "--cached", "--", path], &[])
    } else {
        run(&repo.root, &["diff", "--", path], &[])
    }
}

pub fn parse_diff(out: &str, max_lines: usize) -> (Vec<DiffLine>, bool) {
    let mut lines = Vec::new();
    let mut next_new: Option<usize> = None;
    for (i, raw) in out.lines().enumerate() {
        if i >= max_lines {
            return (lines, true);
        }
        let (kind, new_line) = if raw.starts_with("@@") {
            next_new = hunk_new_start(raw);
            (LineKind::Hunk, next_new)
        } else if next_new.is_none() {
            (LineKind::Header, None)
        } else if raw.starts_with('+') {
            let n = next_new;
            next_new = n.map(|n| n + 1);
            (LineKind::Added, n)
        } else if raw.starts_with('-') {
            (LineKind::Removed, next_new)
        } else if raw.starts_with(' ') || raw.is_empty() {
            let n = next_new;
            next_new = n.map(|n| n + 1);
            (LineKind::Context, n)
        } else if raw.starts_with("diff ") {
            // Próximo arquivo (não acontece com um caminho só, mas por segurança).
            next_new = None;
            (LineKind::Header, None)
        } else {
            // "\ No newline at end of file"
            (LineKind::Header, None)
        };
        lines.push(DiffLine { text: raw.to_string(), kind, new_line });
    }
    (lines, false)
}

/// `@@ -10,7 +12,8 @@ fn x` → 12.
fn hunk_new_start(header: &str) -> Option<usize> {
    let plus = header.split(' ').find(|p| p.starts_with('+'))?;
    let n = plus[1..].split(',').next()?;
    n.parse().ok().map(|n: usize| n.max(1))
}

// ---- Ações -------------------------------------------------------------------

pub fn stage(repo: &Repo, paths: &[String]) -> Result<String, String> {
    let mut args = vec!["add", "-A", "--"];
    args.extend(paths.iter().map(String::as_str));
    run(&repo.root, &args, &[])
}

pub fn stage_all(repo: &Repo) -> Result<String, String> {
    run(&repo.root, &["add", "-A"], &[])
}

pub fn unstage(repo: &Repo, paths: &[String]) -> Result<String, String> {
    let mut args = vec!["restore", "--staged", "--"];
    args.extend(paths.iter().map(String::as_str));
    match run(&repo.root, &args, &[]) {
        Ok(o) => Ok(o),
        // Repositório sem nenhum commit ainda: não há HEAD para "restaurar".
        Err(_) => {
            let mut args = vec!["rm", "--cached", "-r", "-q", "--"];
            args.extend(paths.iter().map(String::as_str));
            run(&repo.root, &args, &[])
        }
    }
}

pub fn unstage_all(repo: &Repo) -> Result<String, String> {
    match run(&repo.root, &["reset", "-q"], &[]) {
        Ok(o) => Ok(o),
        Err(_) => run(&repo.root, &["rm", "--cached", "-r", "-q", "."], &[]),
    }
}

/// Descarta as mudanças fora do stage: arquivo rastreado volta ao que está no stage;
/// arquivo novo (não rastreado) vai para a lixeira.
pub fn discard(repo: &Repo, entry: &Entry) -> Result<String, String> {
    if entry.untracked {
        trash::delete(repo.root.join(&entry.path)).map_err(|e| format!("não foi possível mover para a lixeira: {e}"))?;
        return Ok(String::new());
    }
    run(&repo.root, &["restore", "--", &entry.path], &[])
}

pub fn commit(repo: &Repo, message: &str) -> Result<String, String> {
    let out = run(&repo.root, &["commit", "-q", "-m", message], &[])?;
    // Primeira linha do log: "abc1234 mensagem".
    let head = run(&repo.root, &["log", "-1", "--format=%h %s"], &[]).unwrap_or_default();
    Ok(if head.trim().is_empty() { out } else { head.trim().to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_and_entries() {
        let out = "# branch.oid 1234567890abcdef\0# branch.head main\0# branch.upstream origin/main\0# branch.ab +2 -1\0\
1 .M N... 100644 100644 100644 aaa bbb src/main.rs\0\
1 A. N... 000000 100644 100644 000 ccc nova pasta/novo.rs\0\
2 R. N... 100644 100644 100644 ddd eee R100 depois.rs\0antes.rs\0\
u UU N... 100644 100644 100644 100644 a b c conflito.rs\0\
? lixo.txt\0";
        let st = parse_status(out);
        assert_eq!(st.branch.as_deref(), Some("main"));
        assert_eq!(st.upstream.as_deref(), Some("origin/main"));
        assert_eq!((st.ahead, st.behind), (2, 1));
        let by = |p: &str| st.entries.iter().find(|e| e.path == p).unwrap().clone();
        assert_eq!(by("src/main.rs").staged, None);
        assert_eq!(by("src/main.rs").unstaged, Some('M'));
        assert_eq!(by("nova pasta/novo.rs").staged, Some('A'), "caminho com espaço");
        assert_eq!(by("depois.rs").orig.as_deref(), Some("antes.rs"));
        assert!(by("conflito.rs").conflict);
        assert!(by("lixo.txt").untracked);
        assert_eq!(st.entries.len(), 5, "o nome antigo do rename não vira entrada");
    }

    #[test]
    fn detached_and_initial() {
        let st = parse_status("# branch.oid abcdef1234\0# branch.head (detached)\0");
        assert_eq!(st.branch, None);
        assert_eq!(st.head_label(), "abcdef12");
        let st = parse_status("# branch.oid (initial)\0# branch.head main\0");
        assert_eq!(st.oid, None);
        assert_eq!(st.head_label(), "main");
    }

    #[test]
    fn diff_line_numbers() {
        let out = "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1,3 +1,4 @@\n a\n-b\n+B\n+C\n c\n";
        let (lines, truncated) = parse_diff(out, 100);
        assert!(!truncated);
        let nums: Vec<_> = lines.iter().map(|l| (l.kind, l.new_line)).collect();
        assert_eq!(nums[0], (LineKind::Header, None));
        assert_eq!(nums[2], (LineKind::Header, None), "+++ antes do hunk é cabeçalho");
        assert_eq!(nums[3], (LineKind::Hunk, Some(1)));
        assert_eq!(nums[4], (LineKind::Context, Some(1)));
        assert_eq!(nums[5], (LineKind::Removed, Some(2)));
        assert_eq!(nums[6], (LineKind::Added, Some(2)));
        assert_eq!(nums[7], (LineKind::Added, Some(3)));
        assert_eq!(nums[8], (LineKind::Context, Some(4)));
        assert!(parse_diff(out, 3).1, "corta e avisa");
    }

    /// Fluxo real com o git instalado: init, status, stage, commit, diff.
    #[test]
    fn real_repo_flow() {
        let dir = std::env::temp_dir().join(format!("idetea-git-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        if run(&dir, &["init", "-q", "-b", "main"], &[]).is_err() {
            return; // sem git na máquina
        }
        run(&dir, &["config", "user.email", "t@t"], &[]).unwrap();
        run(&dir, &["config", "user.name", "t"], &[]).unwrap();
        run(&dir, &["config", "commit.gpgsign", "false"], &[]).unwrap();
        let repo = discover(&dir).expect("repositório");
        std::fs::write(dir.join("a.txt"), "um\n").unwrap();

        let st = status(&repo).unwrap();
        assert!(st.entries[0].untracked);
        assert!(diff(&repo, &st.entries[0], false).unwrap().contains("+um"));

        stage(&repo, &["a.txt".into()]).unwrap();
        unstage(&repo, &["a.txt".into()]).unwrap();
        assert!(status(&repo).unwrap().is_staged_empty(), "unstage sem commit nenhum");
        stage_all(&repo).unwrap();
        assert_eq!(status(&repo).unwrap().entries[0].staged, Some('A'));
        assert!(commit(&repo, "primeiro").unwrap().ends_with("primeiro"));
        assert!(status(&repo).unwrap().entries.is_empty());

        std::fs::write(dir.join("a.txt"), "um\ndois\n").unwrap();
        let st = status(&repo).unwrap();
        assert_eq!(st.entries[0].unstaged, Some('M'));
        assert!(diff(&repo, &st.entries[0], false).unwrap().contains("+dois"));

        discard(&repo, &st.entries[0]).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "um\n", "volta ao commit");
        assert!(status(&repo).unwrap().entries.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
