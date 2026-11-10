//! Painel "Debug": botões (continuar, pausar, passos, parar), pilha de chamadas, variáveis e a
//! saída do programa e da compilação.
//!
//! Quem fala com o gdb é o núcleo; este painel só recebe [`CoreEvent::Debug`] e pede coisas
//! (comandos `debug.*`, quadro selecionado, filhos de uma variável).
//!
//! Variáveis abertas são lembradas pelo caminho (`Locals/pessoa/nome`), não pela referência do
//! adaptador, que muda a cada parada: assim o que estava aberto continua aberto depois de um passo.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::collections::{HashMap, HashSet, VecDeque};

use iced::widget::{button, column, container, row, rule, space, text};
use iced::{Element, Fill, Font, Task, Theme};
use tea_core::debug::{Event, Frame, Scope, Status, Variable};
use tea_core::icons::{self, Icon};
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("debug");
/// Linhas de saída guardadas.
const MAX_OUTPUT: usize = 5000;
/// Linhas de saída desenhadas (as últimas).
const SHOWN_OUTPUT: usize = 1000;
/// Valores muito longos são cortados na tela.
const MAX_VALUE_CHARS: usize = 200;

#[derive(Debug, Clone)]
pub enum Message {
    Command(&'static str),
    SelectFrame(usize),
    ToggleScope(String),
    /// Caminho da variável e a referência dela (para pedir os filhos).
    ToggleVariable(String, i64),
}

#[derive(Default)]
pub struct DebugExtension {
    status: Status,
    frames: Vec<Frame>,
    selected: usize,
    scopes: Vec<Scope>,
    /// Referência → filhos já recebidos.
    children: HashMap<i64, Vec<Variable>>,
    /// Referência → caminho, só das pedidas nesta parada (respostas de paradas velhas são ignoradas).
    asked: HashMap<i64, String>,
    /// Variáveis abertas (por caminho).
    expanded: HashSet<String>,
    /// Grupos que o usuário abriu ou fechou (o resto segue o padrão de [`opens_by_default`]).
    scope_open: HashMap<String, bool>,
    output: VecDeque<(String, String)>,
}

/// Argumentos e locais abrem sozinhos; registradores e globais (muitos, e o gdb não marca
/// como caros) só quando pedidos.
fn opens_by_default(scope: &Scope) -> bool {
    !scope.expensive && !matches!(scope.name.as_str(), "Registers" | "Globals")
}

impl DebugExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn scope_is_open(&self, scope: &Scope) -> bool {
        self.scope_open.get(&scope.name).copied().unwrap_or_else(|| opens_by_default(scope))
    }

    fn ask(&mut self, reference: i64, path: String, ctx: &mut Context) {
        if reference > 0 && !self.asked.contains_key(&reference) {
            self.asked.insert(reference, path);
            ctx.debug_variables(reference);
        }
    }

    /// Esquece o que valia só para a parada anterior.
    fn forget_stop(&mut self) {
        self.frames.clear();
        self.selected = 0;
        self.forget_variables();
    }

    fn forget_variables(&mut self) {
        self.scopes.clear();
        self.children.clear();
        self.asked.clear();
    }

    fn push_output(&mut self, category: &str, text: &str) {
        for line in text.lines() {
            self.output.push_back((category.to_string(), line.to_string()));
        }
        while self.output.len() > MAX_OUTPUT {
            self.output.pop_front();
        }
    }

    fn on_debug(&mut self, event: &Event, ctx: &mut Context) {
        match event {
            Event::Status(status) => {
                if self.status == Status::Idle && *status != Status::Idle {
                    ctx.show_panel(PANEL);
                }
                if !matches!(status, Status::Stopped { .. }) {
                    self.forget_stop();
                }
                self.status = status.clone();
            }
            Event::Stack { frames, selected } => {
                self.frames = frames.clone();
                self.selected = *selected;
            }
            Event::Scopes { scopes, .. } => {
                self.forget_variables();
                self.scopes = scopes.clone();
                for scope in scopes.clone() {
                    if self.scope_is_open(&scope) {
                        self.ask(scope.reference, scope.name.clone(), ctx);
                    }
                }
            }
            Event::Variables { reference, variables } => {
                let Some(parent) = self.asked.get(reference).cloned() else { return };
                // Filhos que estavam abertos na parada anterior: pede de novo.
                for v in variables {
                    let path = format!("{parent}/{}", v.name);
                    if v.reference > 0 && self.expanded.contains(&path) {
                        self.ask(v.reference, path, ctx);
                    }
                }
                self.children.insert(*reference, variables.clone());
            }
            Event::Output { category, text } => self.push_output(category, text),
            Event::OutputCleared => self.output.clear(),
        }
    }
}

