//! Chamadas ao binário `git` e leitura da saída. Sem UI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Push/pull/fetch que passam disso são interrompidos (rede parada, servidor que não responde).
const NET_TIMEOUT: Duration = Duration::from_secs(180);

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
        .map_err(|e| fl!("git-run-error", error = e.to_string()))?;
    let code = out.status.code().unwrap_or(-1);
    if code == 0 || ok_codes.contains(&code) {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        match err.lines().rfind(|l| !l.trim().is_empty()) {
            Some(msg) => Err(msg.trim().to_string()),
            None => Err(fl!("git-unknown-error")),
        }
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

/// O arquivo como está no último commit. `None`: fora do repositório, ainda sem commit, binário ou grande.
pub fn head_text(repo: &Repo, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(&repo.root).ok()?.to_string_lossy().replace('\\', "/");
    let text = run(&repo.root, &["show", &format!("HEAD:{rel}")], &[]).ok()?;
    let binary = text.as_bytes().iter().take(8192).any(|&b| b == 0);
    (!binary && text.len() <= 4 * 1024 * 1024).then_some(text)
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
    /// Caminhos ignorados (`.gitignore`); pastas inteiras terminam em `/`.
    pub ignored: Vec<String>,
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
    let out = run(&repo.root, &["--no-optional-locks", "status", "--porcelain=v2", "--branch", "--ignored", "-z"], &[])?;
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
            Some('!') => st.ignored.push(rec[2..].to_string()),
            _ => {}
        }
    }
    st.entries.sort_by(|a, b| a.path.cmp(&b.path));
    st
}

/// Status de cada caminho (absoluto), para quem desenha arquivos (a árvore).
pub fn file_statuses(repo: &Repo, st: &Status) -> HashMap<PathBuf, tea_core::FileStatus> {
    use tea_core::FileStatus as F;
    let abs = |p: &str| repo.root.join(p.trim_end_matches('/'));
    let mut map: HashMap<PathBuf, F> = st.ignored.iter().map(|p| (abs(p), F::Ignored)).collect();
    for e in &st.entries {
        let status = if e.conflict {
            F::Conflict
        } else if e.untracked {
            F::Untracked
        } else if e.unstaged == Some('D') || (e.staged == Some('D') && e.unstaged.is_none()) {
            F::Deleted
        } else if matches!(e.staged, Some('A' | 'R' | 'C')) {
            F::Added
        } else {
            F::Modified
        };
        map.insert(abs(&e.path), status);
    }
    map
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
        trash::delete(repo.root.join(&entry.path)).map_err(|e| fl!("git-trash-error", error = e.to_string()))?;
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

// ---- Branches ----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    /// `main`, ou `origin/feature` para uma remota.
    pub name: String,
    pub remote: bool,
    pub current: bool,
    /// Data do último commit (`2026-10-31`).
    pub date: String,
}

/// Branches locais (da mais recente para a mais antiga) e depois as remotas que ainda não têm uma local
/// com o mesmo nome (escolher uma cria a local, seguindo a remota).
pub fn branches(repo: &Repo) -> Result<Vec<Branch>, String> {
    let out = run(
        &repo.root,
        &[
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(HEAD)%00%(refname)%00%(refname:short)%00%(committerdate:short)",
            "refs/heads",
            "refs/remotes",
        ],
        &[],
    )?;
    Ok(parse_branches(&out))
}

pub fn parse_branches(out: &str) -> Vec<Branch> {
    let mut local = Vec::new();
    let mut remote = Vec::new();
    for line in out.lines() {
        let mut f = line.split('\0');
        let (Some(head), Some(full), Some(short), Some(date)) = (f.next(), f.next(), f.next(), f.next()) else {
            continue;
        };
        let b = Branch { name: short.to_string(), remote: false, current: head == "*", date: date.to_string() };
        if full.starts_with("refs/heads/") {
            local.push(b);
        } else if full.starts_with("refs/remotes/") && !full.ends_with("/HEAD") && short.contains('/') {
            remote.push(Branch { remote: true, ..b });
        }
    }
    remote.retain(|r| {
        let name = r.name.split_once('/').map_or(r.name.as_str(), |(_, n)| n);
        !local.iter().any(|l| l.name == name)
    });
    local.extend(remote);
    local
}

