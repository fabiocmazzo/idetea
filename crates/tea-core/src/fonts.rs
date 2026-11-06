//! Fonte do código: editor (com os números de linha), terminal e as caixas que mostram código.
//!
//! Vem de `[editor] font`, `font_weight` e `line_height`. A JetBrains Mono (SIL OFL 1.1, licença em
//! `assets/fonts/OFL.txt`) vem embutida e é o padrão: todo mundo tem uma fonte boa sem instalar nada.
//! O app carrega [`EMBEDDED`] com [`load_embedded`] antes de abrir a janela; as fontes do sistema o
//! iced já enxerga.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use iced::Font;
use iced::font::{Family, Weight};

use crate::config::EditorConfig;

/// Os arquivos da fonte embutida (regular e negrito).
pub const EMBEDDED: [&[u8]; 2] = [
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
];

/// Põe a fonte embutida no motor de texto do iced. Tem que vir antes de abrir qualquer editor: o widget
/// mede a largura do caractere quando a fonte é definida, e sem a fonte carregada ele mede uma fonte
/// substituta (o realce de sintaxe e os parênteses coloridos saem deslocados). O `.font()` da aplicação
/// carrega depois de `boot`, tarde demais. Carregar de novo não faz nada.
pub fn load_embedded() {
    use iced::advanced::graphics::text::font_system;
    let mut system = font_system().write().unwrap_or_else(|p| p.into_inner());
    for bytes in EMBEDDED {
        system.load_font(std::borrow::Cow::Borrowed(bytes));
    }
}

/// Nome da fonte embutida, o padrão de `[editor] font`.
pub const DEFAULT_FAMILY: &str = "JetBrains Mono";

/// Altura da linha, em múltiplos do tamanho da fonte.
pub const LINE_HEIGHT_DEFAULT: f32 = 1.45;
pub const LINE_HEIGHT_MIN: f32 = 1.0;
pub const LINE_HEIGHT_MAX: f32 = 2.5;

/// Pesos aceitos em `font_weight`, do mais fino ao mais grosso.
pub const WEIGHTS: [&str; 9] =
    ["thin", "extra-light", "light", "normal", "medium", "semibold", "bold", "extra-bold", "black"];

/// A fonte do código já resolvida.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeFont {
    pub font: Font,
    /// Múltiplo do tamanho da fonte.
    pub line_height: f32,
}

impl Default for CodeFont {
    fn default() -> Self {
        Self { font: Font::with_name(DEFAULT_FAMILY), line_height: LINE_HEIGHT_DEFAULT }
    }
}

impl CodeFont {
    /// A fonte do config. Uma fonte que não está instalada vira a embutida (o aviso sai em
    /// [`problem`]); um peso desconhecido vira o normal.
    pub fn from_config(c: &EditorConfig) -> Self {
        let name = c.font.trim();
        let family = if name.eq_ignore_ascii_case("monospace") {
            Family::Monospace
        } else if name.is_empty() || name.eq_ignore_ascii_case(DEFAULT_FAMILY) || !installed(name) {
            Family::Name(DEFAULT_FAMILY)
        } else {
            Family::Name(intern(name))
        };
        let weight = weight(&c.font_weight).unwrap_or(Weight::Normal);
        let line_height = if c.line_height.is_finite() {
            c.line_height.clamp(LINE_HEIGHT_MIN, LINE_HEIGHT_MAX)
        } else {
            LINE_HEIGHT_DEFAULT
        };
        Self { font: Font { family, weight, ..Font::DEFAULT }, line_height }
    }

    /// A mesma fonte em negrito (o trecho ativo na dica de parâmetros).
    pub fn bold(self) -> Font {
        Font { weight: Weight::Bold, ..self.font }
    }

    /// Altura da linha em pixels para este tamanho (inteira, para o texto não ficar borrado).
    pub fn line_px(self, size: f32) -> f32 {
        (size * self.line_height).round().max(size)
    }
}

/// O que há de errado na fonte do config, para avisar junto dos outros problemas do config.
pub fn problem(c: &EditorConfig) -> Option<String> {
    let name = c.font.trim();
    let bad_font = !name.is_empty()
        && !name.eq_ignore_ascii_case(DEFAULT_FAMILY)
        && !name.eq_ignore_ascii_case("monospace")
        && !installed(name);
    let bad_weight = weight(&c.font_weight).is_none();
    match (bad_font, bad_weight) {
        (true, _) => Some(fl!("config-font-missing", name = name, default = DEFAULT_FAMILY)),
        (_, true) => Some(fl!("config-font-weight", name = c.font_weight.as_str(), choices = WEIGHTS.join(", "))),
        _ => None,
    }
}

