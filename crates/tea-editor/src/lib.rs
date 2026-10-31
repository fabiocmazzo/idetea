//! Fachada sobre o widget de edição.
//!
//! O resto da IDE só usa os tipos deste crate. Trocar o `iced-code-editor`
//! por outro widget deve mexer só aqui.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod image_file;
mod key_gate;
mod pdf_file;
mod lsp;
mod marks;

use std::collections::{BTreeSet, HashMap};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use iced::keyboard::key::Named;
use iced::widget::stack;
use iced::{Element, Task, Theme};
use iced_code_editor::{
    CodeEditor, LspDocument, LspOverlayMessage, LspOverlayState, LspTextChange, Message as Inner, theme,
    view_lsp_overlay,
};

use key_gate::KeyGate;
use lsp::PendingHover;

/// Mensagem interna do editor. Opaca para quem está de fora.
#[derive(Debug, Clone)]
pub struct EditorMessage(Msg);

#[derive(Debug, Clone)]
enum Msg {
    Widget(Inner),
    Overlay(LspOverlayMessage),
    /// Clique nos números de linha (linha começando em 0).
    ToggleBreakpoint(usize),
    /// Aba de PDF (zoom, páginas desenhadas...).
    Pdf(pdf_file::Msg),
}

impl EditorMessage {
    /// O editor pediu para salvar (`Ctrl+S` com foco nele, ou `:w` no modo vim).
    pub fn is_save_request(&self) -> bool {
        matches!(self.0, Msg::Widget(Inner::WriteRequested))
    }

    /// Clique nos números de linha: os breakpoints deste editor vão mudar.
    pub fn is_breakpoint_toggle(&self) -> bool {
        matches!(self.0, Msg::ToggleBreakpoint(_))
    }

    /// Mouse passando por cima do texto (não muda nada no arquivo).
    pub fn is_mouse_hover(&self) -> bool {
        matches!(self.0, Msg::Widget(Inner::MouseHover(_)))
    }

    /// Ctrl+clique (ir para definição).
    pub fn is_jump_click(&self) -> bool {
        matches!(self.0, Msg::Widget(Inner::JumpClick(_)))
    }
}

/// LSP ligado neste editor.
struct LspLink {
    client: tea_lsp::Client,
    language_id: String,
    /// Rótulo → texto a inserir, da última resposta de autocomplete.
    completions: HashMap<String, String>,
}

/// Ações de edição que a IDE expõe como comandos.
#[derive(Debug, Clone, Copy)]
pub enum EditorAction {
    Find,
    Replace,
    GotoLine,
    Undo,
    Redo,
    SelectAll,
    FoldAll,
    UnfoldAll,
    ToggleVim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::CrLf => "CRLF",
        }
    }
}

pub struct Editor {
    path: Option<PathBuf>,
    inner: CodeEditor,
    line_ending: LineEnding,
    /// Arquivo ainda não existe no disco (aberto pela linha de comando).
    is_new: bool,
    theme: Theme,
    lsp: Option<LspLink>,
    /// Sobe a cada edição enviada ao servidor de linguagem.
    revision: std::sync::Arc<std::sync::atomic::AtomicU64>,
    overlay: LspOverlayState,
    hover: Option<PendingHover>,
    /// Linhas com breakpoint (começando em 0). Andam junto com o texto ao editar.
    breakpoints: BTreeSet<usize>,
    /// Linha onde o programa parou no debug (começando em 0).
    exec_line: Option<usize>,
    /// Aba de imagem: mostra o arquivo em vez do texto e nunca grava nele.
    image: Option<image_file::ImageFile>,
    /// Aba de PDF: como a de imagem, só mostra.
    pdf: Option<pdf_file::PdfFile>,
}

impl Editor {
    pub fn scratch(text: &str) -> Self {
        Self::build(None, text, false)
    }

