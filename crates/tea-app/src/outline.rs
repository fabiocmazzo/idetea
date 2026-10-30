//! Estrutura do arquivo para o painel: converte os símbolos do servidor de linguagem e descobre a
//! visibilidade de cada um, que o LSP não informa. A regra olha o texto antes do nome, na linha da
//! declaração (`pub fn`, `private void`...), e o que a linguagem diz quando não há palavra nenhuma.

use tea_core::outline::{Kind, Symbol, Visibility};
use tea_lsp::DocumentSymbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lang {
    Rust,
    Java,
    CSharp,
    Kotlin,
    Swift,
    Php,
    Script,
    Go,
    Python,
    Cpp,
    Other,
}

fn lang(ext: &str) -> Lang {
    match ext {
        "rs" => Lang::Rust,
        "java" => Lang::Java,
        "cs" => Lang::CSharp,
        "kt" | "kts" => Lang::Kotlin,
        "swift" => Lang::Swift,
        "php" => Lang::Php,
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts" => Lang::Script,
        "go" => Lang::Go,
        "py" | "pyi" => Lang::Python,
        "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" => Lang::Cpp,
        _ => Lang::Other,
    }
}

struct Ctx<'a> {
    lines: Vec<&'a str>,
    lang: Lang,
}

/// O pai de um símbolo, com a visibilidade que já foi decidida para ele.
struct Parent<'a> {
    symbol: &'a DocumentSymbol,
    kind: Kind,
    visibility: &'a Visibility,
}

pub fn build(symbols: &[DocumentSymbol], text: &str, ext: &str) -> Vec<Symbol> {
    let ctx = Ctx { lines: text.lines().collect(), lang: lang(ext) };
    symbols.iter().map(|s| convert(s, None, &ctx)).collect()
}

fn convert(s: &DocumentSymbol, parent: Option<&Parent>, ctx: &Ctx) -> Symbol {
    let kind = Kind::from_lsp(s.kind);
    let visibility = visibility(s, kind, parent, ctx);
    let me = Parent { symbol: s, kind, visibility: &visibility };
    let children = s.children.iter().map(|c| convert(c, Some(&me), ctx)).collect();
    Symbol {
        name: s.name.clone(),
        kind,
        detail: s.detail.clone(),
        line: s.selection.start.line as usize + 1,
        column: s.selection.start.character as usize + 1,
        start_line: s.range.start.line as usize + 1,
        end_line: s.range.end.line as usize + 1,
        children,
        visibility,
    }
}

/// Texto antes do nome, na linha do nome. Só a declaração dele: com vários na mesma linha
/// (`struct A { pub x: i32, y: i32 }`), o que vem antes do último `{`, `,` ou `;` é de outro.
fn prefix<'a>(s: &DocumentSymbol, ctx: &Ctx<'a>) -> &'a str {
    let line = ctx.lines.get(s.selection.start.line as usize).copied().unwrap_or_default();
    let end = line.char_indices().nth(s.selection.start.character as usize).map_or(line.len(), |(i, _)| i);
    let before = &line[..end];
    before.rfind(['{', ',', ';']).map_or(before, |i| &before[i + 1..])
}

fn words(prefix: &str) -> impl Iterator<Item = &str> {
    prefix.split(|c: char| !(c.is_alphanumeric() || c == '_')).filter(|w| !w.is_empty())
}

fn restricted(label: &str) -> Visibility {
    Visibility::Restricted(label.to_string())
}

/// Palavras de visibilidade das linguagens "com modificador" (Java, C#, Kotlin, Swift, PHP, TS).
fn keyword(prefix: &str) -> Option<Visibility> {
    let mut found = None;
    for w in words(prefix) {
        found = Some(match w {
            "private" | "fileprivate" => Visibility::Private,
            "protected" => restricted("protected"),
            "internal" => restricted("internal"),
            "public" | "open" | "export" => Visibility::Public,
            _ => continue,
        });
    }
    found
}

