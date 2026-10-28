//! Atalhos de teclado: texto do config (`"ctrl+shift+p"`) ↔ tecla do iced.

use std::collections::HashMap;
use std::fmt;

use iced::keyboard::{Key, Modifiers, key::Named};

/// Uma combinação de teclas, ex.: `ctrl+shift+p`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chord {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// Tecla Super/Cmd. No config: `cmd`, `super` ou `meta`.
    pub logo: bool,
    /// Nome da tecla em minúsculas: `"p"`, `"enter"`, `"f3"`, `"pageup"`...
    pub key: String,
}

impl Chord {
    /// Lê um atalho no formato do config. Ex.: `"ctrl+shift+p"`, `"f3"`, `"ctrl+pageup"`.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut chord = Chord { ctrl: false, alt: false, shift: false, logo: false, key: String::new() };
        let parts: Vec<&str> = text.split('+').map(str::trim).collect();
        // `ctrl++` = ctrl e a tecla "+".
        let (mods, key) = match parts.as_slice() {
            [.., "", ""] => (&parts[..parts.len() - 2], "+"),
            [mods @ .., key] => (mods, *key),
            [] => return Err("atalho vazio".into()),
        };
        for m in mods {
            match m.to_lowercase().as_str() {
                "ctrl" | "control" => chord.ctrl = true,
                "alt" | "option" => chord.alt = true,
                "shift" => chord.shift = true,
                "cmd" | "super" | "meta" | "logo" => chord.logo = true,
                other => return Err(format!("modificador desconhecido: {other:?}")),
            }
        }
        if key.is_empty() {
            return Err(format!("atalho sem tecla: {text:?}"));
        }
        chord.key = normalize_key_name(key);
        Ok(chord)
    }

    /// Converte uma tecla do iced. Retorna `None` para teclas que não formam atalho (só modificador).
    pub fn from_event(key: &Key, modifiers: Modifiers) -> Option<Self> {
        let name = match key {
            Key::Character(c) => c.to_lowercase(),
            Key::Named(named) => match named {
                Named::Shift | Named::Control | Named::Alt | Named::Super | Named::Meta => return None,
                other => normalize_key_name(&format!("{other:?}")),
            },
            Key::Unidentified => return None,
        };
        Some(Chord {
            ctrl: modifiers.control(),
            alt: modifiers.alt(),
            shift: modifiers.shift(),
            logo: modifiers.logo(),
            key: name,
        })
    }
}

fn normalize_key_name(name: &str) -> String {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "esc" => "escape".into(),
        "return" => "enter".into(),
        "up" => "arrowup".into(),
        "down" => "arrowdown".into(),
        "left" => "arrowleft".into(),
        "right" => "arrowright".into(),
        "pgup" => "pageup".into(),
        "pgdown" | "pgdn" => "pagedown".into(),
        "del" => "delete".into(),
        _ => lower,
    }
}

impl Chord {
    /// Formato do config: `"ctrl+shift+p"` (o inverso de [`Chord::parse`]).
    pub fn to_config(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.ctrl {
            parts.push("ctrl");
        }
        if self.alt {
            parts.push("alt");
        }
        if self.shift {
            parts.push("shift");
        }
        if self.logo {
            parts.push("super");
        }
        parts.push(&self.key);
        parts.join("+")
    }

