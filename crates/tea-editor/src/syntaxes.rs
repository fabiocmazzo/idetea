//! Registra no editor as sintaxes de `tea_core::syntaxes` (a do ZX e as do usuário).

use std::sync::OnceLock;

use iced_code_editor::{SyntaxRegistrationError, register_syntaxes};
use tea_core::syntaxes::Source;

/// Problemas do registro: (arquivo, erro). Ficam guardados para a IDE avisar depois de abrir.
static PROBLEMS: OnceLock<Vec<(String, String)>> = OnceLock::new();

/// Registra as sintaxes, pulando as que não são válidas. Chame uma vez, antes da primeira aba (o editor monta o
/// conjunto de sintaxes na primeira vez que colore um texto).
pub fn register(user_dir: Option<&std::path::Path>) {
    let (sources, mut problems) = tea_core::syntaxes::sources(user_dir);
    problems.extend(register_valid(sources));
    let _ = PROBLEMS.set(problems);
}

/// Registra todas as válidas; devolve (arquivo, erro) das outras. O registro é tudo ou nada, então uma inválida
/// sai da lista e o resto tenta de novo.
fn register_valid(mut sources: Vec<Source>) -> Vec<(String, String)> {
    let mut problems = Vec::new();
    loop {
        match register_syntaxes(sources.iter().map(|s| s.text.as_str())) {
            Ok(()) => return problems,
            Err(SyntaxRegistrationError::Parse { index, message }) if index < sources.len() => {
                problems.push((sources.remove(index).label, message));
            }
            Err(e) => {
                problems.push((String::new(), e.to_string()));
                return problems;
            }
        }
    }
}

/// O que deu errado no registro (vazio antes de [`register`]).
pub fn problems() -> &'static [(String, String)] {
    PROBLEMS.get().map(Vec::as_slice).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_language_pack_syntaxes_parse() {
        let (packs, errors) = tea_core::languages::builtins();
        assert!(errors.is_empty(), "{errors:?}");
        let sources = packs.into_iter().flat_map(|p| p.syntaxes).collect();
        assert!(register_valid(sources).is_empty());
    }
}