fn visibility(s: &DocumentSymbol, kind: Kind, parent: Option<&Parent>, ctx: &Ctx) -> Visibility {
    // Coisas declaradas dentro de funções são locais: visibilidade não se aplica.
    if parent.is_some_and(|p| matches!(p.kind, Kind::Function | Kind::Method | Kind::Constructor)) {
        return Visibility::Unknown;
    }
    let in_interface = parent.is_some_and(|p| p.kind == Kind::Interface);
    let prefix = prefix(s, ctx);
    match ctx.lang {
        Lang::Rust => {
            if kind == Kind::Block {
                return Visibility::Unknown;
            }
            match parent {
                // Variantes de enum e itens de trait têm a visibilidade do dono.
                Some(p) if kind == Kind::EnumMember || p.kind == Kind::Interface => return p.visibility.clone(),
                // `impl Trait for Tipo`: os métodos valem onde o trait vale.
                Some(p) if p.kind == Kind::Block && p.symbol.name.contains(" for ") => return Visibility::Public,
                _ => {}
            }
            if let Some(i) = prefix.find("pub(") {
                let end = prefix[i..].find(')').map_or(prefix.len(), |e| i + e + 1);
                return restricted(&prefix[i..end]);
            }
            if words(prefix).any(|w| w == "pub") { Visibility::Public } else { Visibility::Private }
        }
        Lang::Java => keyword(prefix).unwrap_or(if in_interface || kind == Kind::EnumMember {
            Visibility::Public
        } else {
            restricted("pacote")
        }),
        Lang::CSharp => keyword(prefix).unwrap_or(if in_interface || kind == Kind::EnumMember {
            Visibility::Public
        } else if parent.is_none() || parent.is_some_and(|p| p.kind == Kind::Module) {
            restricted("internal")
        } else {
            Visibility::Private
        }),
        Lang::Kotlin | Lang::Php => keyword(prefix).unwrap_or(Visibility::Public),
        Lang::Swift => keyword(prefix).unwrap_or_else(|| restricted("internal")),
        Lang::Script => {
            if s.name.starts_with('#') {
                return Visibility::Private;
            }
            if let Some(v) = keyword(prefix) {
                return v;
            }
            // Membros de classe são públicos; no topo do arquivo, só o que tem `export` sai dele.
            match parent {
                Some(p) if matches!(p.kind, Kind::Class | Kind::Interface | Kind::Enum) => Visibility::Public,
                Some(_) => Visibility::Unknown,
                None => Visibility::Private,
            }
        }
        Lang::Go => match s.name.chars().next() {
            Some(c) if c.is_uppercase() => Visibility::Public,
            Some(_) => restricted("pacote"),
            None => Visibility::Unknown,
        },
        Lang::Python => {
            let n = &s.name;
            if n.starts_with("__") && n.ends_with("__") {
                Visibility::Public
            } else if n.starts_with('_') {
                Visibility::Private
            } else {
                Visibility::Public
            }
        }
        Lang::Cpp => cpp(s, kind, parent, prefix, ctx),
        Lang::Other => Visibility::Unknown,
    }
}