    /// Só modificadores e nenhuma tecla "de verdade" não formam atalho útil sozinhos;
    /// letras sem Ctrl/Alt/Super atrapalhariam a digitação.
    pub fn is_bindable(&self) -> bool {
        let named = self.key.chars().count() > 1; // f5, pageup, escape...
        named || self.ctrl || self.alt || self.logo
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.logo {
            parts.push("Cmd".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        let key = match self.key.as_str() {
            "pageup" => "PageUp".to_string(),
            "pagedown" => "PageDown".to_string(),
            k if k.chars().count() == 1 => k.to_uppercase(),
            k => {
                let mut c = k.chars();
                c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
            }
        };
        parts.push(key);
        f.write_str(&parts.join("+"))
    }
}

/// Mapa atalho → id de comando.
#[derive(Debug, Default, Clone)]
pub struct Keymap {
    bindings: HashMap<Chord, String>,
}

impl Keymap {
    /// Monta o mapa a partir de pares `(atalho, comando)`. Atalhos inválidos viram erros, mas não impedem o resto.
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> (Self, Vec<String>) {
        let mut map = Keymap::default();
        let mut errors = Vec::new();
        for (chord, command) in pairs {
            match Chord::parse(chord) {
                // Comando vazio desliga o atalho (útil no config do projeto).
                Ok(c) if command.is_empty() => {
                    map.bindings.remove(&c);
                }
                Ok(c) => {
                    map.bindings.insert(c, command.to_string());
                }
                Err(e) => errors.push(format!("[keys] {chord:?}: {e}")),
            }
        }
        (map, errors)
    }

    pub fn command_for(&self, chord: &Chord) -> Option<&str> {
        self.bindings.get(chord).map(String::as_str)
    }

    /// Primeiro atalho de um comando, para mostrar na palette.
    pub fn chord_for(&self, command: &str) -> Option<&Chord> {
        self.chords_for(command).into_iter().next()
    }

    /// Todos os atalhos de um comando, o mais curto primeiro (ordem estável).
    pub fn chords_for(&self, command: &str) -> Vec<&Chord> {
        let mut found: Vec<&Chord> =
            self.bindings.iter().filter(|(_, c)| c.as_str() == command).map(|(k, _)| k).collect();
        found.sort_by_key(|c| c.to_string());
        found.sort_by_key(|c| c.to_string().len());
        found
    }

    /// Atalhos da IDE que valem mesmo com o foco num painel que usa o teclado todo
    /// (ex.: terminal): os com Ctrl+Shift (menos copiar/colar) e Ctrl+PageUp/PageDown.
    /// O resto (Ctrl+letra, F-teclas...) fica com o painel.
    pub fn wins_over_panels(&self, chord: &Chord) -> bool {
        if self.command_for(chord).is_none() || chord.alt || chord.logo {
            return false;
        }
        let ctrl_shift = chord.ctrl && chord.shift && !matches!(chord.key.as_str(), "c" | "v");
        let tab_switch = chord.ctrl && !chord.shift && matches!(chord.key.as_str(), "pageup" | "pagedown");
        ctrl_shift || tab_switch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_passthrough_rule() {
        let (km, _) = Keymap::from_pairs([
            ("ctrl+shift+p", "palette.open"),
            ("ctrl+shift+c", "x.copy"),
            ("ctrl+b", "filetree.toggle"),
            ("ctrl+pagedown", "tab.next"),
            ("f8", "problems.next"),
        ]);
        let wins = |t: &str| km.wins_over_panels(&Chord::parse(t).unwrap());
        assert!(wins("ctrl+shift+p"));
        assert!(wins("ctrl+pagedown"));
        assert!(!wins("ctrl+shift+c"), "copiar fica com o painel");
        assert!(!wins("ctrl+b"), "Ctrl+letra é do shell");
        assert!(!wins("f8"));
        assert!(!wins("ctrl+shift+z"), "sem comando, não intercepta");
    }

    #[test]
    fn config_roundtrip() {
        for t in ["ctrl+shift+p", "f5", "ctrl+alt+pagedown", "super+k", "ctrl++"] {
            assert_eq!(Chord::parse(t).unwrap().to_config(), t);
        }
        assert!(!Chord::parse("a").unwrap().is_bindable());
        assert!(!Chord::parse("shift+a").unwrap().is_bindable());
        assert!(Chord::parse("f2").unwrap().is_bindable());
        assert!(Chord::parse("ctrl+a").unwrap().is_bindable());
    }

    #[test]
    fn parse_basic() {
        let c = Chord::parse("Ctrl+Shift+P").unwrap();
        assert!(c.ctrl && c.shift && !c.alt && !c.logo);
        assert_eq!(c.key, "p");
        assert_eq!(c.to_string(), "Ctrl+Shift+P");
    }

    #[test]
    fn parse_named_and_aliases() {
        assert_eq!(Chord::parse("ctrl+pgup").unwrap().key, "pageup");
        assert_eq!(Chord::parse("esc").unwrap().key, "escape");
        assert_eq!(Chord::parse("ctrl++").unwrap().key, "+");
        assert!(Chord::parse("hyper+x").is_err());
        assert!(Chord::parse("ctrl+").is_err());
    }

    #[test]
    fn event_matches_parsed() {
        let shift_ctrl = Modifiers::CTRL | Modifiers::SHIFT;
        // Com shift o iced manda a letra maiúscula.
        let ev = Chord::from_event(&Key::Character("P".into()), shift_ctrl).unwrap();
        assert_eq!(ev, Chord::parse("ctrl+shift+p").unwrap());

        let ev = Chord::from_event(&Key::Named(Named::PageUp), Modifiers::CTRL).unwrap();
        assert_eq!(ev, Chord::parse("ctrl+pageup").unwrap());

        assert!(Chord::from_event(&Key::Named(Named::Control), Modifiers::CTRL).is_none());
    }

    #[test]
    fn keymap_override_and_unbind() {
        let (map, errors) = Keymap::from_pairs([
            ("ctrl+s", "file.save"),
            ("ctrl+b", "filetree.toggle"),
            ("ctrl+b", "view.toggle_left"),
            ("ctrl+s", ""),
            ("bad+x", "x"),
        ]);
        assert_eq!(errors.len(), 1);
        assert_eq!(map.command_for(&Chord::parse("ctrl+b").unwrap()), Some("view.toggle_left"));
        assert_eq!(map.command_for(&Chord::parse("ctrl+s").unwrap()), None);
        assert_eq!(map.chord_for("view.toggle_left").unwrap().to_string(), "Ctrl+B");
    }
}
