//! Variações das falas do mascote e um sorteio que não repete a linha anterior.
//!
//! As chaves existem em todos os `.ftl` do mascote. O teste de locales quebra se alguma faltar.

use std::cell::RefCell;
use std::collections::HashMap;

use tea_core::keymap::Keymap;

/// Sorteia um índice do pool sem devolver a mesma chave duas vezes seguidas.
#[derive(Debug, Clone)]
pub struct LinePicker {
    last: Option<&'static str>,
    state: u64,
}

impl LinePicker {
    pub fn new(seed: u64) -> Self {
        Self {
            last: None,
            state: seed,
        }
    }

    /// Uma chave do pool. Com uma opção só, ela se repete: não há outra.
    pub fn pick(&mut self, pool: &[&'static str]) -> &'static str {
        if pool.is_empty() {
            return "";
        }
        if pool.len() == 1 {
            self.last = Some(pool[0]);
            return pool[0];
        }
        for _ in 0..12 {
            let i = (self.next_u64() as usize) % pool.len();
            if self.last != Some(pool[i]) {
                self.last = Some(pool[i]);
                return pool[i];
            }
        }
        let key = pool
            .iter()
            .copied()
            .find(|k| self.last != Some(*k))
            .unwrap_or(pool[0]);
        self.last = Some(key);
        key
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

thread_local! {
    static WATCHER: RefCell<LinePicker> = RefCell::new(LinePicker::new(0x0A5C07));
}

/// Sorteia e formata na thread que lê `.idetea/` (um sorteio por evento, não por quadro).
pub fn format_once(pool: &[&'static str], pairs: &[(&str, &str)]) -> String {
    let key = WATCHER.with(|p| p.borrow_mut().pick(pool));
    format_key(key, pairs)
}

pub fn format_key(key: &str, pairs: &[(&str, &str)]) -> String {
    if pairs.is_empty() {
        return crate::LOADER.get(key);
    }
    let args: HashMap<&str, &str> = pairs.iter().copied().collect();
    crate::LOADER.get_args(key, args)
}

/// `{file.save}` vira o atalho de verdade (ou o texto de "está na paleta").
pub fn with_shortcuts(text: &str, keymap: &Keymap) -> String {
    crate::help::render_paragraph(text, keymap)
        .into_iter()
        .map(|(s, _)| s)
        .collect()
}

pub fn is_tip(key: &str) -> bool {
    key.starts_with("mascot-tip-")
}

/// Parte do dia para o cumprimento. 22–4 é madrugada, com um convite leve a descansar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayPart {
    Morning,
    Afternoon,
    Evening,
    Late,
}

impl DayPart {
    pub fn from_hour(hour: u32) -> Self {
        match hour % 24 {
            5..=11 => Self::Morning,
            12..=17 => Self::Afternoon,
            18..=21 => Self::Evening,
            _ => Self::Late,
        }
    }

    pub fn pool(self) -> &'static [&'static str] {
        match self {
            Self::Morning => MORNING,
            Self::Afternoon => AFTERNOON,
            Self::Evening => EVENING,
            Self::Late => LATE,
        }
    }
}

/// Fala ambiente quando o estado não tem um texto mais específico (tarefa, pergunta, LSP).
pub fn ambient(mood: &str) -> Option<&'static [&'static str]> {
    Some(match mood {
        "idle" => IDLE,
        "working" => WORKING,
        "thinking" => THINKING,
        "celebrating" => CELEBRATE,
        "error" => ERROR_MOOD,
        "curious" => HOVER_CURIOUS,
        "annoyed" => POKE_ANNOYED,
        _ => return None,
    })
}

#[cfg(test)]
pub fn all_pools() -> &'static [&'static [&'static str]] {
    &[
        IDLE,
        HOVER_CURIOUS,
        HOVER_ANNOYED,
        POKE_CURIOUS,
        POKE_DOUBLE,
        POKE_ANNOYED,
        TASK,
        TERM_PENDING,
        TERM_RUNNING,
        TERM_OK,
        TERM_ERR,
        ANSWERED,
        WAIT,
        ERRORS,
        SAVE,
        WAKE,
        SLEEP,
        MORNING,
        AFTERNOON,
        EVENING,
        LATE,
        TIPS,
        BUILD_OK,
        BUILD_ERR,
        WORKING,
        THINKING,
        CELEBRATE,
        ERROR_MOOD,
    ]
}

pub const IDLE: &[&str] = &[
    "mascot-idle-1",
    "mascot-idle-2",
    "mascot-idle-3",
    "mascot-idle-4",
    "mascot-idle-5",
    "mascot-idle-6",
    "mascot-idle-7",
    "mascot-idle-8",
];
pub const HOVER_CURIOUS: &[&str] = &[
    "mascot-hover-curious-1",
    "mascot-hover-curious-2",
    "mascot-hover-curious-3",
    "mascot-hover-curious-4",
    "mascot-hover-curious-5",
    "mascot-hover-curious-6",
    "mascot-hover-curious-7",
    "mascot-hover-curious-8",
];
pub const HOVER_ANNOYED: &[&str] = &[
    "mascot-hover-annoyed-1",
    "mascot-hover-annoyed-2",
    "mascot-hover-annoyed-3",
    "mascot-hover-annoyed-4",
    "mascot-hover-annoyed-5",
    "mascot-hover-annoyed-6",
];
pub const POKE_CURIOUS: &[&str] = &[
    "mascot-poke-curious-1",
    "mascot-poke-curious-2",
    "mascot-poke-curious-3",
    "mascot-poke-curious-4",
    "mascot-poke-curious-5",
    "mascot-poke-curious-6",
];
pub const POKE_DOUBLE: &[&str] = &[
    "mascot-poke-double-1",
    "mascot-poke-double-2",
    "mascot-poke-double-3",
    "mascot-poke-double-4",
    "mascot-poke-double-5",
    "mascot-poke-double-6",
];
pub const POKE_ANNOYED: &[&str] = &[
    "mascot-poke-annoyed-1",
    "mascot-poke-annoyed-2",
    "mascot-poke-annoyed-3",
    "mascot-poke-annoyed-4",
    "mascot-poke-annoyed-5",
    "mascot-poke-annoyed-6",
];
pub const TASK: &[&str] = &[
    "mascot-task-1",
    "mascot-task-2",
    "mascot-task-3",
    "mascot-task-4",
    "mascot-task-5",
    "mascot-task-6",
];
pub const TERM_PENDING: &[&str] = &[
    "mascot-term-pending-1",
    "mascot-term-pending-2",
    "mascot-term-pending-3",
    "mascot-term-pending-4",
    "mascot-term-pending-5",
    "mascot-term-pending-6",
];
pub const TERM_RUNNING: &[&str] = &[
    "mascot-term-running-1",
    "mascot-term-running-2",
    "mascot-term-running-3",
    "mascot-term-running-4",
    "mascot-term-running-5",
    "mascot-term-running-6",
];
pub const TERM_OK: &[&str] = &[
    "mascot-term-ok-1",
    "mascot-term-ok-2",
    "mascot-term-ok-3",
    "mascot-term-ok-4",
    "mascot-term-ok-5",
    "mascot-term-ok-6",
];
pub const TERM_ERR: &[&str] = &[
    "mascot-term-err-1",
    "mascot-term-err-2",
    "mascot-term-err-3",
    "mascot-term-err-4",
    "mascot-term-err-5",
    "mascot-term-err-6",
];
pub const ANSWERED: &[&str] = &[
    "mascot-answered-1",
    "mascot-answered-2",
    "mascot-answered-3",
    "mascot-answered-4",
    "mascot-answered-5",
    "mascot-answered-6",
];
pub const WAIT: &[&str] = &[
    "mascot-wait-1",
    "mascot-wait-2",
    "mascot-wait-3",
    "mascot-wait-4",
    "mascot-wait-5",
];
pub const ERRORS: &[&str] = &[
    "mascot-errors-1",
    "mascot-errors-2",
    "mascot-errors-3",
    "mascot-errors-4",
    "mascot-errors-5",
    "mascot-errors-6",
];
pub const SAVE: &[&str] = &[
    "mascot-save-1",
    "mascot-save-2",
    "mascot-save-3",
    "mascot-save-4",
    "mascot-save-5",
    "mascot-save-6",
];
pub const WAKE: &[&str] = &[
    "mascot-wake-1",
    "mascot-wake-2",
    "mascot-wake-3",
    "mascot-wake-4",
    "mascot-wake-5",
    "mascot-wake-6",
];
pub const SLEEP: &[&str] = &[
    "mascot-sleep-1",
    "mascot-sleep-2",
    "mascot-sleep-3",
    "mascot-sleep-4",
    "mascot-sleep-5",
    "mascot-sleep-6",
];
pub const MORNING: &[&str] = &[
    "mascot-greet-morning-1",
    "mascot-greet-morning-2",
    "mascot-greet-morning-3",
    "mascot-greet-morning-4",
    "mascot-greet-morning-5",
];
pub const AFTERNOON: &[&str] = &[
    "mascot-greet-afternoon-1",
    "mascot-greet-afternoon-2",
    "mascot-greet-afternoon-3",
    "mascot-greet-afternoon-4",
    "mascot-greet-afternoon-5",
];
pub const EVENING: &[&str] = &[
    "mascot-greet-evening-1",
    "mascot-greet-evening-2",
    "mascot-greet-evening-3",
    "mascot-greet-evening-4",
    "mascot-greet-evening-5",
];
pub const LATE: &[&str] = &[
    "mascot-greet-late-1",
    "mascot-greet-late-2",
    "mascot-greet-late-3",
    "mascot-greet-late-4",
    "mascot-greet-late-5",
];
pub const TIPS: &[&str] = &[
    "mascot-tip-1",
    "mascot-tip-2",
    "mascot-tip-3",
    "mascot-tip-4",
    "mascot-tip-5",
    "mascot-tip-6",
    "mascot-tip-7",
    "mascot-tip-8",
];
pub const BUILD_OK: &[&str] = &[
    "mascot-build-ok-1",
    "mascot-build-ok-2",
    "mascot-build-ok-3",
    "mascot-build-ok-4",
    "mascot-build-ok-5",
];
pub const BUILD_ERR: &[&str] = &[
    "mascot-build-err-1",
    "mascot-build-err-2",
    "mascot-build-err-3",
    "mascot-build-err-4",
    "mascot-build-err-5",
];
pub const WORKING: &[&str] = &[
    "mascot-working-1",
    "mascot-working-2",
    "mascot-working-3",
    "mascot-working-4",
    "mascot-working-5",
];
pub const THINKING: &[&str] = &[
    "mascot-thinking-1",
    "mascot-thinking-2",
    "mascot-thinking-3",
    "mascot-thinking-4",
    "mascot-thinking-5",
];
pub const CELEBRATE: &[&str] = &[
    "mascot-celebrate-1",
    "mascot-celebrate-2",
    "mascot-celebrate-3",
    "mascot-celebrate-4",
    "mascot-celebrate-5",
];
pub const ERROR_MOOD: &[&str] = &[
    "mascot-error-1",
    "mascot-error-2",
    "mascot-error-3",
    "mascot-error-4",
    "mascot-error-5",
];

/// Atalhos citados nas dicas. O teste confere que o config padrão liga cada um.
#[cfg(test)]
pub const TIP_COMMANDS: &[&str] = &[
    "palette.open",
    "file.quick_open",
    "file.save",
    "help.open",
    "tasks.run",
    "terminal.focus",
    "problems.focus",
    "problems.next",
    "lsp.definition",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_does_not_repeat_the_same_line_twice_in_a_row() {
        let pool = ["a", "b", "c", "d", "e"];
        let mut first = LinePicker::new(7);
        let mut second = LinePicker::new(7);
        let mut prev = first.pick(&pool);
        assert_eq!(
            second.pick(&pool),
            prev,
            "a mesma semente, a mesma sequência"
        );
        for _ in 0..40 {
            let next = first.pick(&pool);
            assert_ne!(next, prev);
            assert_eq!(second.pick(&pool), next);
            prev = next;
        }
        let mut only = LinePicker::new(1);
        assert_eq!(only.pick(&["solo"]), "solo");
        assert_eq!(only.pick(&["solo"]), "solo");
        assert_eq!(LinePicker::new(1).pick(&[]), "");
    }

    #[test]
    fn common_pools_have_several_lines() {
        for pool in [IDLE, HOVER_CURIOUS, POKE_CURIOUS, TASK, SAVE, TIPS, SLEEP] {
            assert!(pool.len() >= 5 && pool.len() <= 8, "{}", pool[0]);
        }
        let mut seen = std::collections::BTreeSet::new();
        for pool in all_pools() {
            assert!(!pool.is_empty());
            for key in *pool {
                assert!(seen.insert(*key), "chave repetida: {key}");
                assert!(key.starts_with("mascot-"), "{key}");
            }
        }
    }

    #[test]
    fn day_part_follows_the_hour() {
        assert_eq!(DayPart::from_hour(8), DayPart::Morning);
        assert_eq!(DayPart::from_hour(12), DayPart::Afternoon);
        assert_eq!(DayPart::from_hour(18), DayPart::Evening);
        assert_eq!(DayPart::from_hour(23), DayPart::Late);
        assert_eq!(DayPart::from_hour(3), DayPart::Late);
        assert_eq!(DayPart::from_hour(4), DayPart::Late);
        assert_eq!(DayPart::from_hour(5), DayPart::Morning);
    }
}
