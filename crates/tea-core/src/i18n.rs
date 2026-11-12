//! Idioma da interface, com [Fluent](https://projectfluent.org).
//!
//! Cada crate tem os próprios textos em `i18n/<idioma>/<nome-do-crate>.ftl` e um `i18n.toml`
//! (`fallback_language = "pt-BR"`). O português é a língua de origem: a macro `fl!` confere na
//! compilação que a chave existe nele. Inglês, espanhol, francês, alemão, italiano, japonês e
//! chinês simplificado traduzem a interface inteira. O português continua sendo a reserva.
//!
//! Num crate:
//!
//! ```ignore
//! tea_core::i18n_loader!();
//! macro_rules! fl { ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) }; }
//!
//! text(fl!("salvar"));
//! text(fl!("mudou-no-disco", name = file));
//! ```
//!
//! O idioma é escolhido uma vez, ao abrir ([`init`]); trocar vale ao reiniciar, como no VS Code.

use std::sync::OnceLock;

pub use i18n_embed::LanguageLoader;
pub use i18n_embed::fluent::FluentLanguageLoader;
pub use i18n_embed_fl::fl;
pub use rust_embed::{self, RustEmbed};

/// Idiomas da interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    PortugueseBr,
    English,
    Spanish,
    French,
    German,
    Italian,
    Japanese,
    ChineseSimplified,
}

impl Language {
    pub const ALL: [Language; 8] = [
        Language::PortugueseBr,
        Language::English,
        Language::Spanish,
        Language::French,
        Language::German,
        Language::Italian,
        Language::Japanese,
        Language::ChineseSimplified,
    ];

    /// Todos os idiomas da lista traduzem a interface inteira.
    pub const COMPLETE: [Language; 8] = Self::ALL;

    /// Etiqueta do idioma (o nome da pasta dos `.ftl` e o valor no config).
    pub fn tag(self) -> &'static str {
        match self {
            Language::PortugueseBr => "pt-BR",
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Italian => "it",
            Language::Japanese => "ja",
            Language::ChineseSimplified => "zh-CN",
        }
    }

    /// Nome no próprio idioma (assim cada um acha o seu na lista).
    pub fn native_name(self) -> &'static str {
        match self {
            Language::PortugueseBr => "Português (Brasil)",
            Language::English => "English",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Italian => "Italiano",
            Language::Japanese => "日本語",
            Language::ChineseSimplified => "中文（简体）",
        }
    }

    /// A interface inteira está traduzida.
    pub fn is_complete(self) -> bool {
        Self::COMPLETE.contains(&self)
    }

    /// `pt_BR.UTF-8`, `pt-PT`, `en_US`, `es-419`, `zh-Hans`... → o idioma mais próximo que temos.
    /// Chinês tradicional (`zh-TW`, `zh-HK`, `zh-Hant`) não vira simplificado: cai no inglês.
    pub fn from_tag(tag: &str) -> Option<Self> {
        let lower = tag.trim().replace('_', "-").to_ascii_lowercase();
        let primary = lower.split(['-', '.', '@']).next().unwrap_or_default();
        if primary == "zh" {
            if lower.contains("hant")
                || lower.starts_with("zh-tw")
                || lower.starts_with("zh-hk")
                || lower.starts_with("zh-mo")
            {
                return None;
            }
            return Some(Language::ChineseSimplified);
        }
        match primary {
            "pt" => Some(Language::PortugueseBr),
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            "fr" => Some(Language::French),
            "de" => Some(Language::German),
            "it" => Some(Language::Italian),
            "ja" => Some(Language::Japanese),
            _ => None,
        }
    }

    fn identifier(self) -> i18n_embed::unic_langid::LanguageIdentifier {
        self.tag().parse().expect("etiqueta de idioma válida")
    }
}

/// Valor de `[ui] language` → idioma. `"auto"` (ou vazio, ou um idioma que não temos) segue o
/// sistema; sistema sem idioma conhecido (ex.: `LANG=C`) fica em inglês.
pub fn resolve(setting: &str, system: Option<&str>) -> Language {
    let setting = setting.trim();
    if !setting.is_empty()
        && !setting.eq_ignore_ascii_case("auto")
        && let Some(lang) = Language::from_tag(setting)
    {
        return lang;
    }
    system
        .and_then(Language::from_tag)
        .unwrap_or(Language::English)
}

/// Idioma do sistema (`LANG`, `LC_ALL`...).
pub fn system_locale() -> Option<String> {
    sys_locale::get_locale()
}

