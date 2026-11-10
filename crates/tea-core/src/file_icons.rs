//! Ícone de cada arquivo pela linguagem ou pelo tipo (RFC 0008).
//!
//! O conjunto embutido é o Seti, o tema padrão do VS Code: uma fonte de ícones (`assets/seti/seti.ttf`) e a tabela
//! no formato de *file icon theme* do VS Code (`vs-seti-icon-theme.json`), com cores para tema escuro e claro.
//! Licenças em `assets/seti/LICENSE.txt`. O app carrega a fonte com [`crate::fonts::load_embedded`].
//!
//! Os language packs (RFC 0014) trazem ícones próprios para linguagens que o Seti não tem, numa fonte à parte
//! (`assets/languages/icons.ttf`, ver [`crate::languages::Icon`]); a extensão de um pack vem antes do Seti.
//!
//! Ordem de busca no Seti, como no VS Code: nome exato, extensões compostas da mais longa para a mais curta (`d.ts` antes
//! de `ts`), linguagem (pela extensão ou pelo nome), o padrão do tema.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::widget::{Text, text};
use iced::{Color, Font};

/// A fonte do Seti (TrueType convertida do `seti.woff` do VS Code, sem outras mudanças).
pub const FONT_BYTES: &[u8] = include_bytes!("../assets/seti/seti.ttf");
pub const FONT: Font = Font::with_name("seti");
/// A fonte dos ícones dos language packs (`tools/language-icons.py`).
pub const LANGUAGE_FONT: Font = Font::with_name("idetea-languages");
const THEME_JSON: &str = include_str!("../assets/seti/vs-seti-icon-theme.json");

/// Os glifos do Seti são pequenos no quadro: o VS Code desenha a 150%. Aqui, um pouco menos, para caber na linha.
const SCALE: f32 = 1.35;

static ENABLED: AtomicBool = AtomicBool::new(true);

/// `[ui] file_icons`: `"seti"` liga, `"none"` volta ao ícone genérico.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// O que há de errado em `[ui] file_icons`, para o aviso do config.
pub fn problem(setting: &str) -> Option<String> {
    match setting.trim() {
        "seti" | "none" => None,
        other => Some(fl!("config-file-icons", value = other)),
    }
}

/// O ícone de um arquivo: o caractere, a fonte dele (Seti ou a dos packs) e a cor no tema escuro e no claro.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileIcon {
    pub glyph: char,
    pub font: Font,
    pub dark: Option<Color>,
    pub light: Option<Color>,
}

/// O ícone para o nome do arquivo (só o nome, sem pastas).
pub fn lookup(name: &str) -> FileIcon {
    let lower = name.to_lowercase();
    if let Some(icon) = lower.rsplit_once('.').and_then(|(_, ext)| pack_icons().get(ext)) {
        return *icon;
    }
    let theme = theme();
    let dark = theme.dark.find(name).unwrap_or(&theme.dark.file);
    let light = theme.light.find(name).unwrap_or(dark);
    let d = theme.defs.get(dark).copied().unwrap_or(theme.fallback);
    let l = theme.defs.get(light).copied().unwrap_or(d);
    FileIcon { glyph: d.0, font: FONT, dark: d.1, light: l.1 }
}

/// Extensão → ícone, dos language packs que trazem `[icon]`.
fn pack_icons() -> &'static HashMap<String, FileIcon> {
    static ICONS: OnceLock<HashMap<String, FileIcon>> = OnceLock::new();
    ICONS.get_or_init(|| {
        let mut map = HashMap::new();
        for pack in crate::languages::builtins().0 {
            let Some(icon) = pack.manifest.icon else { continue };
            let Some(glyph) = icon.glyph.chars().next() else { continue };
            let dark = hex(&icon.color);
            let light = icon.light.as_deref().and_then(hex).or(dark);
            for extension in pack.manifest.files.extensions.keys() {
                map.insert(extension.to_lowercase(), FileIcon { glyph, font: LANGUAGE_FONT, dark, light });
            }
        }
        map
    })
}

/// O ícone já como texto, na cor do tema (escuro ou claro). Desligado (`file_icons = "none"`), o genérico da
/// Codicons. `colored: false` deixa a cor de quem está em volta (a aba ativa, com fundo da cor de destaque).
pub fn icon<'a>(name: &str, size: f32, colored: bool) -> Text<'a> {
    if !enabled() {
        return crate::icons::icon(crate::icons::Icon::File).size(size);
    }
    let i = lookup(name);
    let t = text(i.glyph.to_string()).font(i.font).size(size * SCALE).line_height(1.0);
    if !colored {
        return t;
    }
    t.style(move |theme: &iced::Theme| text::Style {
        color: if theme.extended_palette().is_dark { i.dark } else { i.light },
    })
}

struct Theme {
    /// Nome da definição → caractere e cor.
    defs: HashMap<String, (char, Option<Color>)>,
    fallback: (char, Option<Color>),
    dark: Mapping,
    light: Mapping,
}