/// C/C++: dentro de class/struct vale o último `public:`/`private:`/`protected:` antes do membro
/// (no nível da própria classe); sem rótulo, class é privada e struct é pública. Fora delas,
/// `static` deixa a função só neste arquivo.
fn cpp(s: &DocumentSymbol, kind: Kind, parent: Option<&Parent>, prefix: &str, ctx: &Ctx) -> Visibility {
    // Método definido fora da classe (`void Conta::depositar()`): a visibilidade está na declaração.
    if s.name.contains("::") {
        return Visibility::Unknown;
    }
    match parent {
        Some(p) if matches!(p.kind, Kind::Class | Kind::Struct) => {
            let mut current = if p.kind == Kind::Class { Visibility::Private } else { Visibility::Public };
            let mut depth = 0i32;
            let from = p.symbol.range.start.line as usize;
            let to = s.selection.start.line as usize;
            for line in ctx.lines.iter().take(to).skip(from) {
                let t = line.trim();
                if depth == 1 {
                    for (label, v) in [
                        ("public", Visibility::Public),
                        ("private", Visibility::Private),
                        ("protected", restricted("protected")),
                    ] {
                        if let Some(rest) = t.strip_prefix(label)
                            && rest.trim_start().starts_with(':')
                            && !rest.trim_start().starts_with("::")
                        {
                            current = v;
                        }
                    }
                }
                depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
            }
            current
        }
        Some(_) if kind == Kind::EnumMember => parent.map_or(Visibility::Unknown, |p| p.visibility.clone()),
        _ if matches!(kind, Kind::Function | Kind::Variable | Kind::Constant) => {
            if words(prefix).any(|w| w == "static") { restricted("só neste arquivo") } else { Visibility::Public }
        }
        _ => Visibility::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tea_lsp::{Position, Range};

    /// Símbolo cujo nome está em `line`, procurando `name` nela; o trecho vai até `end`.
    fn sym(text: &str, name: &str, kind: u32, line: u32, end: u32, children: Vec<DocumentSymbol>) -> DocumentSymbol {
        let col = text.lines().nth(line as usize).and_then(|l| l.find(name)).expect(name);
        let col = text.lines().nth(line as usize).unwrap()[..col].chars().count() as u32;
        let at = Range { start: Position::new(line, col), end: Position::new(line, col + name.chars().count() as u32) };
        DocumentSymbol {
            name: name.into(),
            kind,
            detail: None,
            range: Range { start: Position::new(line, 0), end: Position::new(end, 0) },
            selection: at,
            children,
        }
    }

    fn vis(symbols: &[Symbol]) -> Vec<(String, Visibility)> {
        let mut out = Vec::new();
        for s in symbols {
            out.push((s.name.clone(), s.visibility.clone()));
            out.extend(vis(&s.children));
        }
        out
    }

    fn r(label: &str) -> Visibility {
        restricted(label)
    }

    use Visibility::{Private as Pv, Public as Pb, Unknown as Un};

    #[test]
    fn rust() {
        let t = "pub struct Conta { pub saldo: i64, id: u32 }\n\
                 impl Conta {\n    pub fn depositar(&mut self) {}\n    fn validar(&self) {}\n    pub(crate) fn interno(&self) {}\n}\n\
                 impl Default for Conta {\n    fn default() -> Self { todo!() }\n}\n\
                 enum Cor { Azul }\n\
                 pub trait Forma {\n    fn area(&self) -> f64;\n}\n";
        let s = vec![
            sym(t, "Conta", 23, 0, 0, vec![sym(t, "saldo", 8, 0, 0, vec![]), sym(t, "id", 8, 0, 0, vec![])]),
            sym(
                t,
                "impl Conta",
                19,
                1,
                5,
                vec![sym(t, "depositar", 6, 2, 2, vec![]), sym(t, "validar", 6, 3, 3, vec![]), sym(t, "interno", 6, 4, 4, vec![])],
            )
            .renamed("impl Conta"),
            sym(t, "Default", 19, 6, 8, vec![sym(t, "default", 6, 7, 7, vec![])]).renamed("impl Default for Conta"),
            sym(t, "Cor", 10, 9, 9, vec![sym(t, "Azul", 22, 9, 9, vec![])]),
            sym(t, "Forma", 11, 10, 12, vec![sym(t, "area", 6, 11, 11, vec![])]),
        ];
        let got = vis(&build(&s, t, "rs"));
        let want = [
            ("Conta", Pb),
            ("saldo", Pb),
            ("id", Pv),
            ("impl Conta", Un),
            ("depositar", Pb),
            ("validar", Pv),
            ("interno", r("pub(crate)")),
            ("impl Default for Conta", Un),
            ("default", Pb),
            ("Cor", Pv),
            ("Azul", Pv),
            ("Forma", Pb),
            ("area", Pb),
        ];
        assert_eq!(got, want.map(|(n, v)| (n.to_string(), v)));
    }

    #[test]
    fn cpp_access_labels() {
        let t = "class Conta {\n    int saldo;\npublic:\n    void depositar();\n    struct Dentro {\n    private:\n        int x;\n    };\nprotected:\n    void validar();\n};\n\
                 struct Ponto { int x; };\nstatic int ajuda() { return 0; }\nint main() { return 0; }\n";
        let s = vec![
            sym(
                t,
                "Conta",
                5,
                0,
                10,
                vec![
                    sym(t, "saldo", 8, 1, 1, vec![]),
                    sym(t, "depositar", 6, 3, 3, vec![]),
                    sym(t, "Dentro", 23, 4, 7, vec![sym(t, "x", 8, 6, 6, vec![])]),
                    sym(t, "validar", 6, 9, 9, vec![]),
                ],
            ),
            sym(t, "Ponto", 23, 11, 11, vec![sym(t, "x", 8, 11, 11, vec![])]),
            sym(t, "ajuda", 12, 12, 12, vec![]),
            sym(t, "main", 12, 13, 13, vec![]),
        ];
        let got = vis(&build(&s, t, "cpp"));
        let want = [
            ("Conta", Un),
            ("saldo", Pv),
            ("depositar", Pb),
            ("Dentro", Pb),
            ("x", Pv),
            ("validar", r("protected")),
            ("Ponto", Un),
            ("x", Pb),
            ("ajuda", r("só neste arquivo")),
            ("main", Pb),
        ];
        assert_eq!(got, want.map(|(n, v)| (n.to_string(), v)));
    }

    #[test]
    fn other_languages() {
        let py = "class Conta:\n    def depositar(self): pass\n    def _validar(self): pass\n    def __init__(self): pass\n";
        let s = vec![sym(
            py,
            "Conta",
            5,
            0,
            3,
            vec![sym(py, "depositar", 6, 1, 1, vec![]), sym(py, "_validar", 6, 2, 2, vec![]), sym(py, "__init__", 6, 3, 3, vec![])],
        )];
        let got: Vec<_> = vis(&build(&s, py, "py")).into_iter().map(|(_, v)| v).collect();
        assert_eq!(got, [Pb, Pb, Pv, Pb]);

        let go = "func Soma() {}\nfunc ajuda() {}\n";
        let s = vec![sym(go, "Soma", 12, 0, 0, vec![]), sym(go, "ajuda", 12, 1, 1, vec![])];
        assert_eq!(vis(&build(&s, go, "go")).into_iter().map(|(_, v)| v).collect::<Vec<_>>(), [Pb, r("pacote")]);

        let java = "public class Conta {\n    private int saldo;\n    void ajuda() {}\n    protected void validar() {}\n}\n";
        let s = vec![sym(
            java,
            "Conta",
            5,
            0,
            4,
            vec![sym(java, "saldo", 8, 1, 1, vec![]), sym(java, "ajuda", 6, 2, 2, vec![]), sym(java, "validar", 6, 3, 3, vec![])],
        )];
        let got: Vec<_> = vis(&build(&s, java, "java")).into_iter().map(|(_, v)| v).collect();
        assert_eq!(got, [Pb, Pv, r("pacote"), r("protected")]);

        let ts = "export class Conta {\n    private saldo = 0;\n    depositar() {}\n    #segredo = 1;\n}\nfunction ajuda() {}\n";
        let s = vec![
            sym(
                ts,
                "Conta",
                5,
                0,
                4,
                vec![sym(ts, "saldo", 7, 1, 1, vec![]), sym(ts, "depositar", 6, 2, 2, vec![]), sym(ts, "#segredo", 7, 3, 3, vec![])],
            ),
            sym(ts, "ajuda", 12, 5, 5, vec![]),
        ];
        let got: Vec<_> = vis(&build(&s, ts, "ts")).into_iter().map(|(_, v)| v).collect();
        assert_eq!(got, [Pb, Pv, Pb, Pv, Pv]);
    }

    #[test]
    fn locals_and_unknown_languages() {
        let t = "fn main() {\n    let x = 1;\n}\n";
        let s = vec![sym(t, "main", 12, 0, 2, vec![sym(t, "x", 13, 1, 1, vec![])])];
        let got = build(&s, t, "rs");
        assert_eq!(got[0].children[0].visibility, Un, "variável local");
        assert_eq!((got[0].line, got[0].column, got[0].start_line, got[0].end_line), (1, 4, 1, 3));
        assert_eq!(build(&s, t, "lua")[0].visibility, Un);
    }

    trait Renamed {
        fn renamed(self, name: &str) -> Self;
    }

    impl Renamed for DocumentSymbol {
        fn renamed(mut self, name: &str) -> Self {
            self.name = name.into();
            self
        }
    }
}