/// Troca de branch. `create`: cria a partir de onde está. Remota (`origin/x`): cria a local `x` seguindo ela.
/// Alterações não commitadas vão junto (se não colidirem com a outra branch; senão o git recusa).
pub fn switch(repo: &Repo, name: &str, create: bool, remote: bool) -> Result<String, String> {
    let args: Vec<&str> = match (create, remote) {
        (true, _) => vec!["switch", "-c", name],
        (false, true) => vec!["switch", "--track", name],
        (false, false) => vec!["switch", name],
    };
    run_net(&repo.root, &args, Duration::from_secs(60)).map_err(|e| explain(&e))
}

// ---- Blame -------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameCommit {
    pub hash: String,
    pub author: String,
    /// Segundos desde 1970.
    pub time: i64,
    pub summary: String,
    /// Linha ainda não commitada (o git usa o hash `0000…`).
    pub uncommitted: bool,
}

/// Quem mudou cada linha por último (do arquivo no disco).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Blame {
    pub commits: Vec<BlameCommit>,
    /// Para cada linha (começando em 0), o índice em `commits`.
    pub lines: Vec<usize>,
}

impl Blame {
    /// Linha começando em 1.
    pub fn at(&self, line: usize) -> Option<&BlameCommit> {
        self.commits.get(*self.lines.get(line.checked_sub(1)?)?)
    }
}

pub fn blame(repo: &Repo, path: &Path) -> Result<Blame, String> {
    let rel = path.strip_prefix(&repo.root).map_err(|_| fl!("git-unknown-error"))?.to_string_lossy().into_owned();
    let out = run(&repo.root, &["blame", "--porcelain", "--", &rel], &[])?;
    Ok(parse_blame(&out))
}

/// Lê `git blame --porcelain`: cada linha começa com `<hash> <linha antiga> <linha final> [quantas]`; os dados
/// do commit (author, author-time, summary) só vêm na primeira vez que ele aparece; o texto vem depois de um tab.
pub fn parse_blame(out: &str) -> Blame {
    let mut b = Blame::default();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut current: Option<usize> = None;
    for line in out.lines() {
        if line.starts_with('\t') {
            if let Some(c) = current {
                b.lines.push(c);
            }
            continue;
        }
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        if key.len() == 40 && key.bytes().all(|c| c.is_ascii_hexdigit()) {
            let i = *index.entry(key.to_string()).or_insert_with(|| {
                b.commits.push(BlameCommit {
                    hash: key.to_string(),
                    author: String::new(),
                    time: 0,
                    summary: String::new(),
                    uncommitted: key.bytes().all(|c| c == b'0'),
                });
                b.commits.len() - 1
            });
            current = Some(i);
            continue;
        }
        let Some(c) = current.and_then(|i| b.commits.get_mut(i)) else { continue };
        match key {
            "author" => c.author = value.to_string(),
            "author-time" => c.time = value.parse().unwrap_or(0),
            "summary" => c.summary = value.to_string(),
            _ => {}
        }
    }
    b
}

// ---- Remoto ------------------------------------------------------------------

/// Roda um comando que fala com o remoto. Diferenças para [`run`]:
/// - o processo vai para uma sessão nova, sem terminal: o ssh não consegue pedir senha ou confirmar
///   a chave do servidor no terminal de onde a IDE foi aberta (ficaria esperando para sempre).
///   Sem terminal, ele usa o agente ssh ou o `SSH_ASKPASS` gráfico, ou falha com uma mensagem;
/// - tem tempo limite ([`NET_TIMEOUT`]);
/// - o erro devolve toda a saída de erro, para [`explain`] reconhecer o motivo.
fn run_net(dir: &Path, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(dir)
        .args(["-c", "color.ui=false", "-c", "core.quotepath=false"])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: `setsid` é seguro entre fork e exec (async-signal-safe) e não mexe na memória do processo.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    let mut child = cmd.spawn().map_err(|e| fl!("git-run-error", error = e.to_string()))?;
    let read = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut p) = pipe {
                let mut bytes = Vec::new();
                let _ = p.read_to_end(&mut bytes);
                text = String::from_utf8_lossy(&bytes).into_owned();
            }
            text
        })
    };
    let out = read(child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let err = read(child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                kill_group(&mut child);
                let _ = (out.join(), err.join());
                return Err(fl!("git-timeout", seconds = timeout.as_secs()));
            }
            Err(e) => return Err(e.to_string()),
        }
    };
    let (out, err) = (out.join().unwrap_or_default(), err.join().unwrap_or_default());
    if status.success() { Ok(format!("{out}{err}")) } else { Err(err) }
}