    /// Abre um arquivo. Se ele não existir, abre vazio e cria ao salvar.
    pub fn open(path: &Path) -> io::Result<Self> {
        if image_file::is_image(path) && path.is_file() {
            let mut editor = Self::build(Some(path), "", false);
            editor.image = Some(image_file::ImageFile::read(path)?);
            return Ok(editor);
        }
        if pdf_file::is_pdf(path) && path.is_file() {
            let mut editor = Self::build(Some(path), "", false);
            editor.pdf = Some(pdf_file::PdfFile::read(path)?);
            return Ok(editor);
        }
        match std::fs::read(path) {
            Ok(bytes) => {
                let text = String::from_utf8(bytes).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, fl!("not-utf8"))
                })?;
                Ok(Self::from_text(path, &text))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::build(Some(path), "", true)),
            Err(e) => Err(e),
        }
    }

    /// Cria o editor a partir de um texto já lido (útil para medir sem I/O).
    pub fn from_text(path: &Path, text: &str) -> Self {
        Self::build(Some(path), text, false)
    }

    fn build(path: Option<&Path>, text: &str, is_new: bool) -> Self {
        // O widget trabalha com `\n`. CRLF é convertido na leitura e restaurado ao salvar.
        let (text, line_ending) = if text.contains("\r\n") {
            (std::borrow::Cow::Owned(text.replace("\r\n", "\n")), LineEnding::CrLf)
        } else {
            (std::borrow::Cow::Borrowed(text), LineEnding::Lf)
        };
        let mut inner = CodeEditor::new(&text, &syntax_for(path));
        // A command palette é da IDE, não do widget.
        inner.set_command_palette_enabled(false);
        inner.set_language(match tea_core::i18n::current() {
            tea_core::i18n::Language::PortugueseBr => iced_code_editor::Language::PortugueseBR,
            tea_core::i18n::Language::English => iced_code_editor::Language::English,
            tea_core::i18n::Language::Spanish => iced_code_editor::Language::Spanish,
        });
        Self {
            path: path.map(Path::to_path_buf),
            inner,
            line_ending,
            is_new,
            theme: Theme::TokyoNight,
            lsp: None,
            revision: Default::default(),
            overlay: LspOverlayState::new(),
            hover: None,
            breakpoints: BTreeSet::new(),
            exec_line: None,
            image: None,
            pdf: None,
        }
    }

    /// Aba que só mostra o arquivo (imagem ou PDF), sem texto para editar.
    pub fn is_image(&self) -> bool {
        self.image.is_some() || self.pdf.is_some()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn file_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| fl!("untitled"))
    }

    pub fn title(&self) -> String {
        let name = self.file_name();
        if self.is_modified() { format!("● {name}") } else { name }
    }

    /// Alterações não salvas. Um arquivo novo com conteúdo também conta.
    pub fn is_modified(&self) -> bool {
        self.inner.is_modified()
    }

    /// Aba vazia, sem caminho e sem alterações: pode ser substituída.
    pub fn is_blank(&self) -> bool {
        self.path.is_none() && !self.is_modified()
    }

    pub fn text(&self) -> String {
        self.inner.content()
    }

    /// Linha e coluna do cursor, começando em 1.
    pub fn cursor(&self) -> (usize, usize) {
        let (l, c) = self.inner.cursor_position();
        (l + 1, c + 1)
    }

    pub fn language(&self) -> &'static str {
        if self.image.is_some() {
            return "Imagem";
        }
        if self.pdf.is_some() {
            return "PDF";
        }
        self.inner.syntax_name()
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn save(&mut self) -> io::Result<()> {
        // Imagem e PDF não têm alterações: não há o que gravar (gravar o texto vazio apagaria o arquivo).
        if self.is_image() {
            return Ok(());
        }
        let path = self.path.clone().ok_or_else(|| io::Error::other("arquivo sem caminho"))?;
        self.write_to(&path)
    }

    /// Salva em outro caminho e passa a editar esse arquivo.
    /// Depois de mudar de caminho, o LSP é desligado: o app religa conforme a nova linguagem.
    pub fn save_as(&mut self, path: &Path) -> io::Result<()> {
        if self.path.as_deref() != Some(path) {
            self.detach_lsp();
        }
        if self.is_image() {
            // Salvar como de uma imagem ou PDF: cópia do arquivo.
            if let Some(from) = &self.path
                && from != path
            {
                std::fs::copy(from, path)?;
            }
        } else {
            self.write_to(path)?;
        }
        if self.path.as_deref() != Some(path) {
            self.path = Some(path.to_path_buf());
            self.inner.set_syntax(&syntax_for(Some(path)));
        }
        Ok(())
    }

    fn write_to(&mut self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent()
            && !dir.as_os_str().is_empty()
        {
            std::fs::create_dir_all(dir)?;
        }
        let mut text = self.inner.content();
        if self.line_ending == LineEnding::CrLf {
            text = text.replace('\n', "\r\n");
        }
        std::fs::write(path, text)?;
        self.inner.mark_saved();
        self.is_new = false;
        if self.lsp.is_some() {
            self.inner.lsp_did_save();
        }
        Ok(())
    }

    /// O arquivo foi renomeado ou movido por fora (ex.: árvore de arquivos).
    pub fn set_path(&mut self, path: PathBuf) {
        self.detach_lsp();
        self.inner.set_syntax(&syntax_for(Some(&path)));
        self.path = Some(path);
    }

    pub fn is_new_file(&self) -> bool {
        self.is_new
    }

    pub fn set_theme(&mut self, theme: &Theme) {
        self.inner.set_theme(theme::from_iced_theme(theme));
        self.theme = theme.clone();
    }

    pub fn set_font_size(&mut self, size: f32) {
        if (self.inner.font_size() - size).abs() > f32::EPSILON {
            self.inner.set_font_size(size, true);
        }
    }

    /// Dá foco total ao editor. Só `request_focus` não basta: sem
    /// `CanvasFocusGained` o widget ignora o teclado até levar um clique.
    pub fn focus(&mut self) {
        self.inner.request_focus();
        let _ = self.inner.update(&Inner::CanvasFocusGained);
    }

    pub fn lose_focus(&mut self) {
        self.inner.lose_focus();
    }

    /// Move o cursor. Linha e coluna começam em 1 (coluna em caracteres).
    pub fn goto(&mut self, line: usize, column: usize) -> Task<EditorMessage> {
        self.widget(Inner::GotoPosition(line.saturating_sub(1), column.saturating_sub(1)))
    }

    pub fn run(&mut self, action: EditorAction) -> Task<EditorMessage> {
        if self.is_image() {
            return Task::none();
        }
        let msg = match action {
            EditorAction::Find => Inner::OpenSearch,
            EditorAction::Replace => Inner::OpenSearchReplace,
            EditorAction::GotoLine => Inner::OpenGotoLine,
            EditorAction::Undo => Inner::Undo,
            EditorAction::Redo => Inner::Redo,
            EditorAction::SelectAll => Inner::SelectAll,
            EditorAction::FoldAll => Inner::FoldAll,
            EditorAction::UnfoldAll => Inner::UnfoldAll,
            EditorAction::ToggleVim => Inner::ToggleVimMode,
        };
        self.focus();
        self.widget(msg)
    }

    fn widget(&mut self, msg: Inner) -> Task<EditorMessage> {
        self.inner.update(&msg).map(|m| EditorMessage(Msg::Widget(m)))
    }

    pub fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        match message.0 {
            Msg::Pdf(m) => {
                let Some(pdf) = &mut self.pdf else { return Task::none() };
                pdf.update(m).map(|m| EditorMessage(Msg::Pdf(m)))
            }
            Msg::Widget(msg) => {
                match &msg {
                    Inner::MouseHover(point) => {
                        if self.overlay.hover_visible && !self.overlay.hover_interactive {
                            self.overlay.clear_hover();
                        }
                        if self.lsp.is_some() {
                            self.hover = Some(PendingHover { point: *point, since: Instant::now() });
                        }
                    }
                    Inner::JumpClick(point) => {
                        let point = *point;
                        let task = self.widget(msg);
                        if self.lsp.is_some() {
                            self.inner.lsp_request_definition_at(point);
                        }
                        return task;
                    }
                    Inner::MouseClick(_) | Inner::Scrolled(_) => {
                        self.hover = None;
                        self.overlay.clear_hover();
                        self.overlay.clear_completions();
                    }
                    _ => {
                        // Digitar ou mover o cursor esconde o hover.
                        if !self.overlay.hover_interactive {
                            self.overlay.clear_hover();
                        }
                    }
                }
                // Breakpoints andam junto com o texto quando a edição muda o número de linhas.
                let before = (!self.breakpoints.is_empty())
                    .then(|| (self.inner.cursor_position().0, marks::last_line(&self.inner)));
                let task = self.widget(msg);
                if let Some((cursor_before, lines_before)) = before {
                    let delta = marks::last_line(&self.inner) as isize - lines_before as isize;
                    let at = cursor_before.min(self.inner.cursor_position().0);
                    marks::shift_breakpoints(&mut self.breakpoints, at, delta);
                }
                if self.overlay.completion_visible {
                    self.refresh_completion_filter();
                }
                task
            }
            Msg::ToggleBreakpoint(line) => {
                self.toggle_breakpoint(line);
                Task::none()
            }
            Msg::Overlay(msg) => {
                match msg {
                    LspOverlayMessage::HoverEntered => self.overlay.hover_interactive = true,
                    LspOverlayMessage::HoverExited => {
                        self.overlay.hover_interactive = false;
                        self.overlay.clear_hover();
                    }
                    LspOverlayMessage::CompletionSelected(i) => self.accept_completion(i),
                    LspOverlayMessage::CompletionClosed => self.overlay.clear_completions(),
                    LspOverlayMessage::CompletionNavigateUp => self.overlay.navigate(-1),
                    LspOverlayMessage::CompletionNavigateDown => self.overlay.navigate(1),
                    LspOverlayMessage::CompletionConfirm => self.accept_completion(self.overlay.completion_selected),
                }
                Task::none()
            }
        }
    }

    // ---- Debug ---------------------------------------------------------------

    /// Liga/desliga o breakpoint da linha (começando em 0).
    pub fn toggle_breakpoint(&mut self, line: usize) {
        if self.is_image() {
            return;
        }
        if !self.breakpoints.remove(&line) {
            self.breakpoints.insert(line);
        }
    }

    /// Linhas com breakpoint, começando em 0.
    pub fn breakpoints(&self) -> impl Iterator<Item = usize> + '_ {
        self.breakpoints.iter().copied()
    }

    /// Marca (ou limpa) a linha onde o programa parou, começando em 0.
    pub fn set_exec_line(&mut self, line: Option<usize>) {
        self.exec_line = line;
    }

    pub fn exec_line(&self) -> Option<usize> {
        self.exec_line
    }

    pub fn view(&self) -> Element<'_, EditorMessage> {
        if let Some(img) = &self.image {
            return img.view();
        }
        if let Some(pdf) = &self.pdf {
            return pdf.view().map(|m| EditorMessage(Msg::Pdf(m)));
        }
        let widget = self.inner.view().map(|m| EditorMessage(Msg::Widget(m)));
        let marks = marks::Marks::new(&self.inner, &self.breakpoints, self.exec_line, |line| {
            EditorMessage(Msg::ToggleBreakpoint(line))
        });
        let editor: Element<'_, EditorMessage> = stack![widget, marks].into();
        let content: Element<'_, EditorMessage> = if self.lsp.is_some() {
            let overlay = view_lsp_overlay(
                &self.overlay,
                &self.inner,
                &self.theme,
                self.inner.font_size(),
                self.inner.line_height(),
                |m| EditorMessage(Msg::Overlay(m)),
            );
            stack![editor, overlay].into()
        } else {
            editor
        };
        KeyGate::new(content)
            .intercept(self.overlay.completion_visible, |key| {
                let m = match key {
                    Named::ArrowUp => LspOverlayMessage::CompletionNavigateUp,
                    Named::ArrowDown => LspOverlayMessage::CompletionNavigateDown,
                    Named::Enter | Named::Tab => LspOverlayMessage::CompletionConfirm,
                    Named::Escape => LspOverlayMessage::CompletionClosed,
                    _ => return None,
                };
                Some(EditorMessage(Msg::Overlay(m)))
            })
            .into()
    }

    // ---- LSP ---------------------------------------------------------------

    /// Liga o editor a um servidor LSP. Precisa de caminho.
    pub fn attach_lsp(&mut self, client: tea_lsp::Client, language_id: &str) {
        let Some(path) = self.path.clone() else { return };
        self.detach_lsp();
        let uri = tea_lsp::uri::path_to_uri(&path);
        let bridge = lsp::Bridge { client: client.clone(), path, revision: self.revision.clone() };
        self.inner.attach_lsp(Box::new(bridge), LspDocument::new(uri, language_id));
        self.lsp = Some(LspLink { client, language_id: language_id.to_string(), completions: HashMap::new() });
    }

    pub fn detach_lsp(&mut self) {
        if self.lsp.take().is_some() {
            self.inner.detach_lsp();
        }
        self.overlay.clear_hover();
        self.overlay.clear_completions();
        self.hover = None;
    }

    /// Nome do servidor ligado, se houver.
    pub fn lsp_server(&self) -> Option<&str> {
        self.lsp.as_ref().map(|l| l.client.name())
    }

    pub fn lsp_language(&self) -> Option<&str> {
        self.lsp.as_ref().map(|l| l.language_id.as_str())
    }

    /// Há um hover esperando o mouse parar: o app deve chamar [`Editor::tick`].
    pub fn needs_tick(&self) -> bool {
        self.hover.is_some()
    }

    pub fn tick(&mut self, now: Instant) {
        let Some(h) = &self.hover else { return };
        if !h.is_due(now) {
            return;
        }
        let point = h.point;
        self.hover = None;
        if let Some((_, anchor)) = self.inner.lsp_hover_anchor_at_point(point)
            && self.inner.lsp_request_hover_at(point)
        {
            self.overlay.set_hover_position(anchor);
        }
    }

    pub fn show_hover(&mut self, text: String) {
        if self.overlay.hover_position.is_some() {
            self.overlay.show_hover(text);
        }
    }

    /// Hover no cursor (comando, sem mouse).
    pub fn request_hover(&mut self) {
        if self.lsp.is_some()
            && let Some(p) = self.inner.cursor_screen_position()
        {
            self.overlay.set_hover_position(p);
            self.inner.lsp_request_hover();
        }
    }

    pub fn request_definition(&mut self) -> bool {
        if self.lsp.is_none() {
            return false;
        }
        self.inner.lsp_request_definition();
        true
    }

    /// Muda quando o texto muda (só com servidor de linguagem ligado): dá para saber se algo
    /// derivado do texto (a estrutura do arquivo) precisa ser pedido de novo.
    pub fn text_revision(&self) -> u64 {
        self.revision.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Estrutura do arquivo (resposta em `tea_lsp::Event::DocumentSymbols`).
    pub fn request_document_symbols(&mut self) -> bool {
        let (Some(link), Some(path)) = (&self.lsp, &self.path) else { return false };
        link.client.document_symbols(path);
        true
    }

    /// Onde o símbolo no cursor é usado (resposta em `tea_lsp::Event::References`).
    pub fn request_references(&mut self) -> bool {
        self.request_at_cursor(tea_lsp::Client::references)
    }

    /// Implementações do símbolo no cursor (resposta em `tea_lsp::Event::Locations`).
    pub fn request_implementation(&mut self) -> bool {
        self.request_at_cursor(tea_lsp::Client::implementation)
    }

    /// Tipo do que está no cursor (resposta em `tea_lsp::Event::Locations`).
    pub fn request_type_definition(&mut self) -> bool {
        self.request_at_cursor(tea_lsp::Client::type_definition)
    }

    /// A função no cursor, para a árvore de chamadas (resposta em `tea_lsp::Event::CallRoots`).
    pub fn request_call_hierarchy(&mut self) -> bool {
        self.request_at_cursor(tea_lsp::Client::prepare_call_hierarchy)
    }

    fn request_at_cursor(&mut self, f: fn(&tea_lsp::Client, &Path, tea_lsp::Position)) -> bool {
        let (Some(link), Some(path)) = (&self.lsp, &self.path) else { return false };
        // O servidor precisa do texto atual antes de responder.
        self.inner.lsp_flush_pending_changes();
        let (line, column) = self.inner.cursor_position();
        f(&link.client, path, tea_lsp::Position::new(line as u32, column as u32));
        true
    }

    pub fn request_formatting(&mut self) -> bool {
        self.lsp.is_some() && self.inner.lsp_request_formatting()
    }

    pub fn apply_formatting(&mut self, edits: &[tea_lsp::TextEdit]) {
        let changes: Vec<LspTextChange> = edits
            .iter()
            .map(|e| LspTextChange { range: lsp::to_widget_range(e.range), text: e.new_text.clone() })
            .collect();
        self.inner.apply_lsp_text_edits(&changes);
    }

    pub fn show_completions(&mut self, items: Vec<tea_lsp::CompletionItem>) {
        let Some(link) = &mut self.lsp else { return };
        if items.is_empty() || self.overlay.completion_suppressed {
            self.overlay.clear_completions();
            return;
        }
        let mut labels = Vec::with_capacity(items.len());
        link.completions.clear();
        for item in items {
            labels.push(item.label.clone());
            link.completions.insert(item.label, item.insert);
        }
        let position = self.inner.cursor_screen_position().unwrap_or_default();
        self.overlay.completion_filter = self.current_prefix().map(|(p, _)| p).unwrap_or_default();
        self.overlay.set_completions(labels, position);
    }

    /// Prefixo da palavra no cursor e se o menu ainda faz sentido.
    fn current_prefix(&mut self) -> Option<(String, bool)> {
        let link = self.lsp.as_ref()?;
        let path = self.path.as_deref()?;
        self.inner.lsp_flush_pending_changes();
        let (line, column) = self.inner.cursor_position();
        let text = link.client.line_text(path, line as u32)?;
        Some(lsp::completion_prefix(&text, column))
    }

    fn refresh_completion_filter(&mut self) {
        match self.current_prefix() {
            Some((prefix, true)) => {
                self.overlay.completion_filter = prefix;
                self.overlay.filter_completions();
            }
            _ => self.overlay.clear_completions(),
        }
    }

    fn accept_completion(&mut self, index: usize) {
        let Some(label) = self.overlay.completion_items.get(index).cloned() else { return };
        let insert = self.lsp.as_ref().and_then(|l| l.completions.get(&label).cloned()).unwrap_or(label);
        self.overlay.clear_completions();
        let Some(link) = &self.lsp else { return };
        let Some(path) = self.path.as_deref() else { return };
        self.inner.lsp_flush_pending_changes();
        let (line, column) = self.inner.cursor_position();
        let start = link.client.line_text(path, line as u32).map_or(column, |t| lsp::word_start(&t, column));
        let range = tea_lsp::Range {
            start: tea_lsp::Position::new(line as u32, start as u32),
            end: tea_lsp::Position::new(line as u32, column as u32),
        };
        self.inner.apply_lsp_text_edits(&[LspTextChange { range: lsp::to_widget_range(range), text: insert }]);
        self.focus();
    }
}

