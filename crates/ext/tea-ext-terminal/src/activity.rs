//! O estado de cada aba do terminal (RFC 0024, M1.1): o que a barra mostra ao lado do nome, para saber, sem abrir
//! a aba, se o terminal escondido está rodando, mudou, pediu atenção ou terminou.
//!
//! Só a lógica, sem tela e sem relógio próprio (quem chama passa o `now`), para dar para testar.

use std::time::{Duration, Instant};

/// Saiu algo há menos que isto: a aba está "rodando".
pub const RUNNING_FOR: Duration = Duration::from_secs(2);

/// Um título (OSC 0/2) maior que isto é cortado na barra.
const TITLE_MAX: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Nada desde a última vez que a aba esteve na tela.
    Idle,
    /// Saiu algo há pouco.
    Running,
    /// Saiu algo enquanto estava escondida, e parou.
    Unseen,
    /// Pediu atenção enquanto estava escondida: o sino, ou um hook do agente.
    Attention(Attention),
    /// O comando terminou com este código.
    Exited(i32),
}

/// Por que a aba pede atenção.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attention {
    /// O sino (`\a`).
    Bell,
    /// O agente pediu permissão ou uma resposta (hook `Notification`).
    Asked,
    /// O agente terminou a vez e espera você (hook `Stop`).
    TurnDone,
}

#[derive(Debug, Clone, Default)]
pub struct Activity {
    last_output: Option<Instant>,
    unseen: bool,
    attention: Option<Attention>,
    title: Option<String>,
    exit: Option<i32>,
}

impl Activity {
    /// Chegou saída do processo (o `Wakeup` do alacritty).
    pub fn output(&mut self, now: Instant) {
        self.last_output = Some(now);
        self.unseen = true;
    }

    /// O sino, ou um sinal do agente. Um pedido do agente vale mais que o sino, e o sino não apaga o pedido.
    pub fn attention(&mut self, why: Attention) {
        if why != Attention::Bell || self.attention.is_none() {
            self.attention = Some(why);
        }
    }

    /// A aba está na tela: o que tinha para ver, foi visto.
    pub fn seen(&mut self) {
        self.unseen = false;
        self.attention = None;
    }

    /// O título que o programa mandou; `None` quando ele pede para voltar ao padrão.
    pub fn set_title(&mut self, title: Option<&str>) {
        self.title = title.map(clean_title).filter(|t| !t.is_empty());
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn exited(&mut self, code: i32) {
        self.exit = Some(code);
    }

    /// Ainda vale rever o estado daqui a pouco (o "rodando" passa sozinho, sem evento).
    pub fn ticking(&self, now: Instant) -> bool {
        self.exit.is_none() && self.last_output.is_some_and(|t| now.duration_since(t) < RUNNING_FOR)
    }

    /// O que a barra mostra. O fim vale mais que tudo; o pedido de atenção, mais que a saída.
    pub fn state(&self, now: Instant) -> State {
        if let Some(code) = self.exit {
            State::Exited(code)
        } else if let Some(why) = self.attention {
            State::Attention(why)
        } else if self.ticking(now) {
            State::Running
        } else if self.unseen {
            State::Unseen
        } else {
            State::Idle
        }
    }
}

/// Sem caracteres de controle (um título não pode mexer na barra) e cortado com `…`.
fn clean_title(title: &str) -> String {
    let clean: String = title.chars().filter(|c| !c.is_control()).collect();
    let clean = clean.trim();
    if clean.chars().count() <= TITLE_MAX {
        return clean.to_string();
    }
    let mut cut: String = clean.chars().take(TITLE_MAX - 1).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_runs_then_waits_to_be_seen() {
        let t0 = Instant::now();
        let mut a = Activity::default();
        assert_eq!(a.state(t0), State::Idle);
        a.output(t0);
        assert_eq!(a.state(t0), State::Running);
        assert!(a.ticking(t0));
        let later = t0 + RUNNING_FOR;
        assert_eq!(a.state(later), State::Unseen);
        assert!(!a.ticking(later));
        a.seen();
        assert_eq!(a.state(later), State::Idle);
    }

    #[test]
    fn attention_beats_output_and_clears_when_seen() {
        let t0 = Instant::now();
        let mut a = Activity::default();
        a.output(t0);
        a.attention(Attention::Bell);
        assert_eq!(a.state(t0), State::Attention(Attention::Bell));
        a.attention(Attention::Asked);
        a.attention(Attention::Bell);
        assert_eq!(a.state(t0), State::Attention(Attention::Asked), "o sino não apaga o pedido do agente");
        a.seen();
        assert_eq!(a.state(t0), State::Running);
    }

    #[test]
    fn exit_wins_and_stops_ticking() {
        let t0 = Instant::now();
        let mut a = Activity::default();
        a.output(t0);
        a.attention(Attention::TurnDone);
        a.exited(2);
        assert_eq!(a.state(t0), State::Exited(2));
        assert!(!a.ticking(t0));
        a.seen();
        assert_eq!(a.state(t0), State::Exited(2));
    }

    #[test]
    fn titles_are_clean_and_short() {
        let mut a = Activity::default();
        a.set_title(Some("  vim\u{1b}[31m main.rs \n"));
        assert_eq!(a.title(), Some("vim[31m main.rs"));
        a.set_title(Some(&"x".repeat(50)));
        assert_eq!(a.title().unwrap().chars().count(), TITLE_MAX);
        assert!(a.title().unwrap().ends_with('…'));
        a.set_title(Some("   "));
        assert_eq!(a.title(), None);
        a.set_title(Some("zsh"));
        a.set_title(None);
        assert_eq!(a.title(), None);
    }
}