/// Mata o git e os filhos dele (ssh, credential helper): todos estão no grupo criado pelo `setsid`.
fn kill_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    // SAFETY: só manda um sinal para o grupo do processo filho.
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &child.id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Troca as mensagens mais comuns do git por uma explicação curta do que fazer.
pub fn explain(stderr: &str) -> String {
    let has = |s: &str| stderr.contains(s);
    if has("[rejected]") || has("fetch first") || has("non-fast-forward") {
        fl!("git-explain-rejected")
    } else if has("reconcile divergent branches") {
        fl!("git-explain-diverged")
    } else if has("CONFLICT") || has("Automatic merge failed") {
        fl!("git-explain-conflict")
    } else if has("would be overwritten by checkout") {
        fl!("git-explain-switch-overwritten")
    } else if has("would be overwritten") {
        fl!("git-explain-overwritten")
    } else if has("Authentication failed") || has("could not read Username") || has("could not read Password") {
        fl!("git-explain-auth")
    } else if has("Permission denied (publickey") || has("Host key verification failed") {
        fl!("git-explain-ssh")
    } else if has("Could not resolve host") || has("Could not read from remote repository") {
        fl!("git-explain-network")
    } else {
        match stderr.lines().rfind(|l| !l.trim().is_empty()) {
            Some(line) => line.trim().to_string(),
            None => fl!("git-unknown-error"),
        }
    }
}

pub fn remotes(repo: &Repo) -> Vec<String> {
    run(&repo.root, &["remote"], &[]).map(|o| o.lines().map(str::to_string).collect()).unwrap_or_default()
}

pub fn fetch(repo: &Repo) -> Result<String, String> {
    run_net(&repo.root, &["fetch", "--prune"], NET_TIMEOUT).map_err(|e| explain(&e))?;
    Ok(fl!("git-fetch-done"))
}

pub fn pull(repo: &Repo) -> Result<String, String> {
    let out = run_net(&repo.root, &["pull"], NET_TIMEOUT).map_err(|e| explain(&e))?;
    Ok(if out.contains("Already up to date") { fl!("git-up-to-date") } else { fl!("git-pull-done") })
}