impl Drop for Editor {
    /// Fechar a aba avisa o servidor (`didClose`).
    fn drop(&mut self) {
        self.detach_lsp();
    }
}

/// Nome de sintaxe para o syntect: a extensão do arquivo ou, sem ela, o nome (ex.: `Makefile`).
fn syntax_for(path: Option<&Path>) -> String {
    let Some(path) = path else { return "txt".into() };
    path.extension()
        .or_else(|| path.file_name())
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_else(|| "txt".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ctrl+clique no texto chega ao widget como "ir para definição", com as camadas da tela
    /// (marcas de debug e overlay do LSP) por cima dele.
    #[test]
    fn ctrl_click_reaches_the_widget() {
        use iced::keyboard::{Event as Key, Modifiers};
        let mut editor = Editor::from_text(&tmp("click.rs"), "fn main() {\n    soma(1);\n}\n");
        editor.focus();
        let lh = editor.inner.line_height();
        let mut ui = iced_test::simulator(editor.view());
        ui.point_at(iced::Point::new(marks::GUTTER_WIDTH + 40.0, lh * 1.5));
        ui.simulate([iced::Event::Keyboard(Key::ModifiersChanged(Modifiers::CTRL))]);
        ui.simulate(iced_test::simulator::click());
        let msgs: Vec<String> = ui.into_messages().map(|m| format!("{:?}", m.0)).collect();
        assert!(msgs.iter().any(|m| m.contains("JumpClick")), "{msgs:?}");
    }

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tea-editor-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn crlf_round_trip() {
        let path = tmp("crlf.txt");
        std::fs::write(&path, "a\r\nb\r\n").unwrap();
        let mut ed = Editor::open(&path).unwrap();
        assert_eq!(ed.line_ending(), LineEnding::CrLf);
        assert_eq!(ed.text(), "a\nb\n");
        ed.save().unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "a\r\nb\r\n");
    }

    #[test]
    fn missing_file_opens_empty_and_is_created_on_save() {
        let path = tmp("sub/novo.rs");
        let _ = std::fs::remove_file(&path);
        let mut ed = Editor::open(&path).unwrap();
        assert!(ed.is_new_file());
        assert_eq!(ed.text(), "");
        ed.save().unwrap();
        assert!(path.exists());
        assert!(!ed.is_new_file());
    }

    #[test]
    fn save_as_switches_path() {
        let mut ed = Editor::scratch("oi");
        assert!(ed.save().is_err());
        let path = tmp("salvo.md");
        ed.save_as(&path).unwrap();
        assert_eq!(ed.path(), Some(path.as_path()));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "oi");
    }

    #[test]
    fn non_utf8_is_an_error() {
        let path = tmp("bin.dat");
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        assert!(Editor::open(&path).is_err());
    }
}
