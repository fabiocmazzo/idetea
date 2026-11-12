//! Mascote reativo na IDE (RFC 0012): card flutuante no canto inferior direito (recolhível na barra de status)
//! que acompanha o estado dos agentes e do projeto em tempo real.
//!
//! Fontes de estado (da maior prioridade para a menor):
//! 1. Pergunta aberta em `.idetea/ui/<nome>.json` sem resposta e com `<nome>.wait` recente → `Attention` (clicar abre Interfaces);
//! 2. Pedido de terminal em `.idetea/agents/<nome>.status.json` com `state = "pending"` → `Attention` (clicar abre Terminal);
//! 3. Estado explícito gravado por `idetea mascot <estado> [--text "…"]` em `.idetea/mascot.json` (com TTL);
//! 4. Transições recentes: terminal/tarefa falhou (`Error` por 8 s) ou pergunta respondida / terminal concluiu com 0 (`Celebrating` por 5 s);
//! 5. Terminal de agente em `running` ou tarefa rodando → `Working`;
//! 6. Atividade de LSP em segundo plano → `Thinking`;
//! 7. Pergunta aberta cujo `.wait` parou de atualizar (> 30 s) ou inatividade prolongada (> 5 min) → `Sleeping`;
//! 8. Caso contrário → `Idle` (xícara de chá fumegando suavemente).
//!
//! Por cima disso, sem roubar o foco: salvar um arquivo, o build do depurador, sono depois de
//! um tempo parado, cumprimento conforme a hora e, se `[mascot] tips` estiver ligado, uma dica
//! ocasional de atalho que existe de verdade. `chatty = "quiet"` fica só com o estado.

mod lines;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use iced::widget::image::{FilterMethod, Handle};
use iced::widget::{
    button, column, container, image as img_widget, mouse_area, opaque, row, space, text,
};
use iced::{Color, Element, Fill, Point, Theme};
use serde::{Deserialize, Serialize};

pub const FORMAT: &str = "idetea-mascot";
const POLL_INTERVAL: Duration = Duration::from_millis(250);
const WAIT_FRESH: Duration = Duration::from_secs(30);
const DEFAULT_TTL_SECS: u64 = 30;
const CELEBRATE_DURATION: Duration = Duration::from_secs(5);
const ERROR_DURATION: Duration = Duration::from_secs(8);
const SLEEP_AFTER_IDLE: Duration = Duration::from_secs(300);
const POKE_WINDOW: Duration = Duration::from_secs(6);
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
const HOVER_PASS_WINDOW: Duration = Duration::from_secs(4);
const WIGGLE_THRESHOLD: f32 = 220.0;
const ASIDE_SAVE: Duration = Duration::from_secs(4);
const ASIDE_WAKE: Duration = Duration::from_secs(5);
const ASIDE_GREET: Duration = Duration::from_secs(7);
const ASIDE_TIP: Duration = Duration::from_secs(8);
const TIP_EVERY: Duration = Duration::from_secs(180);
const TIP_EVERY_CHATTY: Duration = Duration::from_secs(70);
const IDLE_ROLL: Duration = Duration::from_secs(40);

const EXIT_OK: i32 = 0;
const EXIT_INVALID: i32 = 2;

/// Estados possíveis do mascote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    Idle,
    Working,
    Thinking,
    Attention,
    Celebrating,
    Error,
    Sleeping,
    Curious,
    Annoyed,
}

impl Mood {
    pub const ALL: [Mood; 9] = [
        Mood::Idle,
        Mood::Working,
        Mood::Thinking,
        Mood::Attention,
        Mood::Celebrating,
        Mood::Error,
        Mood::Sleeping,
        Mood::Curious,
        Mood::Annoyed,
    ];

    pub fn parse(word: &str) -> Option<Self> {
        match word.trim().to_ascii_lowercase().as_str() {
            "idle" | "calmo" => Some(Self::Idle),
            "working" | "work" | "busy" | "trabalhando" => Some(Self::Working),
            "thinking" | "think" | "pensando" => Some(Self::Thinking),
            "attention" | "alert" | "ask" | "waiting" | "atencao" => Some(Self::Attention),
            "celebrating" | "happy" | "done" | "success" | "pronto" => Some(Self::Celebrating),
            "error" | "fail" | "failed" | "erro" => Some(Self::Error),
            "sleeping" | "sleep" | "dormindo" => Some(Self::Sleeping),
            "curious" | "curioso" | "peek" => Some(Self::Curious),
            "annoyed" | "irritado" | "angry" | "grumpy" | "bravo" => Some(Self::Annoyed),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Thinking => "thinking",
            Self::Attention => "attention",
            Self::Celebrating => "celebrating",
            Self::Error => "error",
            Self::Sleeping => "sleeping",
            Self::Curious => "curious",
            Self::Annoyed => "annoyed",
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Idle => fl!("mascot-state-idle"),
            Self::Working => fl!("mascot-state-working"),
            Self::Thinking => fl!("mascot-state-thinking"),
            Self::Attention => fl!("mascot-state-attention"),
            Self::Celebrating => fl!("mascot-state-celebrating"),
            Self::Error => fl!("mascot-state-error"),
            Self::Sleeping => fl!("mascot-state-sleeping"),
            Self::Curious => fl!("mascot-state-curious"),
            Self::Annoyed => fl!("mascot-state-annoyed"),
        }
    }

    pub fn default_fps(self) -> u32 {
        match self {
            Self::Idle | Self::Sleeping => 2,
            Self::Thinking | Self::Curious => 4,
            Self::Working | Self::Attention | Self::Celebrating | Self::Error => 6,
            Self::Annoyed => 8,
        }
    }
}

/// Para onde o clique no balão do mascote leva.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    None,
    Interfaces,
    Terminal,
    Problems,
}

impl Target {
    pub fn command(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Interfaces => Some("interfaces.show"),
            Self::Terminal => Some("terminal.focus"),
            Self::Problems => Some("problems.focus"),
        }
    }
}

/// Mensagens da interface do mascote para o `App`.
#[derive(Debug, Clone)]
pub enum Msg {
    Tick,
    Snapshot(Arc<DiskSnapshot>),
    ToggleCollapsed,
    ActivateTarget,
    PointerEntered,
    PointerMoved(Point),
    PointerExited,
    Poke,
}

/// Estado gravado por `idetea mascot` em `.idetea/mascot.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExternalState {
    pub format: String,
    pub version: u32,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub timestamp_ms: u64,
    pub ttl_secs: u64,
}

impl Default for ExternalState {
    fn default() -> Self {
        Self {
            format: FORMAT.into(),
            version: 1,
            state: "idle".into(),
            text: None,
            source: None,
            timestamp_ms: 0,
            ttl_secs: DEFAULT_TTL_SECS,
        }
    }
}

impl ExternalState {
    pub fn is_active(&self, now_ms: u64) -> bool {
        self.format == FORMAT
            && Mood::parse(&self.state).is_some()
            && now_ms
                <= self
                    .timestamp_ms
                    .saturating_add(self.ttl_secs.saturating_mul(1000))
    }
}

/// O que a thread de observação lê de `.idetea/` (fora da thread da interface).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiskSnapshot {
    /// Pergunta aberta esperando resposta (`(nome, título, origem, wait_fresco, mtime_ms)`).
    pub ui_waiting: Option<(String, String, Option<String>, bool, u64)>,
    /// Quantidade total de perguntas abertas sem resposta.
    pub ui_open_count: usize,
    /// Pedido de terminal aguardando confirmação (`(nome, título, mtime_ms)`).
    pub agent_pending: Option<(String, String, u64)>,
    /// Terminal de agente rodando (`(nome, título)`).
    pub agent_running: Option<(String, String)>,
    /// Transição detectada nesta leitura (`Celebrating` ou `Error` com texto e alvo).
    pub event: Option<(Mood, String, Target)>,
    /// Estado explícito em `.idetea/mascot.json`, se ainda dentro do TTL (`(mood, text, source, timestamp_ms)`).
    pub external: Option<(Mood, Option<String>, Option<String>, u64)>,
    /// Pacote customizado lido de `mascot.toml`, quando muda.
    pub custom_pack: Option<Arc<LoadedPack>>,
}

/// Quadros e FPS de cada estado num pacote de mascote.
#[derive(Debug, Clone)]
pub struct LoadedPack {
    pub name: String,
    pub pixelated: bool,
    pub states: HashMap<Mood, (Vec<Handle>, u32)>,
    pub source_id: String,
}

impl PartialEq for LoadedPack {
    fn eq(&self, other: &Self) -> bool {
        self.source_id == other.source_id
            && self.name == other.name
            && self.pixelated == other.pixelated
    }
}
impl Eq for LoadedPack {}

/// Fala curta que não cobre pergunta, erro nem tarefa (some sozinha).
struct Aside {
    mood: Mood,
    text: String,
    until: Instant,
}