#[derive(Default)]
struct Mapping {
    file: String,
    names: HashMap<String, String>,
    extensions: HashMap<String, String>,
    languages: HashMap<String, String>,
}

impl Mapping {
    fn find(&self, name: &str) -> Option<&String> {
        let lower = name.to_lowercase();
        if let Some(d) = self.names.get(&lower) {
            return Some(d);
        }
        // `a.d.ts` → `d.ts`, depois `ts`. Um nome que começa com ponto (`.env`) também vale como extensão.
        for (i, _) in lower.match_indices('.') {
            if let Some(d) = self.extensions.get(&lower[i + 1..]) {
                return Some(d);
            }
        }
        language(&lower).and_then(|l| self.languages.get(l))
    }
}

fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| parse(THEME_JSON))
}

fn parse(json: &str) -> Theme {
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    let mut defs = HashMap::new();
    if let Some(map) = v["iconDefinitions"].as_object() {
        for (k, d) in map {
            let glyph = d["fontCharacter"]
                .as_str()
                .and_then(|s| u32::from_str_radix(s.trim_start_matches('\\'), 16).ok())
                .and_then(char::from_u32);
            if let Some(glyph) = glyph {
                defs.insert(k.clone(), (glyph, d["fontColor"].as_str().and_then(hex)));
            }
        }
    }
    let mapping = |m: &serde_json::Value| Mapping {
        file: m["file"].as_str().unwrap_or_default().to_string(),
        names: table(&m["fileNames"]),
        extensions: table(&m["fileExtensions"]),
        languages: table(&m["languageIds"]),
    };
    let mut dark = mapping(&v);
    let fallback = defs.get(&dark.file).copied().unwrap_or(('\u{E023}', None));
    let mut light = mapping(&v["light"]);
    if light.file.is_empty() {
        light.file.clone_from(&dark.file);
    }
    for m in [&mut dark, &mut light] {
        for (alias, of) in EXTENSION_ALIASES {
            if let Some(def) = m.extensions.get(*of).cloned() {
                m.extensions.entry((*alias).to_string()).or_insert(def);
            }
        }
    }
    Theme { defs, fallback, dark, light }
}

/// Extensões que o Seti não conhece e usam o ícone de outra: os arquivos do Ziex (Zig com marcação) usam o do Zig.
const EXTENSION_ALIASES: &[(&str, &str)] = &[("zx", "zig"), ("mdzx", "zig")];

fn table(v: &serde_json::Value) -> HashMap<String, String> {
    v.as_object()
        .map(|m| m.iter().filter_map(|(k, d)| Some((k.to_lowercase(), d.as_str()?.to_string()))).collect())
        .unwrap_or_default()
}

fn hex(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#')?;
    let n = u32::from_str_radix(s, 16).ok()?;
    match s.len() {
        6 => Some(Color::from_rgb8((n >> 16) as u8, (n >> 8) as u8, n as u8)),
        _ => None,
    }
}

