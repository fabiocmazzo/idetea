//! Estado da interface que sobrevive entre execuções (tamanho dos painéis).
//! Fica em `$XDG_STATE_HOME/idetea/state.toml` (ou `~/.local/state/...`), separado
//! do config: é a IDE que escreve aqui, não o usuário.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DEFAULT_SIDE: f32 = 260.0;
pub const DEFAULT_BOTTOM: f32 = 260.0;
pub const MIN_SIDE: f32 = 140.0;
pub const MIN_BOTTOM: f32 = 80.0;
/// Espaço que sempre sobra para o editor.
pub const MIN_EDITOR: f32 = 240.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiState {
    pub left_width: f32,
    pub right_width: f32,
    pub bottom_height: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self { left_width: DEFAULT_SIDE, right_width: DEFAULT_SIDE, bottom_height: DEFAULT_BOTTOM }
    }
}

impl UiState {
    /// Lê o arquivo; qualquer problema volta aos padrões (estado não é importante a ponto de avisar).
    pub fn load(path: Option<&Path>) -> Self {
        path.and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| toml::from_str::<UiState>(&t).ok())
            .map(Self::sanitized)
            .unwrap_or_default()
    }

    pub fn save(&self, path: Option<&Path>) {
        let Some(path) = path else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = toml::to_string(self) {
            let _ = std::fs::write(path, text);
        }
    }

    fn sanitized(self) -> Self {
        let ok = |v: f32, min: f32, default: f32| if v.is_finite() && v >= min { v } else { default };
        Self {
            left_width: ok(self.left_width, MIN_SIDE, DEFAULT_SIDE),
            right_width: ok(self.right_width, MIN_SIDE, DEFAULT_SIDE),
            bottom_height: ok(self.bottom_height, MIN_BOTTOM, DEFAULT_BOTTOM),
        }
    }
}

/// Novo tamanho de um painel: soma `delta`, sem passar do mínimo nem do espaço disponível.
pub fn resize(current: f32, delta: f32, min: f32, available: Option<f32>) -> f32 {
    let max = available.map_or(f32::MAX, |a| (a - MIN_EDITOR).max(min));
    (current + delta).clamp(min, max)
}

pub fn state_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("state")))?;
    Some(base.join("idetea").join("state.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_bad_values() {
        let path = std::env::temp_dir().join(format!("idetea-state-{}", std::process::id())).join("state.toml");
        let s = UiState { left_width: 300.0, right_width: 200.0, bottom_height: 400.0 };
        s.save(Some(&path));
        assert_eq!(UiState::load(Some(&path)), s);
        std::fs::write(&path, "left_width = 5\nbottom_height = \"x\"").unwrap();
        assert_eq!(UiState::load(Some(&path)), UiState::default(), "arquivo estragado: padrões");
        std::fs::write(&path, "left_width = 10").unwrap();
        assert_eq!(UiState::load(Some(&path)).left_width, DEFAULT_SIDE, "abaixo do mínimo: padrão");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn resize_limits() {
        assert_eq!(resize(260.0, 40.0, MIN_SIDE, None), 300.0);
        assert_eq!(resize(150.0, -100.0, MIN_SIDE, None), MIN_SIDE);
        assert_eq!(resize(600.0, 500.0, MIN_SIDE, Some(1000.0)), 1000.0 - MIN_EDITOR, "sobra espaço para o editor");
        assert_eq!(resize(200.0, 10.0, MIN_SIDE, Some(300.0)), MIN_SIDE, "janela minúscula: fica no mínimo");
    }
}
