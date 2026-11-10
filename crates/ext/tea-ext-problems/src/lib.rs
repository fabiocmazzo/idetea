//! Painel "Problemas": erros e avisos de todas as fontes (LSP hoje, outras no futuro).
//! Clique leva à linha. `problems.next` pula para o próximo.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use iced::widget::{button, column, row, space, text};
use iced::{Element, Fill, Task, Theme};
use tea_core::icons::Icon;
use tea_core::{Context, CoreEvent, Diagnostic, Extension, PanelId, Severity, Slot};

const PANEL: PanelId = PanelId("problems");

#[derive(Debug, Clone)]
pub enum Message {
    Open { path: PathBuf, line: usize, column: usize },
}

#[derive(Default)]
pub struct ProblemsExtension {
    root: PathBuf,
    /// (arquivo, fonte) → diagnósticos. BTreeMap: ordem estável na tela.
    by_file: BTreeMap<(PathBuf, String), Vec<Diagnostic>>,
    /// Posição do último "próximo problema".
    cursor: usize,
}

/// Contagem de erros e avisos (para quem quiser mostrar um resumo).
pub fn count(items: &[Diagnostic]) -> (usize, usize) {
    let errors = items.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = items.iter().filter(|d| d.severity == Severity::Warning).count();
    (errors, warnings)
}

impl ProblemsExtension {
    pub fn new() -> Self {
        Self::default()
    }

    /// Todos os problemas na ordem da tela: por arquivo, erros primeiro, depois por linha.
    fn ordered(&self) -> Vec<(&Path, &Diagnostic)> {
        let mut files: BTreeMap<&Path, Vec<&Diagnostic>> = BTreeMap::new();
        for ((path, _), items) in &self.by_file {
            files.entry(path.as_path()).or_default().extend(items.iter());
        }
        let mut out = Vec::new();
        for (path, mut items) in files {
            items.sort_by_key(|d| (d.severity, d.line, d.column));
            out.extend(items.into_iter().map(|d| (path, d)));
        }
        out
    }

    fn rel<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, str> {
        path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy()
    }
}

impl Extension for ProblemsExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "problems"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, fl!("problems-panel"), Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::Warning);
        ctx.register_command("problems.focus", fl!("cmd-problems-focus"));
        ctx.register_command("problems.next", fl!("cmd-problems-next"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        if let CoreEvent::Diagnostics { source, path, items } = event {
            let key = (path.clone(), source.clone());
            if items.is_empty() {
                self.by_file.remove(&key);
            } else {
                self.by_file.insert(key, items.clone());
            }
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "problems.focus" => ctx.show_panel(PANEL),
            "problems.next" => {
                let all = self.ordered();
                if all.is_empty() {
                    ctx.notify(fl!("problems-none"));
                } else {
                    let i = self.cursor % all.len();
                    let (path, d) = all[i];
                    ctx.open_file_at(path, d.line, d.column);
                    ctx.notify(format!("{}/{}: {}", i + 1, all.len(), first_line(&d.message)));
                    self.cursor = i + 1;
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        let Message::Open { path, line, column } = message;
        ctx.open_file_at(path, line, column);
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let all = self.ordered();
        if all.is_empty() {
            return text(fl!("problems-none-found")).size(13).style(text::secondary).into();
        }
        let mut list = column![];
        let mut current: Option<&Path> = None;
        for (path, d) in &all {
            if current != Some(*path) {
                current = Some(path);
                let items: Vec<Diagnostic> = all.iter().filter(|(p, _)| p == path).map(|(_, d)| (*d).clone()).collect();
                let (e, w) = count(&items);
                list = list.push(
                    row![
                        text(self.rel(path)).size(13),
                        text(fl!("problems-count", errors = e, warnings = w)).size(12).style(text::secondary),
                    ]
                    .padding([4, 4]),
                );
            }
            let (icon, color): (&str, fn(&Theme) -> iced::Color) = match d.severity {
                Severity::Error => ("✖", |t| t.extended_palette().danger.base.color),
                Severity::Warning => ("⚠", |t| t.extended_palette().warning.base.color),
                Severity::Info => ("ℹ", |t| t.extended_palette().primary.base.color),
                Severity::Hint => ("·", |t| t.extended_palette().background.strong.color),
            };
            let source = match (&d.source, &d.code) {
                (Some(s), Some(c)) => format!("{s}({c})"),
                (Some(s), None) => s.clone(),
                (None, Some(c)) => c.clone(),
                (None, None) => String::new(),
            };
            let line = row![
                space::horizontal().width(14),
                text(icon).size(12).style(move |t: &Theme| text::Style { color: Some(color(t)) }),
                text(first_line(&d.message)).size(12).width(Fill),
                text(source).size(11).style(text::secondary),
                text(format!("{}:{}", d.line, d.column)).size(11).style(text::secondary),
            ]
            .spacing(8);
            list = list.push(
                button(line)
                    .on_press(Message::Open { path: path.to_path_buf(), line: d.line, column: d.column })
                    .style(|t: &Theme, st| tea_core::style::list_item(t, st, false))
                    .padding([1, 4])
                    .width(Fill),
            );
        }
        tea_core::style::scrollable(list).height(Fill).into()
    }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tea_core::Request;

    fn diag(line: usize, severity: Severity) -> Diagnostic {
        Diagnostic { line, column: 1, severity, message: format!("msg {line}"), source: None, code: None }
    }

    #[test]
    fn replaces_per_file_and_source_and_orders_errors_first() {
        let root = PathBuf::from("/p");
        let mut ext = ProblemsExtension::new();
        let mut ctx = Context::new(&root, None);
        let ev = |path: &str, source: &str, items| CoreEvent::Diagnostics {
            source: source.into(),
            path: PathBuf::from(path),
            items,
        };
        let _ = ext.on_event(&ev("/p/a.rs", "ra", vec![diag(9, Severity::Warning), diag(5, Severity::Error)]), &mut ctx);
        let _ = ext.on_event(&ev("/p/a.rs", "outro", vec![diag(1, Severity::Warning)]), &mut ctx);
        let _ = ext.on_event(&ev("/p/b.rs", "ra", vec![diag(2, Severity::Error)]), &mut ctx);
        let order: Vec<_> = ext.ordered().iter().map(|(p, d)| (p.display().to_string(), d.line)).collect();
        assert_eq!(order, [("/p/a.rs".into(), 5), ("/p/a.rs".into(), 1), ("/p/a.rs".into(), 9), ("/p/b.rs".into(), 2)]);

        // Lista vazia apaga só aquela fonte daquele arquivo.
        let _ = ext.on_event(&ev("/p/a.rs", "ra", vec![]), &mut ctx);
        assert_eq!(ext.ordered().len(), 2);
    }

    #[test]
    fn next_cycles_and_opens() {
        let root = PathBuf::from("/p");
        let mut ext = ProblemsExtension::new();
        let mut ctx = Context::new(&root, None);
        let _ = ext.on_event(
            &CoreEvent::Diagnostics { source: "ra".into(), path: "/p/a.rs".into(), items: vec![diag(3, Severity::Error)] },
            &mut ctx,
        );
        let _ = ext.on_command("problems.next", &mut ctx);
        let _ = ext.on_command("problems.next", &mut ctx);
        let opens = ctx.into_requests().into_iter().filter(|r| matches!(r, Request::OpenFileAt { line: 3, .. })).count();
        assert_eq!(opens, 2, "volta ao primeiro depois do último");
    }
}
