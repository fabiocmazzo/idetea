//! Linhas mudadas em relação a uma versão de referência (o último commit, quem passa é o app): as marcas
//! na margem. Compara linha a linha; a quebra de linha no fim do arquivo não conta.

use std::time::{Duration, Instant};

use similar::{Algorithm, DiffOp, capture_diff_slices_deadline};

/// Arquivos muito diferentes não travam a digitação: passou disso, o diff sai menos fino.
const DEADLINE: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkKind {
    Added,
    Modified,
    /// Linhas apagadas antes de `start` (nada no texto atual).
    Deleted,
}

/// Um trecho mudado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub kind: HunkKind,
    /// Primeira linha no texto atual (começa em 0).
    pub start: usize,
    /// Quantas linhas no texto atual (0 em `Deleted`).
    pub len: usize,
    /// Como eram as linhas (vazio em `Added`).
    pub old: Vec<String>,
}

impl Hunk {
    /// A linha está no trecho? `Deleted` fica na linha de baixo do que foi apagado (ou na última).
    pub fn covers(&self, line: usize, last_line: usize) -> bool {
        match self.kind {
            HunkKind::Deleted => line == self.start.min(last_line),
            _ => (self.start..self.start + self.len).contains(&line),
        }
    }
}

pub fn hunks(old: &str, new: &str) -> Vec<Hunk> {
    hunks_until(old, new, Some(Instant::now() + DEADLINE))
}

/// [`hunks`] com outro prazo (`None`: sem prazo, o diff sempre fino; para os testes não dependerem da máquina).
fn hunks_until(old: &str, new: &str, deadline: Option<Instant>) -> Vec<Hunk> {
    let old: Vec<&str> = old.lines().collect();
    let new: Vec<&str> = new.lines().collect();
    let ops = capture_diff_slices_deadline(Algorithm::Myers, &old, &new, deadline);
    let mut out = Vec::new();
    // Juntar apagar + inserir seguidos num trecho "mudado".
    let mut group: Option<(usize, usize, usize, usize)> = None; // (início antigo, fim antigo, início novo, fim novo)
    let flush = |g: Option<(usize, usize, usize, usize)>, out: &mut Vec<Hunk>| {
        let Some((os, oe, ns, ne)) = g else { return };
        let kind = match (oe - os, ne - ns) {
            (0, _) => HunkKind::Added,
            (_, 0) => HunkKind::Deleted,
            _ => HunkKind::Modified,
        };
        out.push(Hunk { kind, start: ns, len: ne - ns, old: old[os..oe].iter().map(|s| s.to_string()).collect() });
    };
    for op in ops {
        let (o, n) = (op.old_range(), op.new_range());
        if let DiffOp::Equal { .. } = op {
            flush(group.take(), &mut out);
            continue;
        }
        group = Some(match group {
            Some((os, _, ns, _)) => (os, o.end, ns, n.end),
            None => (o.start, o.end, n.start, n.end),
        });
    }
    flush(group, &mut out);
    out
}

/// O texto com o trecho voltando a ser como era.
pub fn revert(text: &str, hunk: &Hunk) -> String {
    let mut lines: Vec<&str> = text.split('\n').collect();
    let end = (hunk.start + hunk.len).min(lines.len());
    let start = hunk.start.min(end);
    lines.splice(start..end, hunk.old.iter().map(String::as_str));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_and_revert() {
        let old = "a\nb\nc\nd\ne\n";
        let new = "a\nB\nc\nnova\nd\n";
        let h = hunks_until(old, new, None);
        let shape: Vec<_> = h.iter().map(|h| (h.kind, h.start, h.len, h.old.clone())).collect();
        assert_eq!(
            shape,
            [
                (HunkKind::Modified, 1, 1, vec!["b".to_string()]),
                (HunkKind::Added, 3, 1, vec![]),
                (HunkKind::Deleted, 5, 0, vec!["e".to_string()]),
            ]
        );
        // Desfazer cada trecho volta só ele.
        assert_eq!(revert(new, &h[0]), "a\nb\nc\nnova\nd\n");
        assert_eq!(revert(new, &h[1]), "a\nB\nc\nd\n");
        assert_eq!(revert(new, &h[2]), "a\nB\nc\nnova\nd\ne\n");
        assert!(h[2].covers(4, 4), "apagado no fim fica na última linha");
        assert!(hunks(old, old).is_empty());
        assert!(hunks("a\nb", "a\nb\n").is_empty(), "quebra de linha final não conta");
    }
}