impl Extension for DebugExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "debug"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        ctx.register_panel_hidden(PANEL, fl!("debug-panel"), Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::Debug);
        ctx.register_command("debug.focus", fl!("cmd-debug-focus"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        if let CoreEvent::Debug(e) = event {
            self.on_debug(e, ctx);
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if command == "debug.focus" {
            ctx.show_panel(PANEL);
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Command(id) => ctx.run_command(id),
            Message::SelectFrame(i) => ctx.debug_select_frame(i),
            Message::ToggleScope(name) => {
                let Some(scope) = self.scopes.iter().find(|s| s.name == name).cloned() else { return Task::none() };
                let open = !self.scope_is_open(&scope);
                self.scope_open.insert(name.clone(), open);
                if open {
                    self.ask(scope.reference, name, ctx);
                }
            }
            Message::ToggleVariable(path, reference) => {
                if !self.expanded.remove(&path) {
                    self.expanded.insert(path.clone());
                    self.ask(reference, path, ctx);
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let body = row![
            container(self.stack_view()).width(260).height(Fill),
            rule::vertical(1),
            container(self.variables_view()).width(Fill).height(Fill),
            rule::vertical(1),
            container(self.output_view()).width(Fill).height(Fill),
        ]
        .spacing(6)
        .height(Fill);
        column![self.toolbar(), rule::horizontal(1), body].spacing(4).into()
    }
}

// ---- Telas -------------------------------------------------------------------

fn secondary(t: &Theme) -> text::Style {
    text::secondary(t)
}

fn heading<'a>(label: String) -> Element<'a, Message> {
    text(label).size(11).style(secondary).into()
}

/// Uma linha de lista clicável, destacada quando `selected`.
fn list_button<'a>(content: impl Into<Element<'a, Message>>, msg: Message, selected: bool) -> Element<'a, Message> {
    button(content)
        .on_press(msg)
        .padding([1, 4])
        .width(Fill)
        .style(move |t: &Theme, s| {
            let base = button::text(t, s);
            if selected {
                let p = t.extended_palette();
                button::Style { background: Some(p.primary.weak.color.scale_alpha(0.35).into()), ..base }
            } else {
                base
            }
        })
        .into()
}

fn chevron<'a>(open: bool) -> Element<'a, Message> {
    icons::icon(if open { Icon::ChevronDown } else { Icon::ChevronRight }).size(12).into()
}

fn one_line(s: &str) -> String {
    let line = s.lines().next().unwrap_or_default();
    match line.char_indices().nth(MAX_VALUE_CHARS) {
        Some((i, _)) => format!("{}…", &line[..i]),
        None if s.contains('\n') => format!("{line}…"),
        None => line.to_string(),
    }
}

impl DebugExtension {
    fn toolbar(&self) -> Element<'_, Message> {
        let stopped = matches!(self.status, Status::Stopped { .. });
        let running = self.status == Status::Running;
        let idle = self.status == Status::Idle;
        let when = |ok: bool, id: &'static str| ok.then_some(Message::Command(id));
        let label = match &self.status {
            Status::Idle => fl!("debug-idle"),
            Status::Building => fl!("debug-building"),
            Status::Running => fl!("debug-running"),
            Status::Stopped { reason } => fl!("debug-stopped", reason = reason.as_str()),
        };
        row![
            icons::icon_button(
                Icon::DebugContinue,
                if idle { fl!("debug-start") } else { fl!("debug-continue") },
                when(idle || stopped, "debug.continue")
            ),
            icons::icon_button(Icon::DebugPause, fl!("debug-pause"), when(running, "debug.pause")),
            icons::icon_button(Icon::DebugStepOver, fl!("debug-step-over"), when(stopped, "debug.next")),
            icons::icon_button(Icon::DebugStepInto, fl!("debug-step-into"), when(stopped, "debug.step_in")),
            icons::icon_button(Icon::DebugStepOut, fl!("debug-step-out"), when(stopped, "debug.step_out")),
            icons::icon_button(Icon::DebugStop, fl!("debug-stop"), when(!idle, "debug.stop")),
            space::horizontal().width(8),
            text(label).size(12).style(secondary),
        ]
        .spacing(2)
        .align_y(iced::Center)
        .into()
    }

    fn stack_view(&self) -> Element<'_, Message> {
        let mut list = column![heading(fl!("debug-call-stack"))].spacing(1);
        if self.frames.is_empty() {
            let hint = if matches!(self.status, Status::Stopped { .. }) { fl!("debug-no-stack") } else { fl!("debug-only-stopped") };
            list = list.push(text(hint).size(12).style(secondary));
        }
        for (i, f) in self.frames.iter().enumerate() {
            let place = match &f.path {
                Some(p) => format!("{}:{}", p.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(), f.line),
                None => fl!("debug-no-source"),
            };
            let icon = if i == 0 { Icon::DebugStackFrameActive } else { Icon::DebugStackFrame };
            let has_source = f.path.is_some();
            let line = row![
                icons::icon(icon).size(12),
                column![
                    text(short_function(&f.name)).size(12).style(move |t: &Theme| if has_source {
                        text::Style::default()
                    } else {
                        secondary(t)
                    }),
                    text(place).size(11).style(secondary),
                ]
            ]
            .spacing(6);
            list = list.push(list_button(line, Message::SelectFrame(i), i == self.selected));
        }
        tea_core::style::scrollable(list).height(Fill).into()
    }

    fn variables_view(&self) -> Element<'_, Message> {
        let mut list = column![heading(fl!("debug-variables"))].spacing(1);
        if self.scopes.is_empty() {
            list = list.push(text(fl!("debug-only-stopped")).size(12).style(secondary));
        }
        for scope in &self.scopes {
            let open = self.scope_is_open(scope);
            let head = row![chevron(open), text(&scope.name).size(12)].spacing(4).align_y(iced::Center);
            list = list.push(list_button(head, Message::ToggleScope(scope.name.clone()), false));
            if open {
                list = self.push_children(list, scope.reference, &scope.name, 1);
            }
        }
        tea_core::style::scrollable(list).height(Fill).into()
    }

    fn push_children<'a>(
        &'a self,
        mut list: iced::widget::Column<'a, Message>,
        reference: i64,
        parent: &str,
        depth: u16,
    ) -> iced::widget::Column<'a, Message> {
        let indent = space::horizontal().width(f32::from(depth) * 14.0);
        let Some(vars) = self.children.get(&reference) else {
            return list.push(row![indent, text("…").size(12).style(secondary)]);
        };
        if vars.is_empty() {
            return list.push(row![indent, text(fl!("debug-empty")).size(12).style(secondary)]);
        }
        for v in vars {
            let path = format!("{parent}/{}", v.name);
            let open = v.reference > 0 && self.expanded.contains(&path);
            let lead: Element<'_, Message> =
                if v.reference > 0 { chevron(open) } else { space::horizontal().width(12).into() };
            let mut line = row![
                space::horizontal().width(f32::from(depth) * 14.0),
                lead,
                text(&v.name).size(12).font(Font::MONOSPACE),
                text("=").size(12).style(secondary),
                text(one_line(&v.value)).size(12).font(Font::MONOSPACE).width(Fill),
            ]
            .spacing(4)
            .align_y(iced::Center);
            if let Some(t) = &v.type_name {
                line = line.push(text(one_line(t)).size(11).style(secondary));
            }
            if v.reference > 0 {
                list = list.push(list_button(line, Message::ToggleVariable(path.clone(), v.reference), false));
                if open {
                    list = self.push_children(list, v.reference, &path, depth + 1);
                }
            } else {
                list = list.push(container(line).padding([1, 4]));
            }
        }
        list
    }

    fn output_view(&self) -> Element<'_, Message> {
        let mut list = column![].spacing(0);
        if self.output.is_empty() {
            list = list.push(text(fl!("debug-nothing-yet")).size(12).style(secondary));
        }
        let skip = self.output.len().saturating_sub(SHOWN_OUTPUT);
        for (category, line) in self.output.iter().skip(skip) {
            let style: fn(&Theme) -> text::Style = match category.as_str() {
                "stderr" => |t| text::Style { color: Some(t.extended_palette().danger.base.color) },
                "stdout" => |_| text::Style::default(),
                _ => secondary,
            };
            list = list.push(text(line).size(12).font(Font::MONOSPACE).style(style));
        }
        column![heading(fl!("debug-output")), tea_core::style::scrollable(list).anchor_bottom().width(Fill).height(Fill)].spacing(1).into()
    }
}