/// Estado do mascote dentro do `App`.
pub struct Mascot {
    pub collapsed: bool,
    pack: Arc<LoadedPack>,
    builtin: Arc<LoadedPack>,
    snapshot: DiskSnapshot,
    /// Reação temporária (comemoração ou erro recém-acontecido) com prazo de validade.
    transient: Option<(Mood, String, Target, Instant)>,
    /// Reação direta a interação com o mouse (cliques ou sacudidas) com prazo de validade.
    interaction: Option<(Mood, String, Instant)>,
    /// Se o ponteiro do mouse está em cima do card do mascote.
    hovered: bool,
    /// Entradas recentes do mouse no card (para detectar mouse passando repetidamente).
    hover_enters: Vec<Instant>,
    /// Última posição do ponteiro sobre o mascote e quando ocorreu.
    last_pointer: Option<(Point, Instant)>,
    /// Distância acumulada de movimento rápido sobre o mascote.
    hover_wiggle: f32,
    /// Cliques recentes no mascote.
    pokes: Vec<Instant>,
    /// Contador para alternar frases de curiosidade/irritação.
    interaction_seq: usize,
    chatty: tea_core::config::MascotChatty,
    tips: bool,
    keymap: tea_core::keymap::Keymap,
    picker: RefCell<lines::LinePicker>,
    /// `(situação, texto)` para a fala não trocar a cada quadro.
    line_cache: RefCell<(String, String)>,
    hover_line: Option<String>,
    aside: Option<Aside>,
    greeted: Option<lines::DayPart>,
    last_tip: Instant,
    last_idle_roll: Instant,
    /// Atividade de LSP em segundo plano (fonte → texto).
    lsp_activity: BTreeMap<String, String>,
    /// Quantidade de erros do LSP no projeto.
    lsp_errors: usize,
    /// Última interação do usuário ou mudança de estado de agente.
    last_active: Instant,
    /// Quadro atual da animação.
    frame: usize,
}

impl Mascot {
    pub fn new(collapsed: bool) -> Self {
        let builtin = Arc::new(builtin_teacup_pack());
        Self {
            collapsed,
            pack: builtin.clone(),
            builtin,
            snapshot: DiskSnapshot::default(),
            transient: None,
            interaction: None,
            hovered: false,
            hover_enters: Vec::new(),
            last_pointer: None,
            hover_wiggle: 0.0,
            pokes: Vec::new(),
            interaction_seq: 0,
            chatty: tea_core::config::MascotChatty::Normal,
            tips: true,
            keymap: tea_core::keymap::Keymap::default(),
            picker: RefCell::new(lines::LinePicker::new(picker_seed())),
            line_cache: RefCell::new((String::new(), String::new())),
            hover_line: None,
            aside: None,
            greeted: None,
            last_tip: Instant::now(),
            last_idle_roll: Instant::now(),
            lsp_activity: BTreeMap::new(),
            lsp_errors: 0,
            last_active: Instant::now(),
            frame: 0,
        }
    }

    /// Nível de conversa, dicas e atalhos de verdade (para as dicas citarem o atalho atual).
    pub fn configure(
        &mut self,
        cfg: &tea_core::config::MascotConfig,
        keymap: &tea_core::keymap::Keymap,
    ) {
        self.chatty = cfg.chatty;
        self.tips = cfg.tips;
        self.keymap = keymap.clone();
    }

    pub fn touch_user_activity(&mut self) {
        self.note_activity(true);
    }

    fn note_activity(&mut self, wake_line: bool) {
        let asleep = self.last_active.elapsed() >= SLEEP_AFTER_IDLE;
        self.last_active = Instant::now();
        if asleep
            && wake_line
            && self.chatty != tea_core::config::MascotChatty::Quiet
            && !self.aside_active()
        {
            let text = self.pick_text(lines::WAKE);
            self.show_aside(Mood::Idle, text, ASIDE_WAKE);
        }
    }

    pub fn on_pointer_entered(&mut self) {
        let now = Instant::now();
        self.note_activity(false);
        self.hovered = true;
        self.hover_enters
            .retain(|t| now.duration_since(*t) <= HOVER_PASS_WINDOW);
        self.hover_enters.push(now);
        self.interaction_seq = self.interaction_seq.wrapping_add(1);

        if self.hover_enters.len() >= 3 {
            let msg = self.pick_text(lines::HOVER_ANNOYED);
            self.interaction = Some((Mood::Annoyed, msg, now + Duration::from_secs(4)));
        } else {
            self.hover_line = Some(self.pick_text(lines::HOVER_CURIOUS));
        }
    }

    pub fn on_pointer_moved(&mut self, pt: Point) {
        let now = Instant::now();
        self.note_activity(false);
        self.hovered = true;
        if let Some((prev, prev_t)) = self.last_pointer {
            if now.duration_since(prev_t) <= Duration::from_millis(600) {
                let dx = pt.x - prev.x;
                let dy = pt.y - prev.y;
                self.hover_wiggle += (dx * dx + dy * dy).sqrt();
            } else {
                self.hover_wiggle = 0.0;
            }
        }
        self.last_pointer = Some((pt, now));

        if self.hover_wiggle >= WIGGLE_THRESHOLD {
            self.hover_wiggle = 0.0;
            self.interaction_seq = self.interaction_seq.wrapping_add(1);
            let msg = self.pick_text(lines::HOVER_ANNOYED);
            self.interaction = Some((Mood::Annoyed, msg, now + Duration::from_secs(4)));
        }
    }

    pub fn on_pointer_exited(&mut self) {
        self.hovered = false;
        self.last_pointer = None;
    }

    pub fn on_poke(&mut self) {
        let now = Instant::now();
        self.note_activity(false);
        let rapid = self
            .pokes
            .last()
            .is_some_and(|t| now.duration_since(*t) <= DOUBLE_CLICK);
        self.pokes.retain(|t| now.duration_since(*t) <= POKE_WINDOW);
        self.pokes.push(now);
        let count = self.pokes.len();

        let (mood, pool, dur) = if rapid && count == 2 {
            (
                Mood::Curious,
                lines::POKE_DOUBLE,
                Duration::from_millis(3500),
            )
        } else {
            match count {
                1 | 2 => (
                    Mood::Curious,
                    lines::POKE_CURIOUS,
                    Duration::from_millis(3500),
                ),
                3 | 4 => (
                    Mood::Annoyed,
                    lines::POKE_ANNOYED,
                    Duration::from_millis(4500),
                ),
                _ => (Mood::Annoyed, lines::POKE_ANNOYED, Duration::from_secs(5)),
            }
        };
        self.interaction = Some((mood, self.pick_text(pool), now + dur));
    }

    /// Arquivo gravado pela IDE. `quiet` não comenta.
    pub fn notify_saved(&mut self, name: &str) {
        self.last_active = Instant::now();
        if name.is_empty() || self.chatty == tea_core::config::MascotChatty::Quiet {
            return;
        }
        let text = self.format_fresh(lines::SAVE, &[("name", name)]);
        self.show_aside(Mood::Celebrating, text, ASIDE_SAVE);
    }

    /// Resultado do build que o depurador já conhece (não há evento de fim para as tarefas do terminal).
    pub fn notify_build(&mut self, ok: bool) {
        self.last_active = Instant::now();
        if ok {
            let text = self.pick_text(lines::BUILD_OK);
            self.transient = Some((
                Mood::Celebrating,
                text,
                Target::None,
                Instant::now() + CELEBRATE_DURATION,
            ));
        } else {
            let text = self.pick_text(lines::BUILD_ERR);
            self.transient = Some((
                Mood::Error,
                text,
                Target::Problems,
                Instant::now() + ERROR_DURATION,
            ));
        }
    }

    pub fn set_lsp_activity(&mut self, source: &str, text: Option<&str>) {
        match text {
            Some(t) => {
                self.lsp_activity.insert(source.to_string(), t.to_string());
            }
            None => {
                self.lsp_activity.remove(source);
            }
        }
    }

    pub fn set_lsp_errors(&mut self, total_errors: usize) {
        self.lsp_errors = total_errors;
    }

    pub fn notify_task_started(&mut self, name: &str) {
        self.last_active = Instant::now();
        let text = self.format_fresh(lines::TASK, &[("name", name)]);
        self.transient = Some((
            Mood::Working,
            text,
            Target::Terminal,
            Instant::now() + Duration::from_secs(6),
        ));
    }

    pub fn apply_snapshot(&mut self, mut snap: DiskSnapshot) {
        self.last_active = Instant::now();
        if let Some(pack) = snap.custom_pack.take() {
            if pack.states.is_empty() {
                self.pack = self.builtin.clone();
            } else {
                self.pack = pack;
            }
        }
        if let Some((mood, text, target)) = snap.event.take() {
            let dur = if mood == Mood::Error {
                ERROR_DURATION
            } else {
                CELEBRATE_DURATION
            };
            self.transient = Some((mood, text, target, Instant::now() + dur));
        }
        self.snapshot = snap;
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        let now = Instant::now();
        if self
            .transient
            .as_ref()
            .is_some_and(|(_, _, _, until)| now >= *until)
        {
            self.transient = None;
        }
        if self
            .interaction
            .as_ref()
            .is_some_and(|(_, _, until)| now >= *until)
        {
            self.interaction = None;
        }
        if self.aside.as_ref().is_some_and(|a| now >= a.until) {
            self.aside = None;
        }
        self.consider_greeting(local_hour());
        self.consider_tip();
        self.consider_idle_refresh();
    }

    fn fallback_target(&self) -> Target {
        if self.snapshot.ui_waiting.as_ref().is_some_and(|w| w.3) {
            Target::Interfaces
        } else if self.snapshot.agent_pending.is_some() {
            Target::Terminal
        } else {
            Target::None
        }
    }