/// A linguagem do VS Code (`languageId`) pelo nome do arquivo, para o que o tema só mapeia por linguagem.
/// `name` já em minúsculas.
fn language(name: &str) -> Option<&'static str> {
    const NAMES: &[(&str, &str)] = &[
        ("dockerfile", "dockerfile"),
        ("containerfile", "dockerfile"),
        ("makefile", "makefile"),
        ("gnumakefile", "makefile"),
        ("docker-compose.yml", "dockercompose"),
        ("docker-compose.yaml", "dockercompose"),
        ("compose.yml", "dockercompose"),
        ("compose.yaml", "dockercompose"),
        (".env", "dotenv"),
        (".gitignore", "ignore"),
        (".dockerignore", "ignore"),
        (".ignore", "ignore"),
        ("commit_editmsg", "git-commit"),
    ];
    const EXTENSIONS: &[(&str, &[&str])] = &[
        ("rust", &["rs"]),
        ("python", &["py", "pyw", "pyi"]),
        ("javascript", &["js", "mjs", "cjs"]),
        ("javascriptreact", &["jsx"]),
        ("typescript", &["ts", "mts", "cts"]),
        ("typescriptreact", &["tsx"]),
        ("go", &["go"]),
        ("c", &["c", "h"]),
        ("cpp", &["cpp", "cc", "cxx", "c++", "hpp", "hh", "hxx", "h++", "ino"]),
        ("cuda-cpp", &["cu", "cuh"]),
        ("objective-c", &["m"]),
        ("objective-cpp", &["mm"]),
        ("csharp", &["cs", "csx"]),
        ("fsharp", &["fs", "fsi", "fsx"]),
        ("java", &["java"]),
        ("kotlin", &["kt", "kts"]),
        ("groovy", &["groovy"]),
        ("gradle", &["gradle"]),
        ("swift", &["swift"]),
        ("dart", &["dart"]),
        ("lua", &["lua"]),
        ("ruby", &["rb", "rake", "gemspec"]),
        ("erb", &["erb"]),
        ("php", &["php", "phtml"]),
        ("perl", &["pl", "pm"]),
        ("r", &["r"]),
        ("julia", &["jl"]),
        ("haskell", &["hs", "lhs"]),
        ("elixir", &["ex", "exs"]),
        ("elm", &["elm"]),
        ("clojure", &["clj", "cljs", "cljc", "edn"]),
        ("ocaml", &["ml", "mli"]),
        ("rescript", &["res", "resi"]),
        ("coffeescript", &["coffee"]),
        ("vala", &["vala"]),
        ("haxe", &["hx"]),
        ("godot", &["gd"]),
        ("html", &["html", "htm", "xhtml"]),
        ("django-html", &["djhtml"]),
        ("jinja", &["j2", "jinja", "jinja2"]),
        ("handlebars", &["hbs", "handlebars"]),
        ("mustache", &["mustache"]),
        ("nunjucks", &["njk"]),
        ("jade", &["jade", "pug"]),
        ("haml", &["haml"]),
        ("razor", &["cshtml", "razor"]),
        ("vue", &["vue"]),
        ("css", &["css"]),
        ("scss", &["scss"]),
        ("sass", &["sass"]),
        ("less", &["less"]),
        ("stylus", &["styl"]),
        ("postcss", &["pcss", "postcss"]),
        ("json", &["json"]),
        ("jsonc", &["jsonc"]),
        ("jsonl", &["jsonl"]),
        ("yaml", &["yml", "yaml"]),
        ("xml", &["xml", "xsd", "xsl", "plist"]),
        ("markdown", &["md", "markdown", "mdx"]),
        ("latex", &["tex"]),
        ("tex", &["sty", "cls"]),
        ("shellscript", &["sh", "bash", "zsh", "fish", "ksh"]),
        ("powershell", &["ps1", "psm1", "psd1"]),
        ("bat", &["bat", "cmd"]),
        ("sql", &["sql"]),
        ("terraform", &["tf", "tfvars"]),
        ("bicep", &["bicep"]),
        ("properties", &["properties", "ini", "cfg", "conf"]),
        ("makefile", &["mk"]),
    ];
    if let Some((_, l)) = NAMES.iter().find(|(n, _)| *n == name) {
        return Some(l);
    }
    let ext = name.rsplit_once('.')?.1;
    EXTENSIONS.iter().find(|(_, exts)| exts.contains(&ext)).map(|(l, _)| *l)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(name: &str) -> &'static str {
        let t = theme();
        let d = t.dark.find(name).unwrap_or(&t.dark.file);
        // Devolve a definição para comparar pelo nome (mais legível que o caractere).
        Box::leak(d.clone().into_boxed_str())
    }

    #[test]
    fn embedded_theme_and_font_load() {
        let t = theme();
        assert!(t.defs.len() > 300, "{}", t.defs.len());
        assert_eq!(t.dark.file, "_default");
        assert_eq!(&FONT_BYTES[..4], &[0, 1, 0, 0], "TrueType");
    }

    #[test]
    fn lookup_order_follows_vs_code() {
        // Nome exato vence a extensão.
        assert_eq!(def("README.md"), "_info");
        assert_eq!(def("notes.md"), "_markdown");
        // Extensão composta antes da simples.
        assert_eq!(def("app.spec.js"), "_javascript_1");
        assert_eq!(def("app.js"), "_javascript");
        // Só pela linguagem (o tema não mapeia `.py` por extensão).
        assert_eq!(def("main.py"), "_python");
        assert_eq!(def("Dockerfile"), def("Containerfile"));
        // Desconhecido: o padrão do tema.
        assert_eq!(def("arquivo.qualquercoisa"), "_default");
        assert_eq!(def("SEM_EXTENSAO"), "_default");
        // Ziex: o ícone do Zig.
        assert_eq!(def("page.zx"), def("main.zig"));
        assert_eq!(def("post.mdzx"), "_zig");
    }

    #[test]
    fn colors_differ_for_light_themes() {
        let rs = lookup("main.rs");
        assert!(rs.dark.is_some() && rs.light.is_some());
        assert_ne!(rs.dark, rs.light, "o Seti escurece as cores no tema claro");
        assert_eq!(lookup("main.rs").glyph, '\u{E082}');
    }

    #[test]
    fn language_packs_bring_their_own_icons() {
        let mbt = lookup("main.mbt");
        assert_eq!((mbt.glyph, mbt.font), ('\u{E001}', LANGUAGE_FONT));
        assert_eq!(lookup("pkg.generated.MBTI").glyph, '\u{E001}');
        assert_eq!(lookup("sum.ha").glyph, '\u{E000}');
        assert!(mbt.dark.is_some() && mbt.light.is_some() && mbt.dark != mbt.light);
        assert_eq!(lookup("main.rs").font, FONT);
    }

    #[test]
    fn setting_is_checked() {
        assert_eq!(problem("seti"), None);
        assert_eq!(problem("none"), None);
        assert!(problem("material").is_some());
    }
}