/// Peso pelo nome (`"normal"` e `"regular"` são o mesmo; `"semi-bold"` também vale).
pub fn weight(name: &str) -> Option<Weight> {
    let n = name.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    Some(match n.as_str() {
        "thin" => Weight::Thin,
        "extra-light" | "extralight" => Weight::ExtraLight,
        "light" => Weight::Light,
        "" | "normal" | "regular" => Weight::Normal,
        "medium" => Weight::Medium,
        "semibold" | "semi-bold" => Weight::Semibold,
        "bold" => Weight::Bold,
        "extra-bold" | "extrabold" => Weight::ExtraBold,
        "black" => Weight::Black,
        _ => return None,
    })
}

/// As fontes monoespaçadas instaladas, por nome, sem repetir (a embutida primeiro). Lida uma vez:
/// uma fonte instalada com a IDE aberta aparece ao reiniciar.
pub fn monospace_families() -> &'static [String] {
    static LIST: OnceLock<Vec<String>> = OnceLock::new();
    LIST.get_or_init(|| {
        let mut names: Vec<String> = system_faces()
            .into_iter()
            .filter(|(_, mono)| *mono)
            .map(|(name, _)| name)
            .filter(|n| !n.eq_ignore_ascii_case(DEFAULT_FAMILY))
            .collect();
        names.sort_by_key(|n| n.to_lowercase());
        names.dedup();
        names.insert(0, DEFAULT_FAMILY.to_string());
        names
    })
}

/// A fonte está instalada (qualquer uma, monoespaçada ou não).
pub fn installed(name: &str) -> bool {
    static ALL: OnceLock<Vec<String>> = OnceLock::new();
    ALL.get_or_init(|| system_faces().into_iter().map(|(n, _)| n.to_lowercase()).collect())
        .contains(&name.trim().to_lowercase())
}

/// Nome e se é monoespaçada, de cada fonte que o iced conhece.
fn system_faces() -> Vec<(String, bool)> {
    use iced::advanced::graphics::text::font_system;
    let mut system = match font_system().write() {
        Ok(s) => s,
        Err(poisoned) => poisoned.into_inner(),
    };
    system
        .raw()
        .db()
        .faces()
        .flat_map(|f| f.families.iter().map(move |(name, _)| (name.clone(), f.monospaced)))
        .collect()
}

/// `Font::with_name` pede `&'static str`: guarda cada nome uma vez só.
fn intern(name: &str) -> &'static str {
    static NAMES: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let mut names = NAMES.get_or_init(Default::default).lock().unwrap_or_else(|p| p.into_inner());
    names.entry(name.to_string()).or_insert_with(|| Box::leak(name.to_string().into_boxed_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_font_is_truetype() {
        for bytes in EMBEDDED {
            assert!(bytes.len() > 100_000);
            assert_eq!(&bytes[..4], &[0, 1, 0, 0], "TrueType");
        }
    }

    #[test]
    fn config_resolves_to_a_font() {
        let mut c = EditorConfig::default();
        assert_eq!(CodeFont::from_config(&c), CodeFont::default());
        assert_eq!(problem(&c), None);

        c.font = "monospace".into();
        c.font_weight = "Semi Bold".into();
        c.line_height = 9.0;
        let f = CodeFont::from_config(&c);
        assert_eq!(f.font.family, Family::Monospace);
        assert_eq!(f.font.weight, Weight::Semibold);
        assert_eq!(f.line_height, LINE_HEIGHT_MAX);
        assert_eq!(f.line_px(14.0), 35.0);

        c.font = "Fonte Que Não Existe 123".into();
        assert_eq!(CodeFont::from_config(&c).font.family, Family::Name(DEFAULT_FAMILY));
        assert!(problem(&c).unwrap().contains("Fonte Que Não Existe 123"));

        c.font = String::new();
        c.font_weight = "gordinha".into();
        assert_eq!(CodeFont::from_config(&c).font.weight, Weight::Normal);
        assert!(problem(&c).unwrap().contains("gordinha"));
    }

    #[test]
    fn the_same_name_is_interned_once() {
        assert!(std::ptr::eq(intern("Abc Mono"), intern("Abc Mono")));
    }
}