static CURRENT: OnceLock<Language> = OnceLock::new();

/// Escolhe o idioma da interface. Só a primeira chamada vale (o idioma não muda com a IDE aberta).
pub fn init(setting: &str) -> Language {
    *CURRENT.get_or_init(|| resolve(setting, system_locale().as_deref()))
}

/// Idioma da interface. Sem [`init`] (testes), português: os textos de hoje.
pub fn current() -> Language {
    CURRENT.get().copied().unwrap_or(Language::PortugueseBr)
}

/// Carrega os textos de um crate no idioma atual. O português fica por baixo, como reserva.
/// Use pela macro [`i18n_loader!`](crate::i18n_loader).
pub fn load(domain: &str, assets: &dyn i18n_embed::I18nAssets) -> FluentLanguageLoader {
    load_in(domain, assets, current())
}

/// Como [`load`], num idioma escolhido (testes).
pub fn load_in(
    domain: &str,
    assets: &dyn i18n_embed::I18nAssets,
    language: Language,
) -> FluentLanguageLoader {
    let loader = FluentLanguageLoader::new(domain, Language::PortugueseBr.identifier());
    let mut chain = Vec::new();
    let primary = language.identifier();
    if asset_has(assets, domain, &primary) {
        chain.push(primary);
    }
    // Um pacote incompleto (não deve acontecer: o teste exige todas as chaves) cai no inglês
    // antes do português.
    if !language.is_complete() {
        let english = Language::English.identifier();
        if asset_has(assets, domain, &english) && !chain.contains(&english) {
            chain.push(english);
        }
    }
    if chain.is_empty() || loader.load_languages(assets, &chain).is_err() {
        let _ = loader.load_fallback_language(assets);
    }
    // Sem as marcas Unicode de isolamento em volta dos valores (o iced desenharia quadradinhos).
    loader.set_use_isolating(false);
    loader
}

fn asset_has(
    assets: &dyn i18n_embed::I18nAssets,
    domain: &str,
    language: &i18n_embed::unic_langid::LanguageIdentifier,
) -> bool {
    let path = format!("{language}/{domain}.ftl");
    !assets.get_files(&path).is_empty()
}

/// Define o `LOADER` do crate, com os `.ftl` de `i18n/` embutidos no executável.
#[macro_export]
macro_rules! i18n_loader {
    () => {
        #[derive($crate::i18n::RustEmbed)]
        #[folder = "i18n/"]
        #[crate_path = "tea_core::i18n::rust_embed"]
        struct Localizations;

        /// Textos deste crate no idioma da interface.
        pub(crate) static LOADER: std::sync::LazyLock<$crate::i18n::FluentLanguageLoader> =
            std::sync::LazyLock::new(|| $crate::i18n::load(env!("CARGO_PKG_NAME"), &Localizations));
    };
}

use fluent_syntax::ast::{Expression, InlineExpression, Pattern, PatternElement};
use std::collections::BTreeSet;

/// Valores (`$nome`) usados num texto, inclusive dentro de plurais.
fn vars(pattern: &Pattern<&str>, out: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            expr(expression, out);
        }
    }
}
fn expr(e: &Expression<&str>, out: &mut BTreeSet<String>) {
    match e {
        Expression::Select { selector, variants } => {
            inline(selector, out);
            for v in variants {
                vars(&v.value, out);
            }
        }
        Expression::Inline(i) => inline(i, out),
    }
}
fn inline(i: &InlineExpression<&str>, out: &mut BTreeSet<String>) {
    match i {
        InlineExpression::VariableReference { id } => {
            out.insert(id.name.to_string());
        }
        InlineExpression::Placeable { expression } => expr(expression, out),
        InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline(a, out);
            }
        }
        _ => {}
    }
}

/// Texto em que cada valor `{ $a-b }` vira o marcador `{a.b}`. É o jeito do guia da ajuda citar
/// comandos: o marcador depois vira o atalho atual do usuário.
pub fn get_with_markers(loader: &FluentLanguageLoader, id: &str) -> String {
    let names: Vec<String> = loader
        .with_fluent_message(id, |m| {
            let mut out = BTreeSet::new();
            if let Some(p) = m.value() {
                vars(p, &mut out);
            }
            out.into_iter().collect()
        })
        .unwrap_or_default();
    let markers: Vec<String> = names
        .iter()
        .map(|n| format!("{{{}}}", n.replace('-', ".")))
        .collect();
    let args = names
        .iter()
        .map(String::as_str)
        .zip(markers.iter().map(|m| m.as_str().into()))
        .collect();
    loader.get_args_concrete(id, args)
}