    /// Resolve `(Mood, texto_do_balão, Target)` segundo a ordem de prioridade da RFC 0012.
    pub fn resolved(&self) -> (Mood, Option<String>, Target) {
        // 0. Interação direta do usuário com o mascote (cliques, sacudidas ou ponteiro sobre o mascote).
        if let Some((mood, text, until)) = &self.interaction
            && Instant::now() < *until
        {
            return (*mood, Some(text.clone()), self.fallback_target());
        }
        if self.hovered {
            let msg = self
                .hover_line
                .clone()
                .unwrap_or_else(|| self.cached_line("hover", lines::HOVER_CURIOUS));
            return (Mood::Curious, Some(msg), self.fallback_target());
        }

        let base = self.resolve_base();
        // Cumprimento, dica, salvar e acordar: só quando não há nada mais urgente, e sem mudar o foco.
        if let Some(aside) = &self.aside
            && Instant::now() < aside.until
            && matches!(base.0, Mood::Idle | Mood::Sleeping | Mood::Thinking)
        {
            return (aside.mood, Some(aside.text.clone()), Target::None);
        }
        base
    }

    /// O que o card mostra. `compact` é a barra de status (recolhido): sem a conversa de descanso.
    fn present(&self, compact: bool) -> (Mood, Option<String>, Target) {
        let mut shown = self.resolved();
        if shown.1.is_none() && shown.0 == Mood::Sleeping {
            shown.1 = Some(self.cached_line("mood:sleep", lines::SLEEP));
        } else if !compact
            && shown.1.is_none()
            && self.chatty != tea_core::config::MascotChatty::Quiet
            && let Some(pool) = lines::ambient(shown.0.as_str())
        {
            let situation = format!("mood:{}", shown.0.as_str());
            shown.1 = Some(self.cached_line(&situation, pool));
        }
        shown
    }

    fn resolve_base(&self) -> (Mood, Option<String>, Target) {
        let ext_ts = self
            .snapshot
            .external
            .as_ref()
            .map(|(_, _, _, ts)| *ts)
            .unwrap_or(0);

        // 1. Pergunta aberta no painel Interfaces com `.wait` ativo que chegou depois do último `idetea mascot`.
        if let Some((_, title, source, true, ui_mtime)) = &self.snapshot.ui_waiting
            && (self.snapshot.external.is_none() || *ui_mtime > ext_ts)
        {
            let text = match source.as_deref().filter(|s| !s.is_empty()) {
                Some(src) => format!("{src}: {title}"),
                None => title.clone(),
            };
            return (Mood::Attention, Some(text), Target::Interfaces);
        }
        // 2. Pedido de terminal aguardando confirmação que chegou depois do último `idetea mascot`.
        if let Some((_, title, agent_mtime)) = &self.snapshot.agent_pending
            && (self.snapshot.external.is_none() || *agent_mtime > ext_ts)
        {
            let text = self.format_cached(
                &format!("term-pending:{title}"),
                lines::TERM_PENDING,
                &[("title", title.as_str())],
            );
            return (Mood::Attention, Some(text), Target::Terminal);
        }
        // 3. Estado explícito ativo enviado por `idetea mascot` (dentro do TTL).
        if let Some((mood, text, source, _)) = &self.snapshot.external {
            return (
                *mood,
                format_external(source.as_deref(), text.as_deref()),
                self.fallback_target(),
            );
        }
        // 4. Transição recente (erro ou comemoração).
        if let Some((mood, text, target, until)) = &self.transient
            && Instant::now() < *until
        {
            return (*mood, Some(text.clone()), target.clone());
        }
        // 5. Terminal de agente rodando.
        if let Some((_, title)) = &self.snapshot.agent_running {
            let text = self.format_cached(
                &format!("term-run:{title}"),
                lines::TERM_RUNNING,
                &[("title", title.as_str())],
            );
            return (Mood::Working, Some(text), Target::Terminal);
        }
        // 6. Atividade de LSP em segundo plano.
        if let Some((src, act)) = self.lsp_activity.iter().next() {
            return (Mood::Thinking, Some(format!("{src}: {act}")), Target::None);
        }
        // 7. Pergunta aberta mas o `.wait` expirou (o agente parou de esperar).
        if let Some((_, title, _, false, _)) = &self.snapshot.ui_waiting {
            let text = self.format_cached(
                &format!("wait:{title}"),
                lines::WAIT,
                &[("title", title.as_str())],
            );
            return (Mood::Sleeping, Some(text), Target::Interfaces);
        }
        // 8. Inatividade prolongada.
        if self.last_active.elapsed() >= SLEEP_AFTER_IDLE {
            return (Mood::Sleeping, None, Target::None);
        }
        // 9. Calmo (mostra lembrete discreto se houver erros de LSP).
        if self.lsp_errors > 0 {
            let n = self.lsp_errors.to_string();
            let text = self.format_cached(
                &format!("errors:{n}"),
                lines::ERRORS,
                &[("count", n.as_str())],
            );
            return (Mood::Idle, Some(text), Target::Problems);
        }
        (Mood::Idle, None, Target::None)
    }

    fn aside_active(&self) -> bool {
        self.aside
            .as_ref()
            .is_some_and(|a| Instant::now() < a.until)
    }

    fn show_aside(&mut self, mood: Mood, text: String, ttl: Duration) {
        self.aside = Some(Aside {
            mood,
            text,
            until: Instant::now() + ttl,
        });
    }

