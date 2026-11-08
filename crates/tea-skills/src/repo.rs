//! A fonte das skills: um repositório git, numa pasta de cache. Tudo é lido do histórico (`ls-tree`, `cat-file`):
//! a cópia de trabalho nunca é usada, e nada do repositório roda.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// De onde vêm as skills, como o usuário escreveu (`dono/repo`, uma URL ou uma pasta).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Como foi escrita (vai para o `skills.toml`).
    pub spec: String,
    /// O que o `git clone` recebe.
    pub url: String,
}

impl Source {
    /// `dono/repo` é do GitHub; uma URL (`https://`, `git@`, `file://`) vai como está; uma pasta que existe, pelo
    /// caminho absoluto.
    pub fn parse(spec: &str, cwd: &Path) -> Option<Self> {
        let spec = spec.trim();
        if spec.is_empty() || spec.starts_with('-') {
            return None;
        }
        let url = if spec.contains("://") || spec.starts_with("git@") {
            spec.to_string()
        } else if cwd.join(spec).is_dir() {
            cwd.join(spec)
                .canonicalize()
                .ok()?
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string()
        } else {
            let mut parts = spec.split('/');
            let (owner, repo) = (parts.next()?, parts.next()?);
            let ok = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) && s != "..";
            if parts.next().is_some() || !ok(owner) || !ok(repo) {
                return None;
            }
            format!("https://github.com/{owner}/{repo}.git")
        };
        Some(Self { spec: spec.to_string(), url })
    }

    /// A pasta do cache desta fonte: legível e sem misturar duas fontes.
    pub fn cache_dir(&self, cache: &Path) -> PathBuf {
        let readable: String = self
            .url
            .trim_end_matches(".git")
            .rsplit(['/', ':'])
            .take(2)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("-")
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let hash = self.url.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
        cache.join(format!("{readable}-{:08x}", hash as u32))
    }
}

/// O repositório no cache.
#[derive(Debug, Clone)]
pub struct Repo {
    dir: PathBuf,
}

/// Um arquivo do repositório num commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// `100644`, `100755` (executável), `120000` (symlink) ou `160000` (submódulo).
    pub mode: String,
    pub object: String,
    pub size: u64,
    pub path: String,
}

impl Entry {
    pub fn is_file(&self) -> bool {
        self.mode == "100644" || self.mode == "100755"
    }
}

fn git(dir: Option<&Path>, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut cmd = Command::new("git");
    if let Some(dir) = dir {
        cmd.arg("-C").arg(dir);
    }
    let out = cmd
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() { format!("git {}: {}", args.join(" "), out.status) } else { err })
    }
}

impl Repo {
    /// Clona a fonte no cache, ou busca o que mudou nela.
    pub fn fetch(source: &Source, cache: &Path) -> Result<Self, String> {
        let dir = source.cache_dir(cache);
        if dir.join("HEAD").is_file() || dir.join(".git").exists() {
            git(Some(&dir), &["fetch", "--quiet", "--prune", "origin", "+HEAD:refs/remotes/origin/HEAD"])?;
        } else {
            std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
            let target = dir.to_string_lossy().into_owned();
            git(None, &["clone", "--quiet", "--bare", "--", &source.url, &target])?;
            git(Some(&dir), &["fetch", "--quiet", "origin", "+HEAD:refs/remotes/origin/HEAD"])?;
        }
        Ok(Self { dir })
    }

    /// O commit mais novo da fonte.
    pub fn head(&self) -> Result<String, String> {
        let out = git(Some(&self.dir), &["rev-parse", "--verify", "refs/remotes/origin/HEAD^{commit}"])?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    }

    /// Os arquivos de `commit` (abaixo de `prefix`, ou todos).
    pub fn entries(&self, commit: &str, prefix: Option<&str>) -> Result<Vec<Entry>, String> {
        let mut args = vec!["ls-tree", "-r", "-z", "--long", commit];
        if let Some(p) = prefix {
            args.extend(["--", p]);
        }
        let out = git(Some(&self.dir), &args)?;
        Ok(out
            .split(|b| *b == 0)
            .filter_map(|line| {
                let line = std::str::from_utf8(line).ok()?;
                let (meta, path) = line.split_once('\t')?;
                let mut m = meta.split_whitespace();
                let (mode, _kind, object, size) = (m.next()?, m.next()?, m.next()?, m.next()?);
                Some(Entry { mode: mode.into(), object: object.into(), size: size.parse().unwrap_or(0), path: path.into() })
            })
            .collect())
    }

    pub fn read(&self, object: &str) -> Result<Vec<u8>, String> {
        git(Some(&self.dir), &["cat-file", "blob", object])
    }

    /// O que mudou em `path` de `old` para `new`: o resumo e o diff inteiro.
    pub fn diff(&self, old: &str, new: &str, path: &str) -> Result<(String, String), String> {
        let stat = git(Some(&self.dir), &["diff", "--no-color", "--stat", old, new, "--", path])?;
        let full = git(Some(&self.dir), &["diff", "--no-color", old, new, "--", path])?;
        Ok((String::from_utf8_lossy(&stat).into_owned(), String::from_utf8_lossy(&full).into_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sources() {
        let cwd = std::env::temp_dir();
        let gh = Source::parse("anthropics/skills", &cwd).unwrap();
        assert_eq!(gh.url, "https://github.com/anthropics/skills.git");
        assert!(
            gh.cache_dir(Path::new("/c"))
                .to_string_lossy()
                .replace('\\', "/")
                .starts_with("/c/anthropics-skills-")
        );
        assert_eq!(Source::parse("https://example.com/a/b.git", &cwd).unwrap().url, "https://example.com/a/b.git");
        for bad in ["", "-x", "a/b/c", "a/..", "a/", "a b/c"] {
            assert_eq!(Source::parse(bad, &cwd), None, "{bad}");
        }
    }
}