/// `publish`: (remoto, branch) para criar o upstream (`push -u`) numa branch que ainda não tem.
pub fn push(repo: &Repo, publish: Option<(&str, &str)>) -> Result<String, String> {
    let mut args = vec!["push"];
    if let Some((remote, branch)) = publish {
        args.extend(["-u", remote, branch]);
    }
    let out = run_net(&repo.root, &args, NET_TIMEOUT).map_err(|e| explain(&e))?;
    Ok(match publish {
        Some((remote, branch)) => fl!("git-published", branch = branch, remote = remote),
        None if out.contains("Everything up-to-date") => fl!("git-nothing-to-push"),
        None => fl!("git-push-done"),
    })
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
    fn statuses_per_file() {
        use tea_core::FileStatus as F;
        let out = "1 .M N... 100644 100644 100644 aaa bbb src/main.rs\0\
1 A. N... 000000 100644 100644 000 ccc novo.rs\0\
1 AM N... 000000 100644 100644 000 ccc novo2.rs\0\
1 .D N... 100644 100644 000000 aaa aaa apagado.rs\0\
u UU N... 100644 100644 100644 100644 a b c conflito.rs\0\
? pasta nova/\0! target/\0";
        let st = parse_status(out);
        assert_eq!(st.ignored, ["target/"]);
        assert_eq!(st.entries.len(), 6, "ignorado não entra no painel");
        let repo = Repo { root: PathBuf::from("/r"), git_dir: PathBuf::from("/r/.git") };
        let map = file_statuses(&repo, &st);
        let at = |p: &str| map.get(Path::new(p)).copied();
        assert_eq!(at("/r/src/main.rs"), Some(F::Modified));
        assert_eq!(at("/r/novo.rs"), Some(F::Added));
        assert_eq!(at("/r/novo2.rs"), Some(F::Added), "novo e mexido depois continua novo");
        assert_eq!(at("/r/apagado.rs"), Some(F::Deleted));
        assert_eq!(at("/r/conflito.rs"), Some(F::Conflict));
        assert_eq!(at("/r/pasta nova"), Some(F::Untracked), "sem a barra do fim");
        assert_eq!(at("/r/target"), Some(F::Ignored));
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
    /// Dois clones de um remoto local (bare): publicar, push, pull, fetch e push recusado.
    #[test]
    fn remote_flow() {
        let base = std::env::temp_dir().join(format!("idetea-git-remote-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        if run(&base, &["init", "-q", "--bare", "-b", "main", "remote.git"], &[]).is_err() {
            return; // sem git na máquina
        }
        let clone = |name: &str| {
            run(&base, &["clone", "-q", "remote.git", name], &[]).unwrap();
            let dir = base.join(name);
            for (k, v) in [("user.email", "t@t"), ("user.name", "t"), ("commit.gpgsign", "false"), ("pull.rebase", "false")] {
                run(&dir, &["config", k, v], &[]).unwrap();
            }
            discover(&dir).unwrap()
        };
        let a = clone("a");
        let commit_file = |repo: &Repo, file: &str, text: &str| {
            std::fs::write(repo.root.join(file), text).unwrap();
            stage_all(repo).unwrap();
            commit(repo, file).unwrap();
        };

        commit_file(&a, "x.txt", "1\n");
        assert_eq!(remotes(&a), ["origin"]);
        assert_eq!(push(&a, None).unwrap(), "push feito");
        assert_eq!(push(&a, None).unwrap(), "nada para enviar");

        // Branch nova: sem upstream até publicar.
        run(&a.root, &["switch", "-q", "-c", "nova"], &[]).unwrap();
        commit_file(&a, "n.txt", "n\n");
        assert_eq!(status(&a).unwrap().upstream, None);
        assert_eq!(push(&a, Some(("origin", "nova"))).unwrap(), "branch nova publicada em origin");
        assert_eq!(status(&a).unwrap().upstream.as_deref(), Some("origin/nova"));
        run(&a.root, &["switch", "-q", "main"], &[]).unwrap();

        let b = clone("b");
        commit_file(&b, "y.txt", "2\n");
        assert_eq!(push(&b, None).unwrap(), "push feito");

        // `a` está atrás: fetch mostra, push é recusado, pull traz.
        fetch(&a).unwrap();
        assert_eq!(status(&a).unwrap().behind, 1);
        commit_file(&a, "z.txt", "3\n");
        assert_eq!(push(&a, None).unwrap_err(), "o remoto tem commits que você não tem: faça pull antes");
        assert_eq!(pull(&a).unwrap(), "pull feito");
        assert!(a.root.join("y.txt").is_file());
        assert_eq!(pull(&a).unwrap(), "já estava atualizado");
        assert_eq!(push(&a, None).unwrap(), "push feito");
        let _ = std::fs::remove_dir_all(&base);
    }

    /// Um remoto que nunca responde: o tempo limite mata o git e os filhos dele (que seguram a saída aberta).
    #[test]
    fn remote_timeout_kills_children() {
        let dir = std::env::temp_dir();
        if run(&dir, &["--version"], &[]).is_err() {
            return;
        }
        let started = Instant::now();
        let args = ["-c", "protocol.ext.allow=always", "ls-remote", "ext::sleep 30"];
        let err = run_net(&dir, &args, Duration::from_millis(500)).unwrap_err();
        assert!(err.starts_with("tempo esgotado"), "{err}");
        assert!(started.elapsed() < Duration::from_secs(5), "não esperou o sleep acabar");
    }

    #[test]
    fn explains_common_errors() {
        assert_eq!(
            explain("Permission denied (publickey).\nfatal: Could not read from remote repository."),
            "o ssh recusou a conexão: carregue a chave no agente (ssh-add) ou confirme o servidor no terminal"
        );
        assert_eq!(explain("hint: x\nfatal: algo novo\n"), "fatal: algo novo", "desconhecido: última linha");
    }

    #[test]
    fn blame_porcelain() {
        let a = "a".repeat(40);
        let z = "0".repeat(40);
        let out = format!(
            "{a} 1 1 2\nauthor Ana\nauthor-time 1700000000\nsummary Primeiro\nfilename x\n\tum\n{a} 2 2\n\tdois\n\
             {z} 3 3 1\nauthor Not Committed Yet\nauthor-time 1800000000\nsummary Version of x from x\n\ttrês\n"
        );
        let b = parse_blame(&out);
        assert_eq!(b.lines, [0, 0, 1]);
        let first = b.at(2).unwrap();
        assert_eq!((first.author.as_str(), first.time, first.summary.as_str(), first.uncommitted), ("Ana", 1_700_000_000, "Primeiro", false));
        assert!(b.at(3).unwrap().uncommitted);
        assert!(b.at(0).is_none() && b.at(4).is_none());
    }

    #[test]
    fn branch_list() {
        let out = "*\0refs/heads/main\0main\x002026-10-31\n \0refs/heads/dev\0dev\x002026-10-01\n \0refs/remotes/origin/HEAD\0origin\x002026-10-31\n \0refs/remotes/origin/main\0origin/main\x002026-10-31\n \0refs/remotes/origin/nova\0origin/nova\x002026-09-01\n";
        let b = parse_branches(out);
        let names: Vec<_> = b.iter().map(|b| (b.name.as_str(), b.remote, b.current)).collect();
        assert_eq!(names, [("main", false, true), ("dev", false, false), ("origin/nova", true, false)], "origin/main já tem local; HEAD sai");
    }

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
        assert_eq!(head_text(&repo, &dir.join("a.txt")), None, "ainda sem commit");

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
        assert_eq!(head_text(&repo, &repo.root.join("a.txt")).as_deref(), Some("um\n"));
        assert_eq!(head_text(&repo, &repo.root.join("novo.txt")), None, "fora do commit");
        assert_eq!(head_text(&repo, Path::new("/etc/hosts")), None, "fora do repositório");

        std::fs::write(dir.join("a.txt"), "um\ndois\n").unwrap();
        let st = status(&repo).unwrap();
        assert_eq!(st.entries[0].unstaged, Some('M'));
        assert!(diff(&repo, &st.entries[0], false).unwrap().contains("+dois"));

        discard(&repo, &st.entries[0]).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "um\n", "volta ao commit");
        assert!(status(&repo).unwrap().entries.is_empty());

        // Branches: criar, voltar, e a recusa quando a alteração colidiria com a outra branch.
        switch(&repo, "nova", true, false).unwrap();
        assert_eq!(status(&repo).unwrap().branch.as_deref(), Some("nova"));
        std::fs::write(dir.join("a.txt"), "na nova\n").unwrap();
        stage_all(&repo).unwrap();
        commit(&repo, "na nova").unwrap();
        switch(&repo, "main", false, false).unwrap();
        let list = branches(&repo).unwrap();
        assert_eq!(list.iter().filter(|b| b.current).map(|b| b.name.as_str()).collect::<Vec<_>>(), ["main"]);
        assert_eq!(list.len(), 2);
        std::fs::write(dir.join("a.txt"), "mexido\n").unwrap();
        let err = switch(&repo, "nova", false, false).unwrap_err();
        assert!(err.contains("commit"), "explica o que fazer: {err}");
        assert!(switch(&repo, "nova", true, false).is_err(), "já existe");

        // Blame: a linha commitada mostra o autor; a mexida no disco é "não commitada".
        let b = blame(&repo, &dir.join("a.txt")).unwrap();
        assert_eq!(b.lines.len(), 1);
        assert!(b.at(1).unwrap().uncommitted);
        run(&dir, &["checkout", "-q", "--", "a.txt"], &[]).unwrap();
        let b = blame(&repo, &dir.join("a.txt")).unwrap();
        assert_eq!((b.at(1).unwrap().author.as_str(), b.at(1).unwrap().summary.as_str()), ("t", "primeiro"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