/// Mensagem → valores (`{ $nome }`) que ela usa.
pub fn read_messages(
    path: &std::path::Path,
) -> Result<std::collections::BTreeMap<String, std::collections::BTreeSet<String>>, String> {
    use fluent_syntax::ast::Entry;
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let resource = fluent_syntax::parser::parse(text.as_str()).map_err(|(_, errors)| {
        format!("{}: {} erro(s) de sintaxe", path.display(), errors.len())
    })?;
    let mut out = std::collections::BTreeMap::new();
    for entry in &resource.body {
        if let Entry::Message(m) = entry {
            let mut used = std::collections::BTreeSet::new();
            if let Some(value) = &m.value {
                vars(value, &mut used);
            }
            out.insert(m.id.name.to_string(), used);
        }
    }
    Ok(out)
}

/// Confere as traduções de uma pasta `i18n/`.
///
/// Cada idioma precisa ter as mesmas mensagens e os mesmos valores (`{ $name }`) que o inglês.
/// Chave ausente é erro, não fallback. Devolve a lista de problemas (vazia = tudo certo).
pub fn check_translations(i18n_dir: &std::path::Path) -> Vec<String> {
    let mut problems = Vec::new();
    let Ok(entries) = std::fs::read_dir(i18n_dir.join("en")) else {
        return vec![format!("{}: sem a pasta en", i18n_dir.display())];
    };
    for file in entries.flatten().map(|e| e.file_name()) {
        let base = match read_messages(&i18n_dir.join("en").join(&file)) {
            Ok(m) => m,
            Err(e) => {
                problems.push(e);
                continue;
            }
        };
        for lang in Language::ALL.iter().filter(|l| **l != Language::English) {
            let path = i18n_dir.join(lang.tag()).join(&file);
            let other = match read_messages(&path) {
                Ok(m) => m,
                Err(e) => {
                    problems.push(e);
                    continue;
                }
            };
            for (id, vars) in &base {
                match other.get(id) {
                    None => problems.push(format!("{}: falta \"{id}\"", path.display())),
                    Some(v) if v != vars => problems.push(format!(
                        "{}: \"{id}\" usa {v:?}, o inglês usa {vars:?}",
                        path.display()
                    )),
                    _ => {}
                }
            }
            for id in other.keys().filter(|id| !base.contains_key(*id)) {
                problems.push(format!("{}: \"{id}\" não existe em inglês", path.display()));
            }
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_language() {
        assert_eq!(resolve("auto", Some("pt_BR.UTF-8")), Language::PortugueseBr);
        assert_eq!(resolve("auto", Some("pt-PT")), Language::PortugueseBr);
        assert_eq!(resolve("", Some("es-419")), Language::Spanish);
        assert_eq!(resolve("auto", Some("C")), Language::English);
        assert_eq!(resolve("auto", Some("de_DE")), Language::German);
        assert_eq!(resolve("auto", Some("fr_FR.UTF-8")), Language::French);
        assert_eq!(resolve("auto", Some("it_IT")), Language::Italian);
        assert_eq!(resolve("auto", Some("ja_JP")), Language::Japanese);
        assert_eq!(resolve("zh-CN", Some("en")), Language::ChineseSimplified);
        assert_eq!(resolve("zh-Hans", Some("en")), Language::ChineseSimplified);
        assert_eq!(
            resolve("auto", Some("zh_TW")),
            Language::English,
            "tradicional não vira simplificado"
        );
        assert_eq!(resolve("auto", Some("zh-Hant")), Language::English);
        assert_eq!(resolve("auto", None), Language::English);
        assert_eq!(
            resolve("es", Some("pt_BR")),
            Language::Spanish,
            "o config vence o sistema"
        );
        assert_eq!(resolve("EN", None), Language::English);
        assert_eq!(
            resolve("klingon", Some("pt_BR")),
            Language::PortugueseBr,
            "idioma desconhecido: o do sistema"
        );
    }

    #[test]
    fn japanese_catalog_is_used() {
        let loader = load_in("tea-core", &crate::Localizations, Language::Japanese);
        let args = std::collections::HashMap::from([("count", "2")]);
        let got = loader.get_args("count-references", args);
        assert!(got.contains('件'), "{got}");
        assert!(!got.to_lowercase().contains("reference"), "{got}");
        assert!(!got.contains("referên"), "{got}");
    }
}
