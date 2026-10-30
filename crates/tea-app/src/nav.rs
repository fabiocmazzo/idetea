//! Histórico de navegação: de onde se saltou (definição, referência, Ctrl+P, painéis), para
//! voltar (`Alt+←`) e avançar (`Alt+→`), como num navegador.

use std::path::{Path, PathBuf};

/// Quantos lugares guardar para voltar.
const LIMIT: usize = 100;

/// Um lugar no código. Linha e coluna começam em 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spot {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Default)]
pub struct History {
    back: Vec<Spot>,
    forward: Vec<Spot>,
}

impl History {
    /// Vai saltar de `here` para `path` (na `line`, se souber). Saltos na mesma linha não contam.
    pub fn remember(&mut self, here: Spot, path: &Path, line: Option<usize>) {
        if here.path == path && line.is_none_or(|l| l == here.line) {
            return;
        }
        if self.back.last() != Some(&here) {
            self.back.push(here);
        }
        if self.back.len() > LIMIT {
            self.back.remove(0);
        }
        self.forward.clear();
    }

    /// Para onde ir ao voltar (`forward = false`) ou avançar, guardando `here` do outro lado.
    /// Lugares para os quais `exists` diz não (arquivo apagado) são pulados.
    pub fn step(&mut self, forward: bool, here: Option<Spot>, exists: impl Fn(&Path) -> bool) -> Option<Spot> {
        let (from, to) = if forward { (&mut self.forward, &mut self.back) } else { (&mut self.back, &mut self.forward) };
        while let Some(target) = from.pop() {
            if exists(&target.path) {
                to.extend(here);
                return Some(target);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spot(path: &str, line: usize) -> Spot {
        Spot { path: path.into(), line, column: 1 }
    }

    #[test]
    fn back_and_forward_like_a_browser() {
        let mut h = History::default();
        let yes = |_: &Path| true;
        h.remember(spot("/a", 10), Path::new("/b"), Some(3)); // a:10 → b:3
        h.remember(spot("/b", 3), Path::new("/c"), Some(7)); // b:3 → c:7
        assert_eq!(h.step(false, Some(spot("/c", 7)), yes), Some(spot("/b", 3)));
        assert_eq!(h.step(false, Some(spot("/b", 3)), yes), Some(spot("/a", 10)));
        assert_eq!(h.step(false, Some(spot("/a", 10)), yes), None, "nada mais para voltar");
        assert_eq!(h.step(true, Some(spot("/a", 10)), yes), Some(spot("/b", 3)));
        // Saltar para outro lugar apaga o "avançar".
        h.remember(spot("/b", 3), Path::new("/d"), Some(1));
        assert_eq!(h.step(true, Some(spot("/d", 1)), yes), None);
    }

    #[test]
    fn ignores_same_line_and_skips_deleted_files() {
        let mut h = History::default();
        h.remember(spot("/a", 5), Path::new("/a"), Some(5));
        h.remember(spot("/a", 5), Path::new("/a"), None);
        assert_eq!(h.step(false, None, |_| true), None, "mesma linha ou mesmo arquivo sem linha não contam");
        h.remember(spot("/apagado", 1), Path::new("/a"), Some(2));
        h.remember(spot("/a", 2), Path::new("/b"), Some(1));
        assert_eq!(h.step(false, None, |p| p != Path::new("/apagado")), Some(spot("/a", 2)));
        assert_eq!(h.step(false, None, |p| p != Path::new("/apagado")), None, "o apagado é pulado");
    }

    #[test]
    fn keeps_at_most_the_limit() {
        let mut h = History::default();
        for i in 0..LIMIT + 20 {
            h.remember(spot("/a", i + 1), Path::new("/b"), Some(1));
        }
        assert_eq!(h.back.len(), LIMIT);
        assert_eq!(h.back[0].line, 21, "os mais velhos saem");
    }
}