    fn pick_text(&self, pool: &[&'static str]) -> String {
        let key = self.picker.borrow_mut().pick(pool);
        self.render_key(key)
    }

    fn render_key(&self, key: &str) -> String {
        if lines::is_tip(key) {
            let marked = tea_core::i18n::get_with_markers(&crate::LOADER, key);
            return lines::with_shortcuts(&marked, &self.keymap);
        }
        crate::LOADER.get(key)
    }

    fn format_fresh(&self, pool: &[&'static str], pairs: &[(&str, &str)]) -> String {
        let key = self.picker.borrow_mut().pick(pool);
        lines::format_key(key, pairs)
    }

    fn format_cached(
        &self,
        situation: &str,
        pool: &[&'static str],
        pairs: &[(&str, &str)],
    ) -> String {
        if self.line_cache.borrow().0 == situation {
            return self.line_cache.borrow().1.clone();
        }
        let text = self.format_fresh(pool, pairs);
        *self.line_cache.borrow_mut() = (situation.to_string(), text.clone());
        text
    }

    fn cached_line(&self, situation: &str, pool: &[&'static str]) -> String {
        if self.line_cache.borrow().0 == situation {
            return self.line_cache.borrow().1.clone();
        }
        let text = self.pick_text(pool);
        *self.line_cache.borrow_mut() = (situation.to_string(), text.clone());
        text
    }

    fn consider_greeting(&mut self, hour: u32) {
        if self.collapsed
            || self.chatty == tea_core::config::MascotChatty::Quiet
            || self.hovered
            || self.aside_active()
        {
            return;
        }
        let part = lines::DayPart::from_hour(hour);
        if self.greeted == Some(part) {
            return;
        }
        if self.resolve_base().0 != Mood::Idle {
            return;
        }
        self.greeted = Some(part);
        let text = self.pick_text(part.pool());
        self.show_aside(Mood::Idle, text, ASIDE_GREET);
    }

    fn consider_tip(&mut self) {
        if self.collapsed
            || !self.tips
            || self.chatty == tea_core::config::MascotChatty::Quiet
            || self.hovered
        {
            return;
        }
        let every = if self.chatty == tea_core::config::MascotChatty::Chatty {
            TIP_EVERY_CHATTY
        } else {
            TIP_EVERY
        };
        if self.last_tip.elapsed() < every || self.aside_active() {
            return;
        }
        if self.resolve_base().0 != Mood::Idle {
            return;
        }
        self.last_tip = Instant::now();
        let text = self.pick_text(lines::TIPS);
        self.show_aside(Mood::Curious, text, ASIDE_TIP);
    }

    fn consider_idle_refresh(&mut self) {
        if self.collapsed
            || self.chatty != tea_core::config::MascotChatty::Chatty
            || self.aside_active()
        {
            return;
        }
        if self.last_idle_roll.elapsed() < IDLE_ROLL {
            return;
        }
        let (mood, text, _) = self.resolve_base();
        if mood != Mood::Idle || text.is_some() {
            return;
        }
        self.last_idle_roll = Instant::now();
        self.line_cache.borrow_mut().0.clear();
    }

    /// Intervalo do relógio de animação (desligado quando recolhido).
    pub fn frame_interval(&self, enabled: bool) -> Option<Duration> {
        if !enabled || self.collapsed {
            return None;
        }
        let (mood, _, _) = self.resolved();
        let fps = self
            .pack
            .states
            .get(&mood)
            .map(|(_, fps)| *fps)
            .unwrap_or_else(|| mood.default_fps())
            .clamp(1, 24);
        Some(Duration::from_millis((1000 / u64::from(fps)).max(40)))
    }

    fn current_handle(&self, mood: Mood) -> (Handle, bool) {
        let (frames, _) = self
            .pack
            .states
            .get(&mood)
            .or_else(|| self.builtin.states.get(&mood))
            .expect("todos os estados têm quadros embutidos");
        let idx = self.frame % frames.len().max(1);
        (frames[idx].clone(), self.pack.pixelated)
    }

    /// Botão compacto na barra de status (sempre visível quando `[mascot] enabled = true`).
    pub fn status_bar_button(&self, enabled: bool) -> Option<Element<'_, Msg>> {
        if !enabled {
            return None;
        }
        let (mood, bubble, _) = self.present(true);
        let badge = match mood {
            Mood::Attention => " !",
            Mood::Error => " ×",
            Mood::Working => " …",
            Mood::Celebrating => " ★",
            Mood::Curious => " ?",
            Mood::Annoyed => " #",
            _ => "",
        };
        let label = if self.collapsed
            && let Some(b) = &bubble
        {
            let short = truncate_chars(b, 32);
            format!("🍵 {}{badge} · {short}", mood.label())
        } else {
            format!("🍵 {}{badge}", self.pack.name)
        };
        let urgent = matches!(mood, Mood::Attention | Mood::Error | Mood::Annoyed);
        let style = if urgent {
            button::primary
        } else {
            button::text
        };
        Some(
            button(text(label).size(12))
                .on_press(Msg::ToggleCollapsed)
                .style(style)
                .padding([0, 6])
                .into(),
        )
    }

    /// Card flutuante no canto inferior direito (acima da barra de status).
    pub fn overlay_view(&self, enabled: bool) -> Option<Element<'_, Msg>> {
        if !enabled || self.collapsed {
            return None;
        }
        let (mood, bubble, target) = self.present(false);
        let (handle, pixelated) = self.current_handle(mood);

        let sprite = img_widget(handle)
            .width(56)
            .height(56)
            .filter_method(if pixelated {
                FilterMethod::Nearest
            } else {
                FilterMethod::Linear
            });

        let sprite_box = mouse_area(container(sprite).padding(4).style(move |theme: &Theme| {
            let p = theme.extended_palette();
            let border_color = match mood {
                Mood::Attention => p.warning.base.color,
                Mood::Error | Mood::Annoyed => p.danger.base.color,
                Mood::Celebrating => p.success.base.color,
                Mood::Working | Mood::Thinking | Mood::Curious => p.primary.base.color,
                Mood::Idle | Mood::Sleeping => p.background.strong.color,
            };
            container::Style {
                background: Some(p.background.base.color.into()),
                border: iced::Border {
                    color: border_color,
                    width: 1.5,
                    radius: 8.0.into(),
                },
                ..container::Style::default()
            }
        }))
        .on_press(Msg::Poke)
        .interaction(iced::mouse::Interaction::Pointer);

        let mood_color = move |theme: &Theme| -> text::Style {
            let p = theme.extended_palette();
            let color = match mood {
                Mood::Attention => p.warning.base.color,
                Mood::Error | Mood::Annoyed => p.danger.base.color,
                Mood::Celebrating => p.success.base.color,
                Mood::Working | Mood::Thinking | Mood::Curious => p.primary.base.color,
                Mood::Idle | Mood::Sleeping => p.background.base.text,
            };
            text::Style { color: Some(color) }
        };

        let header = row![
            text(&self.pack.name).size(12),
            text("·").size(12).style(text::secondary),
            text(mood.label()).size(12).style(mood_color),
            space::horizontal(),
            button(text("–").size(12))
                .on_press(Msg::ToggleCollapsed)
                .style(button::text)
                .padding([0, 5]),
        ]
        .spacing(4)
        .align_y(iced::Center);

        let mut right_col = column![header].spacing(4).width(Fill);

        if let Some(msg) = bubble {
            let shown = truncate_chars(&msg, 90);
            if target != Target::None && !matches!(mood, Mood::Curious | Mood::Annoyed) {
                let action_label = match target {
                    Target::Interfaces => fl!("mascot-open-interfaces"),
                    Target::Terminal => fl!("mascot-open-terminal"),
                    Target::Problems => fl!("mascot-open-problems"),
                    Target::None => String::new(),
                };
                let bubble_btn = button(
                    column![
                        text(shown).size(12),
                        text(action_label).size(11).style(|t: &Theme| text::Style {
                            color: Some(t.extended_palette().primary.base.color),
                        }),
                    ]
                    .spacing(2),
                )
                .on_press(Msg::ActivateTarget)
                .style(button::secondary)
                .padding([4, 8])
                .width(Fill);
                right_col = right_col.push(bubble_btn);
            } else {
                let poke_text = mouse_area(text(shown).size(12).style(text::secondary))
                    .on_press(Msg::Poke)
                    .interaction(iced::mouse::Interaction::Pointer);
                right_col = right_col.push(poke_text);
            }
        } else {
            let poke_hint = mouse_area(
                text(fl!("mascot-idle-hint"))
                    .size(11)
                    .style(text::secondary),
            )
            .on_press(Msg::Poke)
            .interaction(iced::mouse::Interaction::Pointer);
            right_col = right_col.push(poke_hint);
        }

        let card_width = 270.0;
        let card = mouse_area(
            container(
                row![sprite_box, right_col]
                    .spacing(10)
                    .align_y(iced::Center),
            )
            .width(card_width)
            .padding(8)
            .style(move |theme: &Theme| {
                let p = theme.extended_palette();
                let accent = match mood {
                    Mood::Attention => p.warning.base.color,
                    Mood::Error | Mood::Annoyed => p.danger.base.color,
                    Mood::Curious => p.primary.base.color,
                    _ => p.background.strong.color,
                };
                container::Style {
                    background: Some(p.background.weak.color.into()),
                    border: iced::Border {
                        color: accent,
                        width: if matches!(
                            mood,
                            Mood::Attention | Mood::Error | Mood::Annoyed | Mood::Curious
                        ) {
                            1.5
                        } else {
                            1.0
                        },
                        radius: 10.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 14.0,
                    },
                    ..container::Style::default()
                }
            }),
        )
        .on_enter(Msg::PointerEntered)
        .on_move(Msg::PointerMoved)
        .on_exit(Msg::PointerExited);

        Some(
            container(opaque(card))
                .width(Fill)
                .height(Fill)
                .align_x(iced::Right)
                .align_y(iced::Bottom)
                .padding(iced::Padding {
                    right: 14.0,
                    bottom: 32.0,
                    ..iced::Padding::ZERO
                })
                .into(),
        )
    }
}

fn picker_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_nanos() as u64)
}

/// Hora local (0–23). Sem fuso conhecido, fica a UTC: o cumprimento ainda muda ao longo do dia.
fn local_hour() -> u32 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    #[cfg(unix)]
    {
        #[allow(clippy::useless_conversion)]
        let raw = secs as libc::time_t;
        // SAFETY: `localtime_r` só escreve no `tm` que passamos; os dois ponteiros valem durante a chamada.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if !unsafe { libc::localtime_r(&raw, &mut tm) }.is_null() {
            return tm.tm_hour.clamp(0, 23) as u32;
        }
    }
    ((secs % 86_400) / 3600) as u32
}