/// `demo_debug::soma` → `soma` (o caminho completo fica longo demais na coluna estreita).
fn short_function(name: &str) -> &str {
    // Genéricos (`<T as X>::f`) têm `::` dentro de `<>`: corta só no último `::` fora deles.
    let mut depth = 0i32;
    let mut cut = 0;
    let bytes = name.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b':' if depth == 0 && bytes.get(i + 1) == Some(&b':') => cut = i + 2,
            _ => {}
        }
    }
    &name[cut..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tea_core::Request;

    fn var(name: &str, reference: i64) -> Variable {
        Variable { name: name.into(), value: "v".into(), type_name: None, reference }
    }

    fn scope(name: &str, reference: i64) -> Scope {
        Scope { name: name.into(), reference, expensive: false }
    }

    fn asked(ctx: Context) -> Vec<i64> {
        ctx.into_requests()
            .into_iter()
            .filter_map(|r| if let Request::DebugVariables(r) = r { Some(r) } else { None })
            .collect()
    }

    #[test]
    fn opens_locals_but_not_registers() {
        let root = PathBuf::from("/p");
        let mut ext = DebugExtension::new();
        let mut ctx = Context::new(&root, None);
        let scopes = vec![scope("Locals", 1), scope("Registers", 2), scope("Globals", 3)];
        ext.on_debug(&Event::Scopes { frame_id: 0, scopes }, &mut ctx);
        assert_eq!(asked(ctx), [1]);
    }

    #[test]
    fn expanded_variables_survive_a_step() {
        let root = PathBuf::from("/p");
        let mut ext = DebugExtension::new();
        let mut ctx = Context::new(&root, None);
        ext.on_debug(&Event::Scopes { frame_id: 0, scopes: vec![scope("Locals", 1)] }, &mut ctx);
        ext.on_debug(&Event::Variables { reference: 1, variables: vec![var("p", 7), var("n", 0)] }, &mut ctx);
        let _ = ext.update(Message::ToggleVariable("Locals/p".into(), 7), &mut ctx);
        assert_eq!(asked(ctx), [1, 7]);

        // Passo: referências novas; `p` continua aberta e é pedida de novo sozinha.
        let mut ctx = Context::new(&root, None);
        ext.on_debug(&Event::Status(Status::Running), &mut ctx);
        ext.on_debug(&Event::Status(Status::Stopped { reason: "passo".into() }), &mut ctx);
        ext.on_debug(&Event::Scopes { frame_id: 0, scopes: vec![scope("Locals", 11)] }, &mut ctx);
        ext.on_debug(&Event::Variables { reference: 1, variables: vec![var("velha", 0)] }, &mut ctx);
        assert!(!ext.children.contains_key(&1), "resposta da parada anterior é ignorada");
        ext.on_debug(&Event::Variables { reference: 11, variables: vec![var("p", 17)] }, &mut ctx);
        assert_eq!(asked(ctx), [11, 17]);
    }

    #[test]
    fn shows_panel_when_a_session_starts_and_caps_output() {
        let root = PathBuf::from("/p");
        let mut ext = DebugExtension::new();
        let mut ctx = Context::new(&root, None);
        ext.on_debug(&Event::Status(Status::Building), &mut ctx);
        ext.on_debug(&Event::Status(Status::Running), &mut ctx);
        let shows = ctx.into_requests().iter().filter(|r| matches!(r, Request::PanelVisibility { .. })).count();
        assert_eq!(shows, 1, "só ao sair de Idle");

        let mut ctx = Context::new(&root, None);
        let big = "x\n".repeat(MAX_OUTPUT + 10);
        ext.on_debug(&Event::Output { category: "stdout".into(), text: big }, &mut ctx);
        assert_eq!(ext.output.len(), MAX_OUTPUT);
    }

    #[test]
    fn short_names() {
        assert_eq!(short_function("demo_debug::soma"), "soma");
        assert_eq!(short_function("main"), "main");
        assert_eq!(short_function("<alloc::vec::Vec<T> as core::ops::Drop>::drop"), "drop");
        assert_eq!(one_line("a\nb"), "a…");
    }
}