fn format_external(source: Option<&str>, text: Option<&str>) -> Option<String> {
    match (
        source.filter(|s| !s.is_empty()),
        text.filter(|t| !t.is_empty()),
    ) {
        (Some(s), Some(t)) => Some(format!("{s}: {t}")),
        (None, Some(t)) => Some(t.to_string()),
        (Some(s), None) => Some(s.to_string()),
        (None, None) => None,
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let head: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

// ---- Observador de `.idetea/` em thread separada --------------------------

#[allow(clippy::ptr_arg)]
pub fn watch(
    spec: &(PathBuf, String),
) -> iced::futures::channel::mpsc::UnboundedReceiver<Arc<DiskSnapshot>> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded();
    let (root, configured_pack) = spec.clone();
    std::thread::spawn(move || {
        let mut prev = DiskSnapshot::default();
        let mut prev_ui_unanswered: BTreeMap<String, String> = BTreeMap::new();
        let mut prev_agents: BTreeMap<String, (String, String)> = BTreeMap::new();
        let mut last_pack_sig = String::new();
        let mut first = true;

        loop {
            if tx.is_closed() {
                return;
            }
            let (mut snap, ui_map, agents_map) =
                scan_project(&root, &prev_ui_unanswered, &prev_agents, first);
            prev_ui_unanswered = ui_map;
            prev_agents = agents_map;
            first = false;

            let (pack_sig, pack_dir) = resolve_pack_dir(&root, &configured_pack);
            if pack_sig != last_pack_sig {
                last_pack_sig = pack_sig.clone();
                snap.custom_pack = Some(Arc::new(match pack_dir {
                    Some(dir) => load_custom_pack(&dir, &pack_sig).unwrap_or_else(|e| {
                        eprintln!(
                            "idetea: pacote de mascote inválido em {}: {e}",
                            dir.display()
                        );
                        empty_pack()
                    }),
                    None => empty_pack(),
                }));
            }

            if snap != prev || snap.event.is_some() || snap.custom_pack.is_some() {
                let to_send = snap.clone();
                snap.event = None;
                snap.custom_pack = None;
                prev = snap;
                if tx.unbounded_send(Arc::new(to_send)).is_err() {
                    return;
                }
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    });
    rx
}

fn empty_pack() -> LoadedPack {
    LoadedPack {
        name: String::new(),
        pixelated: true,
        states: HashMap::new(),
        source_id: String::new(),
    }
}

type ScanResult = (
    DiskSnapshot,
    BTreeMap<String, String>,
    BTreeMap<String, (String, String)>,
);

pub fn scan_project(
    root: &Path,
    prev_ui: &BTreeMap<String, String>,
    prev_agents: &BTreeMap<String, (String, String)>,
    first: bool,
) -> ScanResult {
    let mut snap = DiskSnapshot::default();
    let mut current_ui = BTreeMap::new();
    let mut current_agents = BTreeMap::new();

    // 1. Interfaces em `.idetea/ui/`
    let ui_dir = root.join(".idetea").join("ui");
    if let Ok(entries) = std::fs::read_dir(&ui_dir) {
        let mut names: Vec<(String, PathBuf)> = entries
            .flatten()
            .filter_map(|e| {
                let file = e.file_name().to_str()?.to_string();
                if file.starts_with('.')
                    || !file.ends_with(".json")
                    || file == "idetea-ui.schema.json"
                {
                    return None;
                }
                let name = file.strip_suffix(".json")?.to_string();
                Some((name, e.path()))
            })
            .collect();
        names.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, path) in names {
            if has_response(&ui_dir, &name) {
                continue;
            }
            let (title, source, has_buttons) = read_ui_summary(&path, &name);
            if !has_buttons {
                continue;
            }
            snap.ui_open_count += 1;
            current_ui.insert(name.clone(), title.clone());
            let wait_path = ui_dir.join(format!("{name}.wait"));
            let wait_fresh = is_fresh(&wait_path, WAIT_FRESH) || is_fresh(&path, WAIT_FRESH);
            let mtime = file_mtime_ms(&path);
            if snap.ui_waiting.is_none() || (wait_fresh && !snap.ui_waiting.as_ref().unwrap().3) {
                snap.ui_waiting = Some((name, title, source, wait_fresh, mtime));
            }
        }
    }

    if !first {
        for (old_name, old_title) in prev_ui {
            if !current_ui.contains_key(old_name) {
                snap.event = Some((
                    Mood::Celebrating,
                    lines::format_once(lines::ANSWERED, &[("title", old_title.as_str())]),
                    Target::None,
                ));
                break;
            }
        }
    }

    // 2. Terminais de agentes em `.idetea/agents/`
    let agents_dir = root.join(".idetea").join("agents");
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        let mut status_files: Vec<(String, PathBuf)> = entries
            .flatten()
            .filter_map(|e| {
                let file = e.file_name().to_str()?.to_string();
                let name = file.strip_suffix(".status.json")?.to_string();
                if name.starts_with('.') {
                    return None;
                }
                Some((name, e.path()))
            })
            .collect();
        status_files.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, status_path) in status_files {
            let Ok(text) = std::fs::read_to_string(&status_path) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
                continue;
            };
            let state = v["state"].as_str().unwrap_or_default().to_string();
            let code = v["code"].as_i64().unwrap_or(0);
            let req_path = agents_dir.join(format!("{name}.json"));
            let title = read_agent_title(&req_path, &name);
            current_agents.insert(name.clone(), (state.clone(), title.clone()));

            match state.as_str() {
                "pending" if snap.agent_pending.is_none() => {
                    snap.agent_pending =
                        Some((name.clone(), title.clone(), file_mtime_ms(&status_path)));
                }
                "running" if snap.agent_running.is_none() => {
                    snap.agent_running = Some((name.clone(), title.clone()));
                }
                "exited" if !first => {
                    if let Some((prev_state, _)) = prev_agents.get(&name)
                        && prev_state != "exited"
                    {
                        let code_s = code.to_string();
                        if code == 0 {
                            snap.event = Some((
                                Mood::Celebrating,
                                lines::format_once(lines::TERM_OK, &[("title", title.as_str())]),
                                Target::None,
                            ));
                        } else {
                            snap.event = Some((
                                Mood::Error,
                                lines::format_once(
                                    lines::TERM_ERR,
                                    &[("title", title.as_str()), ("code", code_s.as_str())],
                                ),
                                Target::Terminal,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // 3. Estado explícito em `.idetea/mascot.json`
    if let Some(ext) = read_external_state(root)
        && ext.is_active(now_ms())
        && let Some(mood) = Mood::parse(&ext.state)
    {
        snap.external = Some((mood, ext.text, ext.source, ext.timestamp_ms));
    }

    (snap, current_ui, current_agents)
}

fn has_response(ui_dir: &Path, name: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(ui_dir.join("responses").join(name)) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.file_name()
            .to_str()
            .is_some_and(|f| f.ends_with(".json") && !f.starts_with('.'))
    })
}

fn read_ui_summary(path: &Path, fallback: &str) -> (String, Option<String>, bool) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return (fallback.to_string(), None, false);
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return (fallback.to_string(), None, false);
    };
    let title = v["title"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback)
        .to_string();
    let source = v["source"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(String::from);
    let has_buttons = v["components"].as_array().is_some_and(|arr| {
        arr.iter()
            .any(|c| c["component"].as_str() == Some("button"))
    });
    (title, source, has_buttons)
}

fn read_agent_title(path: &Path, fallback: &str) -> String {
    let Ok(text) = std::fs::read_to_string(path) else {
        return fallback.to_string();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return fallback.to_string();
    };
    v["title"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn is_fresh(path: &Path, max_age: Duration) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age <= max_age)
}

fn file_mtime_ms(path: &Path) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn mascot_state_path(root: &Path) -> PathBuf {
    root.join(".idetea").join("mascot.json")
}

pub fn read_external_state(root: &Path) -> Option<ExternalState> {
    let text = std::fs::read_to_string(mascot_state_path(root)).ok()?;
    serde_json::from_str::<ExternalState>(&text).ok()
}

// ---- Pacotes de mascote (`mascot.toml` + xícara embutida) ------------------

#[derive(Debug, Deserialize)]
#[serde(default)]
struct PackManifest {
    name: String,
    pixelated: bool,
    frame_width: u32,
    frame_height: u32,
    states: BTreeMap<String, StateSpec>,
}

impl Default for PackManifest {
    fn default() -> Self {
        Self {
            name: "Tea".into(),
            pixelated: true,
            frame_width: 32,
            frame_height: 32,
            states: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct StateSpec {
    path: String,
    row: u32,
    frames: u32,
    fps: u32,
    frame_width: Option<u32>,
    frame_height: Option<u32>,
}

impl Default for StateSpec {
    fn default() -> Self {
        Self {
            path: String::new(),
            row: 0,
            frames: 1,
            fps: 4,
            frame_width: None,
            frame_height: None,
        }
    }
}

fn resolve_pack_dir(root: &Path, configured: &str) -> (String, Option<PathBuf>) {
    let mut candidates = Vec::new();
    if !configured.trim().is_empty() {
        let p = PathBuf::from(configured.trim());
        candidates.push(if p.is_absolute() { p } else { root.join(p) });
    }
    candidates.push(root.join(".idetea").join("mascot"));
    if let Some(cfg) =
        tea_core::config::global_config_path().and_then(|p| p.parent().map(Path::to_path_buf))
    {
        candidates.push(cfg.join("mascot"));
    }
    for dir in candidates {
        let manifest = dir.join("mascot.toml");
        if let Ok(meta) = std::fs::metadata(&manifest) {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            return (
                format!("{}:{}:{mtime}", dir.display(), meta.len()),
                Some(dir),
            );
        }
    }
    (String::new(), None)
}

pub fn load_custom_pack(dir: &Path, source_id: &str) -> Result<LoadedPack, String> {
    let manifest_path = dir.join("mascot.toml");
    let text = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: PackManifest = toml::from_str(&text).map_err(|e| e.to_string())?;
    let builtin = builtin_teacup_pack();
    let mut states = builtin.states;

    for (key, spec) in &manifest.states {
        let Some(mood) = Mood::parse(key) else {
            return Err(format!("estado desconhecido em mascot.toml: {key}"));
        };
        if spec.path.is_empty() || spec.path.contains("..") {
            return Err(format!("caminho inválido para {key}: {}", spec.path));
        }
        let img_path = dir.join(&spec.path);
        let bytes = std::fs::read(&img_path).map_err(|e| format!("{}: {e}", spec.path))?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(format!("{}: imagem maior que 8 MB", spec.path));
        }
        let reader = image::ImageReader::new(Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|e| e.to_string())?;
        let (w, h) = reader.into_dimensions().map_err(|e| e.to_string())?;
        if w == 0 || h == 0 || w > 4096 || h > 4096 {
            return Err(format!("{}: dimensões inválidas ({w}x{h})", spec.path));
        }
        let rgba = image::load_from_memory(&bytes)
            .map_err(|e| e.to_string())?
            .to_rgba8();
        let fw = spec.frame_width.unwrap_or(manifest.frame_width).max(1);
        let fh = spec.frame_height.unwrap_or(manifest.frame_height).max(1);
        let count = spec.frames.clamp(1, 64);
        let y0 = spec.row.saturating_mul(fh);
        if fw.saturating_mul(count) > w || y0.saturating_add(fh) > h {
            return Err(format!("{}: grade fora da imagem ({w}x{h})", spec.path));
        }
        let mut handles = Vec::with_capacity(count as usize);
        for i in 0..count {
            let x0 = i * fw;
            let mut buf = Vec::with_capacity((fw * fh * 4) as usize);
            for y in y0..(y0 + fh) {
                for x in x0..(x0 + fw) {
                    buf.extend_from_slice(&rgba.get_pixel(x, y).0);
                }
            }
            handles.push(Handle::from_rgba(fw, fh, buf));
        }
        states.insert(mood, (handles, spec.fps.clamp(1, 24)));
    }

    Ok(LoadedPack {
        name: if manifest.name.trim().is_empty() {
            "Tea".into()
        } else {
            manifest.name
        },
        pixelated: manifest.pixelated,
        states,
        source_id: source_id.to_string(),
    })
}

/// Gera a xícara de chá em pixel art embutida (24×24 px, 4 quadros por estado).
pub fn builtin_teacup_pack() -> LoadedPack {
    let mut states = HashMap::new();
    for mood in Mood::ALL {
        let frames: Vec<Handle> = (0..4).map(|f| render_teacup_frame(mood, f)).collect();
        states.insert(mood, (frames, mood.default_fps()));
    }
    LoadedPack {
        name: "Tea".into(),
        pixelated: true,
        states,
        source_id: "builtin".into(),
    }
}

fn render_teacup_frame(mood: Mood, frame: usize) -> Handle {
    const W: u32 = 24;
    const H: u32 = 24;
    let mut px = vec![[0u8, 0, 0, 0]; (W * H) as usize];
    let mut put = |x: i32, y: i32, c: [u8; 4]| {
        if (0..W as i32).contains(&x) && (0..H as i32).contains(&y) {
            px[(y as u32 * W + x as u32) as usize] = c;
        }
    };

    // Deslocamento da xícara conforme o estado e o quadro (pulo ou tremor).
    let (dx, dy) = match mood {
        Mood::Attention | Mood::Celebrating => (0, if frame % 2 == 1 { -1 } else { 0 }),
        Mood::Working => (0, if frame == 1 || frame == 3 { -1 } else { 0 }),
        Mood::Error => (if frame.is_multiple_of(2) { -1 } else { 1 }, 0),
        Mood::Curious => (
            match frame {
                1 => -1,
                3 => 1,
                _ => 0,
            },
            if frame == 2 { -1 } else { 0 },
        ),
        Mood::Annoyed => (
            if frame.is_multiple_of(2) { -1 } else { 1 },
            if frame == 1 { -1 } else { 0 },
        ),
        _ => (0, 0),
    };

    // Paleta da xícara de chá.
    let outline = [38, 43, 68, 255];
    let cup_body = [235, 243, 249, 255];
    let cup_shade = [194, 211, 227, 255];
    let tea_green = match mood {
        Mood::Error => [230, 85, 85, 255],
        Mood::Annoyed => [245, 100, 55, 255],
        Mood::Attention => [240, 180, 65, 255],
        Mood::Curious => [90, 210, 175, 255],
        _ => [98, 195, 112, 255],
    };
    let saucer = [173, 193, 212, 255];
    let cheek = match mood {
        Mood::Annoyed => [245, 75, 90, 245],
        _ => [244, 143, 160, 220],
    };

    // Pires na base (y = 20..21).
    for x in 4..=18 {
        put(x, 20, saucer);
        put(x, 21, outline);
    }
    put(3, 20, outline);
    put(19, 20, outline);

    // Corpo da xícara (x = 5..17, y = 10..19).
    for y in 10..=19 {
        let inset = if y >= 18 {
            2
        } else if y >= 16 {
            1
        } else {
            0
        };
        let x0 = 5 + inset;
        let x1 = 17 - inset;
        for x in x0..=x1 {
            let color = if x == x0 || x == x1 || y == 10 || y == 19 {
                outline
            } else if y == 11 {
                tea_green
            } else if x >= x1 - 2 {
                cup_shade
            } else {
                cup_body
            };
            put(x + dx, y + dy, color);
        }
    }

    // Asa da xícara à direita (x = 18..20, y = 12..16).
    put(18 + dx, 12 + dy, outline);
    put(19 + dx, 12 + dy, outline);
    put(20 + dx, 13 + dy, outline);
    put(20 + dx, 14 + dy, outline);
    put(20 + dx, 15 + dy, outline);
    put(18 + dx, 16 + dy, outline);
    put(19 + dx, 16 + dy, outline);

    // Bochechas.
    put(7 + dx, 16 + dy, cheek);
    put(15 + dx, 16 + dy, cheek);

    // Olhos e boca conforme o estado.
    match mood {
        Mood::Idle => {
            if frame == 3 {
                // Piscando.
                put(8 + dx, 15 + dy, outline);
                put(9 + dx, 15 + dy, outline);
                put(13 + dx, 15 + dy, outline);
                put(14 + dx, 15 + dy, outline);
            } else {
                put(8 + dx, 14 + dy, outline);
                put(8 + dx, 15 + dy, outline);
                put(14 + dx, 14 + dy, outline);
                put(14 + dx, 15 + dy, outline);
            }
            put(10 + dx, 16 + dy, outline);
            put(11 + dx, 17 + dy, outline);
            put(12 + dx, 16 + dy, outline);
        }
        Mood::Working => {
            // Olhar focado + boca determinada.
            put(8 + dx, 14 + dy, outline);
            put(9 + dx, 15 + dy, outline);
            put(14 + dx, 14 + dy, outline);
            put(13 + dx, 15 + dy, outline);
            put(10 + dx, 17 + dy, outline);
            put(11 + dx, 17 + dy, outline);
            put(12 + dx, 17 + dy, outline);
        }
        Mood::Thinking => {
            // Olhando para cima.
            put(9 + dx, 13 + dy, outline);
            put(15 + dx, 13 + dy, outline);
            put(11 + dx, 17 + dy, outline);
            put(12 + dx, 17 + dy, outline);
        }
        Mood::Attention => {
            // Olhos atentos + boca aberta "o".
            put(8 + dx, 14 + dy, outline);
            put(8 + dx, 15 + dy, outline);
            put(14 + dx, 14 + dy, outline);
            put(14 + dx, 15 + dy, outline);
            put(11 + dx, 16 + dy, outline);
            put(11 + dx, 17 + dy, outline);
        }
        Mood::Celebrating => {
            // Olhos felizes "^ ^".
            put(8 + dx, 15 + dy, outline);
            put(9 + dx, 14 + dy, outline);
            put(13 + dx, 14 + dy, outline);
            put(14 + dx, 15 + dy, outline);
            put(10 + dx, 16 + dy, outline);
            put(11 + dx, 17 + dy, outline);
            put(12 + dx, 16 + dy, outline);
        }
        Mood::Error => {
            // Olhos "x x".
            put(8 + dx, 14 + dy, outline);
            put(9 + dx, 15 + dy, outline);
            put(9 + dx, 14 + dy, outline);
            put(8 + dx, 15 + dy, outline);
            put(13 + dx, 14 + dy, outline);
            put(14 + dx, 15 + dy, outline);
            put(14 + dx, 14 + dy, outline);
            put(13 + dx, 15 + dy, outline);
            put(10 + dx, 17 + dy, outline);
            put(11 + dx, 16 + dy, outline);
            put(12 + dx, 17 + dy, outline);
        }
        Mood::Sleeping => {
            // Olhos fechados "- -".
            put(8 + dx, 15 + dy, outline);
            put(9 + dx, 15 + dy, outline);
            put(13 + dx, 15 + dy, outline);
            put(14 + dx, 15 + dy, outline);
            put(11 + dx, 17 + dy, outline);
        }
        Mood::Curious => {
            // Sobrancelha direita erguida + olho direito arregalado + boca "o" curiosa.
            put(8 + dx, 14 + dy, outline);
            put(8 + dx, 15 + dy, outline);
            put(13 + dx, 12 + dy, outline);
            put(14 + dx, 12 + dy, outline);
            put(13 + dx, 14 + dy, outline);
            put(14 + dx, 14 + dy, outline);
            put(13 + dx, 15 + dy, outline);
            put(14 + dx, 15 + dy, outline);
            put(11 + dx, 16 + dy, outline);
            put(12 + dx, 16 + dy, outline);
            put(11 + dx, 17 + dy, outline);
            put(12 + dx, 17 + dy, outline);
        }
        Mood::Annoyed => {
            // Sobrancelhas franzidas "\ /" + olhos cerrados + boca zangada.
            put(7 + dx, 13 + dy, outline);
            put(8 + dx, 14 + dy, outline);
            put(9 + dx, 14 + dy, outline);
            put(8 + dx, 15 + dy, outline);
            put(9 + dx, 15 + dy, outline);
            put(15 + dx, 13 + dy, outline);
            put(14 + dx, 14 + dy, outline);
            put(13 + dx, 14 + dy, outline);
            put(13 + dx, 15 + dy, outline);
            put(14 + dx, 15 + dy, outline);
            put(9 + dx, 17 + dy, outline);
            put(10 + dx, 16 + dy, outline);
            put(11 + dx, 17 + dy, outline);
            put(12 + dx, 16 + dy, outline);
            put(13 + dx, 17 + dy, outline);
        }
    }

    // Efeito acima da xícara (y = 2..8).
    let shift = (frame as i32) % 4;
    match mood {
        Mood::Idle => {
            let steam = [200, 220, 235, 165];
            put(9 + (shift % 2), 8 - shift, steam);
            put(13 - (shift % 2), 7 - (shift % 3), steam);
        }
        Mood::Working => {
            let active = [110, 215, 255, 240];
            put(8 + (shift % 2), 7 - (shift % 3), active);
            put(11, 6 - (shift % 3), active);
            put(14 - (shift % 2), 7 - ((shift + 1) % 3), active);
        }
        Mood::Thinking => {
            let bubble = [180, 200, 255, 230];
            put(14, 7, bubble);
            if frame >= 1 {
                put(16, 5, bubble);
            }
            if frame >= 2 {
                put(18, 3, bubble);
                put(19, 3, bubble);
                put(18, 4, bubble);
                put(19, 4, bubble);
            }
        }
        Mood::Attention => {
            // Ponto de exclamação amarelo/laranja.
            let bang = [255, 196, 45, 255];
            let by = if frame.is_multiple_of(2) { 0 } else { -1 };
            for y in 2..=5 {
                put(11, y + by, bang);
                put(12, y + by, bang);
            }
            put(11, 7 + by, bang);
            put(12, 7 + by, bang);
        }
        Mood::Celebrating => {
            let star = [255, 215, 70, 255];
            let sy = if frame.is_multiple_of(2) { 0 } else { -1 };
            put(6, 5 + sy, star);
            put(11, 3 - sy, star);
            put(16, 5 + sy, star);
            put(11, 2 - sy, star);
        }
        Mood::Error => {
            let warn = [255, 90, 90, 255];
            for y in 2..=5 {
                put(11, y, warn);
            }
            put(11, 7, warn);
            put(7, 5 + (shift % 2), warn);
            put(15, 5 + ((shift + 1) % 2), warn);
        }
        Mood::Sleeping => {
            let zzz = [150, 185, 245, 220];
            let zy = 7 - (shift % 3);
            put(14, zy, zzz);
            put(15, zy, zzz);
            put(14, zy + 1, zzz);
            put(15, zy + 1, zzz);
        }
        Mood::Curious => {
            // Ponto de interrogação "?" flutuante.
            let qmark = [110, 220, 255, 255];
            let qy = if frame.is_multiple_of(2) { 0 } else { -1 };
            put(10, 3 + qy, qmark);
            put(11, 2 + qy, qmark);
            put(12, 2 + qy, qmark);
            put(13, 3 + qy, qmark);
            put(12, 4 + qy, qmark);
            put(11, 5 + qy, qmark);
            put(11, 7 + qy, qmark);
        }
        Mood::Annoyed => {
            // Vapor fervendo vermelho + símbolo de irritação no canto superior direito.
            let rage = [255, 75, 75, 255];
            let hot_steam = [255, 145, 80, 235];
            put(8, 7 - (shift % 3), hot_steam);
            put(10, 6 - ((shift + 1) % 3), hot_steam);
            let ry = if frame.is_multiple_of(2) { 0 } else { -1 };
            put(14, 3 + ry, rage);
            put(16, 3 + ry, rage);
            put(13, 4 + ry, rage);
            put(17, 4 + ry, rage);
            put(14, 6 + ry, rage);
            put(16, 6 + ry, rage);
            put(13, 5 + ry, rage);
            put(17, 5 + ry, rage);
        }
    }

    let mut rgba = Vec::with_capacity((W * H * 4) as usize);
    for c in px {
        rgba.extend_from_slice(&c);
    }
    Handle::from_rgba(W, H, rgba)
}

// ---- Linha de comando (`idetea mascot …`) ---------------------------------

pub fn run_cli(args: &[String], cwd: &Path) -> i32 {
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{}", fl!("mascot-usage"));
        return EXIT_OK;
    }

    let root = project_root(cwd);
    let sub = args[0].as_str();

    if sub == "clear" || sub == "off" || sub == "reset" {
        let _ = std::fs::remove_file(mascot_state_path(&root));
        return EXIT_OK;
    }

    if sub == "status" {
        let as_json = args[1..].iter().any(|a| a == "--json");
        let now = now_ms();
        match read_external_state(&root).filter(|s| s.is_active(now)) {
            Some(s) => {
                if as_json {
                    println!("{}", serde_json::to_string_pretty(&s).unwrap_or_default());
                } else {
                    match (&s.source, &s.text) {
                        (Some(src), Some(txt)) => println!("{}: {src}: {txt}", s.state),
                        (None, Some(txt)) => println!("{}: {txt}", s.state),
                        _ => println!("{}", s.state),
                    }
                }
            }
            None => {
                if as_json {
                    println!("{}", serde_json::json!({"state": "auto"}));
                } else {
                    println!("auto");
                }
            }
        }
        return EXIT_OK;
    }

    let Some(mood) = Mood::parse(sub) else {
        eprintln!("{}", fl!("mascot-bad-state", state = sub.to_string()));
        eprintln!("{}", fl!("mascot-usage"));
        return EXIT_INVALID;
    };

    let mut text = None;
    let mut source = None;
    let mut ttl_secs = DEFAULT_TTL_SECS;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--text" | "-t" | "--bubble" => {
                let Some(val) = args.get(i + 1) else {
                    eprintln!("{}", fl!("mascot-usage"));
                    return EXIT_INVALID;
                };
                text = Some(val.clone());
                i += 2;
            }
            "--source" | "-s" => {
                let Some(val) = args.get(i + 1) else {
                    eprintln!("{}", fl!("mascot-usage"));
                    return EXIT_INVALID;
                };
                source = Some(val.clone());
                i += 2;
            }
            "--ttl" => {
                let Some(val) = args
                    .get(i + 1)
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|n| *n > 0)
                else {
                    eprintln!("{}", fl!("mascot-usage"));
                    return EXIT_INVALID;
                };
                ttl_secs = val.min(3600);
                i += 2;
            }
            other => {
                eprintln!("{}", fl!("status-bad-arg", arg = other.to_string()));
                eprintln!("{}", fl!("mascot-usage"));
                return EXIT_INVALID;
            }
        }
    }

    let dir = root.join(".idetea");
    if std::fs::create_dir_all(&dir).is_err() {
        return EXIT_INVALID;
    }
    ignore_in_idetea(&root, "mascot.json");
    let state = ExternalState {
        format: FORMAT.into(),
        version: 1,
        state: mood.as_str().into(),
        text,
        source,
        timestamp_ms: now_ms(),
        ttl_secs,
    };
    let Ok(json) = serde_json::to_string_pretty(&state) else {
        return EXIT_INVALID;
    };
    let tmp = dir.join(".mascot.json.tmp");
    let path = mascot_state_path(&root);
    if std::fs::write(&tmp, json + "\n").is_err() || std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return EXIT_INVALID;
    }
    EXIT_OK
}

fn project_root(cwd: &Path) -> PathBuf {
    let cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    cwd.ancestors()
        .find(|d| d.join(".idetea").is_dir() || d.join(".git").exists())
        .unwrap_or(&cwd)
        .to_path_buf()
}

fn ignore_in_idetea(root: &Path, name: &str) {
    let path = root.join(".idetea").join(".gitignore");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| "/.gitignore\n".into());
    if text
        .lines()
        .any(|l| l.trim() == name || l.trim() == format!("/{name}"))
    {
        return;
    }
    let sep = if text.is_empty() || text.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    let _ = std::fs::write(&path, format!("{text}{sep}/{name}\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tea-mascot-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".idetea")).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn builtin_pack_has_four_frames_for_every_mood() {
        let pack = builtin_teacup_pack();
        assert_eq!(pack.name, "Tea");
        assert!(pack.pixelated);
        for mood in Mood::ALL {
            let (frames, fps) = pack.states.get(&mood).expect("estado presente");
            assert_eq!(frames.len(), 4);
            assert!(fps >= &2);
        }
    }

    #[test]
    fn cli_writes_reads_and_clears_state() {
        tea_core::i18n::init("pt-BR");
        let root = temp_project("cli");
        let args = |s: &[&str]| s.iter().map(|x| x.to_string()).collect::<Vec<_>>();

        assert_eq!(
            run_cli(
                &args(&[
                    "working",
                    "--text",
                    "Compilando…",
                    "--source",
                    "Claude",
                    "--ttl",
                    "60"
                ]),
                &root
            ),
            EXIT_OK
        );
        let ext = read_external_state(&root).expect("arquivo gravado");
        assert_eq!(ext.state, "working");
        assert_eq!(ext.text.as_deref(), Some("Compilando…"));
        assert_eq!(ext.source.as_deref(), Some("Claude"));
        assert_eq!(ext.ttl_secs, 60);
        assert!(ext.is_active(now_ms()));

        assert_eq!(run_cli(&args(&["clear"]), &root), EXIT_OK);
        assert!(read_external_state(&root).is_none());

        assert_eq!(run_cli(&args(&["estado-invalido"]), &root), EXIT_INVALID);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn priority_orders_ui_question_and_explicit_cli_by_recency() {
        tea_core::i18n::init("pt-BR");
        let root = temp_project("prio");
        let ui_dir = root.join(".idetea/ui");
        std::fs::create_dir_all(&ui_dir).unwrap();

        // 1. Um comando `idetea mascot working` é enviado antes da pergunta.
        run_cli(
            &["working".into(), "--text".into(), "Rodando…".into()],
            &root,
        );
        std::thread::sleep(Duration::from_millis(10));

        // 2. Depois chega uma pergunta aberta com `.wait`: ela é mais recente e assume `Attention`.
        std::fs::write(
            ui_dir.join("pergunta.json"),
            r#"{"format":"idetea-ui","version":1,"revision":1,"title":"Aprovar?","source":"Claude","components":[{"id":"b","component":"button","label":"Sim","action":{"name":"ok"}}]}"#,
        )
        .unwrap();
        std::fs::write(ui_dir.join("pergunta.wait"), "").unwrap();

        let (snap1, ui1, agents1) = scan_project(&root, &BTreeMap::new(), &BTreeMap::new(), true);
        let mut mascot = Mascot::new(false);
        mascot.apply_snapshot(snap1);

        let (mood, bubble, target) = mascot.resolved();
        assert_eq!(mood, Mood::Attention);
        assert_eq!(bubble.as_deref(), Some("Claude: Aprovar?"));
        assert_eq!(target, Target::Interfaces);

        // 3. Com a pergunta ainda aberta, rodar `idetea mascot error` (mais novo) muda o mascote na hora!
        std::thread::sleep(Duration::from_millis(10));
        run_cli(
            &["error".into(), "--text".into(), "Teste de alerta".into()],
            &root,
        );
        let (snap_cli, _, _) = scan_project(&root, &ui1, &agents1, false);
        mascot.apply_snapshot(snap_cli);
        let (mood_cli, bubble_cli, target_cli) = mascot.resolved();
        assert_eq!(mood_cli, Mood::Error);
        assert_eq!(bubble_cli.as_deref(), Some("Teste de alerta"));
        assert_eq!(target_cli, Target::Interfaces);

        // 4. `idetea mascot clear` limpa o override e quando a pergunta some, comemora (`Celebrating`).
        std::fs::remove_file(ui_dir.join("pergunta.json")).unwrap();
        run_cli(&["clear".into()], &root);
        let (snap2, _, _) = scan_project(&root, &ui1, &agents1, false);
        mascot.apply_snapshot(snap2);
        let (mood2, _, _) = mascot.resolved();
        assert_eq!(mood2, Mood::Celebrating);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn loads_custom_mascot_toml_and_slices_spritesheet() {
        let root = temp_project("pack");
        let pack_dir = root.join(".idetea/mascot");
        std::fs::create_dir_all(&pack_dir).unwrap();

        // Cria um PNG 16×8 (2 quadros de 8×8).
        let img = image::RgbaImage::from_fn(16, 8, |x, _y| {
            if x < 8 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 255, 0, 255])
            }
        });
        img.save(pack_dir.join("robo.png")).unwrap();
        std::fs::write(
            pack_dir.join("mascot.toml"),
            r#"
name = "RoboTea"
pixelated = true
frame_width = 8
frame_height = 8

[states.idle]
path = "robo.png"
row = 0
frames = 2
fps = 5
"#,
        )
        .unwrap();

        let loaded = load_custom_pack(&pack_dir, "sig1").expect("pacote válido");
        assert_eq!(loaded.name, "RoboTea");
        let (idle_frames, idle_fps) = loaded.states.get(&Mood::Idle).unwrap();
        assert_eq!(idle_frames.len(), 2);
        assert_eq!(*idle_fps, 5);
        // Estados não listados no TOML herdam os quadros da xícara embutida.
        assert_eq!(loaded.states.get(&Mood::Working).unwrap().0.len(), 4);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn hover_and_poke_trigger_curious_then_annoyed() {
        tea_core::i18n::init("pt-BR");
        let mut mascot = Mascot::new(false);

        // Passar o mouse sobre o mascote deixa ele Curioso.
        mascot.on_pointer_entered();
        assert_eq!(mascot.resolved().0, Mood::Curious);

        // Sair com o mouse volta para Idle.
        mascot.on_pointer_exited();
        assert_eq!(mascot.resolved().0, Mood::Idle);

        // 1º e 2º cliques despertam curiosidade.
        mascot.on_poke();
        assert_eq!(mascot.resolved().0, Mood::Curious);
        mascot.on_poke();
        assert_eq!(mascot.resolved().0, Mood::Curious);

        // A partir do 3º clique seguido, fica Irritado!
        mascot.on_poke();
        assert_eq!(mascot.resolved().0, Mood::Annoyed);

        // Sacudir o mouse sobre o mascote também deixa Irritado.
        let mut mascot2 = Mascot::new(false);
        mascot2.on_pointer_moved(Point::new(0.0, 0.0));
        mascot2.on_pointer_moved(Point::new(150.0, 0.0));
        mascot2.on_pointer_moved(Point::new(0.0, 0.0));
        assert_eq!(mascot2.resolved().0, Mood::Annoyed);
    }

    #[test]
    fn save_sleep_greeting_and_build_respect_chatty() {
        tea_core::i18n::init("pt-BR");
        let mut mascot = Mascot::new(false);
        mascot.notify_saved("main.rs");
        let (mood, text, target) = mascot.present(false);
        assert_eq!(mood, Mood::Celebrating);
        assert!(text.unwrap().contains("main.rs"));
        assert_eq!(
            target,
            Target::None,
            "salvar não abre painel nem rouba o foco"
        );

        mascot.chatty = tea_core::config::MascotChatty::Quiet;
        mascot.aside = None;
        mascot.notify_saved("lib.rs");
        assert!(mascot.aside.is_none(), "quiet não comenta o salvar");

        mascot.notify_build(false);
        assert_eq!(mascot.resolved().0, Mood::Error);
        assert_eq!(mascot.resolved().2, Target::Problems);
        mascot.transient = None;

        mascot.chatty = tea_core::config::MascotChatty::Normal;
        mascot.last_active = Instant::now()
            .checked_sub(Duration::from_secs(301))
            .unwrap();
        let (mood, text, _) = mascot.present(false);
        assert_eq!(mood, Mood::Sleeping);
        assert!(text.is_some(), "adormecer tem uma fala");
        mascot.touch_user_activity();
        assert_eq!(mascot.resolved().0, Mood::Idle);
        assert!(
            mascot.aside.is_some(),
            "acordar fala quando o usuário volta"
        );

        mascot.aside = None;
        mascot.greeted = None;
        mascot.consider_greeting(23);
        let late = mascot.present(false).1.unwrap();
        mascot.consider_greeting(23);
        assert_eq!(
            mascot.present(false).1.as_deref(),
            Some(late.as_str()),
            "o mesmo período não repete o cumprimento"
        );
        mascot.aside = None;
        mascot.consider_greeting(9);
        assert_ne!(mascot.present(false).1.as_deref(), Some(late.as_str()));
    }

    #[test]
    fn every_locale_has_every_mascot_key() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n");
        let base = tea_core::i18n::read_messages(&dir.join("pt-BR/tea-app.ftl")).expect("pt-BR");
        let mascot: Vec<_> = base
            .iter()
            .filter(|(k, _)| k.starts_with("mascot-"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        assert!(
            mascot.len() > 40,
            "o mascote ficou com poucas chaves: {}",
            mascot.len()
        );
        for lang in tea_core::i18n::Language::ALL {
            let path = dir.join(lang.tag()).join("tea-app.ftl");
            let other = tea_core::i18n::read_messages(&path).unwrap_or_else(|e| panic!("{e}"));
            for (id, vars) in &mascot {
                match other.get(id) {
                    None => panic!("{}: falta \"{id}\"", path.display()),
                    Some(v) if v != vars => panic!(
                        "{}: \"{id}\" usa {v:?}, o português usa {vars:?}",
                        path.display()
                    ),
                    _ => {}
                }
            }
        }
        let have: std::collections::BTreeSet<_> = mascot.iter().map(|(k, _)| k.as_str()).collect();
        for pool in lines::all_pools() {
            for key in *pool {
                assert!(have.contains(key), "o código pede {key} e o pt-BR não tem");
            }
        }
    }

    #[test]
    fn tips_use_shortcuts_that_exist_by_default() {
        let cfg = tea_core::config::Config::load(None, None);
        assert!(cfg.errors.is_empty(), "{:?}", cfg.errors);
        let (keymap, errs) = tea_core::keymap::Keymap::from_pairs(
            cfg.keys.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        );
        assert!(errs.is_empty(), "{errs:?}");
        for cmd in lines::TIP_COMMANDS {
            let chord = keymap
                .chord_for(cmd)
                .unwrap_or_else(|| panic!("{cmd} não tem atalho no config padrão"));
            assert!(!chord.to_string().is_empty());
        }
        let loader = tea_core::i18n::load_in(
            "tea-app",
            &crate::Localizations,
            tea_core::i18n::Language::English,
        );
        let marked = tea_core::i18n::get_with_markers(&loader, "mascot-tip-1");
        let text = lines::with_shortcuts(&marked, &keymap);
        assert!(text.contains("Ctrl+Shift+P"), "{text}");
        assert!(!text.contains('$') && !text.contains('{'), "{text}");
    }
}
