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
pub mod syntaxes;
mod lsp;
mod marks;
pub mod quick_diff;

use std::collections::{BTreeMap, BTreeSet, HashMap};
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
    /// Esc com a dica de parâmetros aberta.
    CloseSignature,
    /// Clique na marca do git (índice do trecho), fechar a caixa, desfazer o trecho.
    OpenHunk(usize),
    CloseHunk,
    RevertHunk(usize),
    /// Botão do mouse apertado com Shift: o próximo clique estende a seleção.
    ShiftPress,
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

    /// Clique no editor (texto, margem ou marca do git): com o editor dividido, o lado clicado ganha o foco.
    pub fn is_press(&self) -> bool {
        matches!(
            self.0,
            Msg::Widget(
                Inner::MouseClick(_)
                    | Inner::DoubleClick(_)
                    | Inner::TripleClick(_)
                    | Inner::AltClick(_)
                    | Inner::JumpClick(_)
                    | Inner::ContextMenuRequested(_)
            ) | Msg::ToggleBreakpoint(_)
                | Msg::OpenHunk(_)
        )
    }

    /// Tecla que muda o texto (digitar, apagar, Enter, colar...): o usuário está escrevendo.
    pub fn is_typing(&self) -> bool {
        matches!(
            self.0,
            Msg::Widget(
                Inner::CharacterInput(_)
                    | Inner::VimKey(_)
                    | Inner::Backspace
                    | Inner::Delete
                    | Inner::DeleteWordBack
                    | Inner::DeleteWordForward
                    | Inner::Enter
                    | Inner::Tab
                    | Inner::Backtab
                    | Inner::Paste(_)
                    | Inner::Cut
                    | Inner::DeleteSelection
            )
        )
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
    /// Recolhe ou expande o bloco do cursor (Ctrl+Shift+[ e ]).
    ToggleFold,
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

/// Como o editor mostra e grava o texto. Vem do config `[editor]` e do `.editorconfig` (quem monta é o app).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub wrap: bool,
    pub show_whitespace: bool,
    pub indent_guides: bool,
    /// Tamanho do recuo em espaços (o Tab do teclado). O caractere tab sempre ocupa 4 colunas na tela.
    pub tab_size: u8,
    pub insert_spaces: bool,
    pub trim_trailing_whitespace: bool,
    pub insert_final_newline: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wrap: true,
            show_whitespace: true,
            indent_guides: true,
            tab_size: 4,
            insert_spaces: true,
            trim_trailing_whitespace: false,
            insert_final_newline: false,
        }
    }
}

pub struct Editor {
    path: Option<PathBuf>,
    options: Options,
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
    /// Diagnósticos por servidor: uma resposta vazia limpa só quem a enviou.
    diagnostics: BTreeMap<String, Vec<tea_lsp::Diagnostic>>,
    /// Linhas com breakpoint (começando em 0). Andam junto com o texto ao editar.
    breakpoints: BTreeSet<usize>,
    /// Linha onde o programa parou no debug (começando em 0).
    exec_line: Option<usize>,
    /// Aba de imagem: mostra o arquivo em vez do texto e nunca grava nele.
    image: Option<image_file::ImageFile>,
    /// Aba de PDF: como a de imagem, só mostra.
    pdf: Option<pdf_file::PdfFile>,
    /// Abas de imagem e PDF: tamanho e data do arquivo quando foi lido (para perceber que mudou no disco).
    file_stamp: Option<(u64, Option<std::time::SystemTime>)>,
    /// Dica de parâmetros: `signature_open` = pedida (o cursor está numa chamada); `signature` = a que aparece.
    signature_open: bool,
    signature: Option<tea_lsp::SignatureHelp>,
    /// Versão de referência para as marcas do git (o último commit). `None`: sem marcas.
    diff_base: Option<std::sync::Arc<str>>,
    hunks: Vec<quick_diff::Hunk>,
    /// Hash do texto comparado por último (não compara de novo se nada mudou).
    diff_hash: u64,
    /// Trecho aberto (clique na marca).
    hunk_open: Option<usize>,
    /// Modo leve: o arquivo tem uma linha gigante (ver [`LONG_LINE`]). Guarda o nome da linguagem (o
    /// widget passa a achar que é texto puro).
    light: Option<String>,
    /// O próximo clique veio com Shift (ver [`Msg::ShiftPress`]).
    shift_press: bool,
    /// Onde começa a seleção feita com o mouse: o último clique sem Shift. Shift+clique estende a
    /// partir daqui; teclas que mexem o cursor ou o texto esquecem (aí vale o cursor).
    click_anchor: Option<(usize, usize)>,
    /// Fonte do código (`[editor] font`...). Começa com a do widget, que [`Self::set_code_font`] troca.
    code_font: tea_core::fonts::CodeFont,
}

/// Mensagens que mexem no cursor ou no texto sem o mouse: depois delas, Shift+clique estende a
/// partir do cursor, não do último clique.
fn forgets_click_anchor(msg: &Inner) -> bool {
    matches!(
        msg,
        Inner::CharacterInput(_)
            | Inner::VimKey(_)
            | Inner::Backspace
            | Inner::Delete
            | Inner::DeleteWordBack
            | Inner::DeleteWordForward
            | Inner::Enter
            | Inner::Tab
            | Inner::Backtab
            | Inner::Paste(_)
            | Inner::Cut
            | Inner::DeleteSelection
            | Inner::ArrowKey(..)
            | Inner::WordLeft(_)
            | Inner::WordRight(_)
            | Inner::Home(_)
            | Inner::End(_)
            | Inner::PageUp(_)
            | Inner::PageDown(_)
            | Inner::DoubleClick(_)
            | Inner::TripleClick(_)
            | Inner::AltClick(_)
            | Inner::JumpClick(_)
            | Inner::SelectAll
            | Inner::Undo
            | Inner::Redo
    )
}

/// Linha maior que isto (em bytes) liga o modo leve: sem ela, um JSON ou JS minificado trava a tela (o
/// desenho do texto cresce com o quadrado do tamanho da linha sem quebra, e o destaque refaz a linha inteira a
/// cada tecla). O mesmo limite do VS Code para parar o destaque.
pub const LONG_LINE: usize = 20_000;

fn has_long_line(text: &str) -> bool {
    text.split('\n').any(|l| l.len() > LONG_LINE)
}

impl Editor {
    pub fn scratch(text: &str) -> Self {
        Self::build(None, text, false)
    }

    /// Abre um arquivo. Se ele não existir, abre vazio e cria ao salvar.
    pub fn open(path: &Path) -> io::Result<Self> {
        if image_file::is_image(path) && path.is_file() {
            let mut editor = Self::build(Some(path), "", false);
            editor.file_stamp = file_stamp(path);
            editor.image = Some(image_file::ImageFile::read(path)?);
            return Ok(editor);
        }
        if pdf_file::is_pdf(path) && path.is_file() {
            let mut editor = Self::build(Some(path), "", false);
            editor.file_stamp = file_stamp(path);
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
        let mut editor = Self {
            path: path.map(Path::to_path_buf),
            options: Options::default(),
            inner,
            line_ending,
            is_new,
            theme: Theme::TokyoNight,
            lsp: None,
            revision: Default::default(),
            overlay: LspOverlayState::new(),
            hover: None,
            diagnostics: BTreeMap::new(),
            breakpoints: BTreeSet::new(),
            exec_line: None,
            image: None,
            pdf: None,
            file_stamp: None,
            signature_open: false,
            signature: None,
            diff_base: None,
            hunks: Vec::new(),
            diff_hash: 0,
            hunk_open: None,
            light: None,
            shift_press: false,
            click_anchor: None,
            code_font: tea_core::fonts::CodeFont { font: iced::Font::MONOSPACE, line_height: 20.0 / 14.0 },
        };
        if has_long_line(&text) {
            editor.enter_light_mode();
        }
        editor
    }

    /// Modo leve: quebra de linha sempre ligada e sem destaque, espaços visíveis, guias, cores de parênteses,
    /// amostras de cor e servidor de linguagem. Fica até fechar a aba.
    fn enter_light_mode(&mut self) {
        self.light = Some(self.inner.syntax_name().to_string());
        self.inner.set_syntax("text");
        self.inner.set_wrap_enabled(true);
        self.inner.set_show_whitespace(false);
        self.inner.set_show_indent_guides(false);
        self.inner.set_sticky_scroll_enabled(false);
        self.inner.set_bracket_pair_colorization_enabled(false);
        self.inner.set_bracket_match_highlight_enabled(false);
        self.inner.set_show_color_previews(false);
        self.inner.set_lsp_enabled(false);
    }

    /// Aberto no modo leve (linha gigante): a barra de status avisa.
    pub fn is_light(&self) -> bool {
        self.light.is_some()
    }

    /// Aba de imagem ou PDF cujo arquivo mudou no disco (tamanho ou data) desde que foi lido: hora de reler.
    /// Não depende do observador de arquivos: quem chama olha de tempos em tempos.
    pub fn file_changed_on_disk(&self) -> bool {
        match (&self.file_stamp, self.path()) {
            (Some(stamp), Some(path)) => file_stamp(path).is_some_and(|now| now != *stamp),
            _ => false,
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

    /// Troca o texto todo como uma edição (desfazer volta ao anterior): a aba fica com alterações.
    /// Só o trecho do meio que mudou é trocado (o começo e o fim iguais ficam).
    pub fn replace_text(&mut self, text: &str) {
        if self.is_image() {
            return;
        }
        let new = text.replace("\r\n", "\n");
        let old = self.inner.content();
        let prefix = common_prefix(&old, &new);
        let suffix = common_suffix(&old[prefix..], &new[prefix..]);
        let (old_end, new_end) = (old.len() - suffix, new.len() - suffix);
        if prefix == old_end && prefix == new_end {
            return;
        }
        let range = iced_code_editor::LspRange { start: lsp_position(&old, prefix), end: lsp_position(&old, old_end) };
        self.inner.apply_lsp_text_edits(&[LspTextChange { range, text: new[prefix..new_end].to_string() }]);
    }

    /// Linha e coluna do cursor, começando em 1.
    pub fn cursor(&self) -> (usize, usize) {
        let (l, c) = self.inner.cursor_position();
        (l + 1, c + 1)
    }

    pub fn language(&self) -> String {
        if self.image.is_some() {
            return fl!("language-image");
        }
        if self.pdf.is_some() {
            return "PDF".into();
        }
        if let Some(language) = &self.light {
            return fl!("language-light", language = language.clone());
        }
        self.inner.syntax_name().to_string()
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

    /// Muda as opções. As de tela valem na hora; as de salvar, no próximo salvar.
    pub fn set_options(&mut self, options: &Options) {
        if &self.options == options {
            return;
        }
        if self.light.is_none() {
            self.inner.set_wrap_enabled(options.wrap);
            self.inner.set_show_whitespace(options.show_whitespace);
            self.inner.set_show_indent_guides(options.indent_guides);
        }
        self.inner.set_indent_style(if options.insert_spaces {
            iced_code_editor::IndentStyle::Spaces(options.tab_size.max(1))
        } else {
            iced_code_editor::IndentStyle::Tab
        });
        self.options = options.clone();
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    /// O arquivo recua com tabs? `None`: não tem linha recuada para saber.
    pub fn indents_with_tabs(&self) -> Option<bool> {
        indents_with_tabs(&self.inner.content())
    }

    /// Limpa o texto antes de gravar, conforme as opções (um passo de desfazer, como formatar).
    fn tidy(&mut self) {
        let (trim, newline) = (self.options.trim_trailing_whitespace, self.options.insert_final_newline);
        if !trim && !newline {
            return;
        }
        let edits = tidy_edits(&self.inner.content(), trim, newline);
        if !edits.is_empty() {
            self.apply_formatting(&edits);
        }
    }

    fn write_to(&mut self, path: &Path) -> io::Result<()> {
        self.tidy();
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
        // Barra como a do resto da IDE (RFC 0015): sem trilho, polegar translúcido. Raio e largura são do widget.
        let mut style = theme::from_iced_theme(theme);
        style.scrollbar_background = iced::Color::TRANSPARENT;
        style.scroller_color = theme.extended_palette().background.strongest.color.scale_alpha(0.5);
        self.inner.set_theme(style);
        self.theme = theme.clone();
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.set_code_font(size, self.code_font);
    }

    /// Fonte, tamanho e altura da linha do código (os números de linha vêm junto: o widget usa a mesma).
    pub fn set_code_font(&mut self, size: f32, font: tea_core::fonts::CodeFont) {
        if font.font != self.code_font.font {
            self.inner.set_font(font.font);
        }
        if (self.inner.font_size() - size).abs() > f32::EPSILON {
            self.inner.set_font_size(size, false);
        }
        self.code_font = font;
        let line = font.line_px(size);
        if (self.inner.line_height() - line).abs() > f32::EPSILON {
            self.inner.set_line_height(line);
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

    /// Este editor recebe o teclado?
    pub fn is_focused(&self) -> bool {
        self.inner.is_focused()
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
            EditorAction::ToggleFold => Inner::ToggleFoldAtCursor,
            EditorAction::ToggleVim => Inner::ToggleVimMode,
        };
        self.focus();
        self.widget(msg)
    }

    fn widget(&mut self, msg: Inner) -> Task<EditorMessage> {
        self.inner.update(&msg).map(|m| EditorMessage(Msg::Widget(m)))
    }

    pub fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        if let Msg::ShiftPress = message.0 {
            self.shift_press = true;
            return Task::none();
        }
        // Vale só para a mensagem que vem logo depois (o clique do mesmo evento).
        let shift_click = std::mem::take(&mut self.shift_press);
        match message.0 {
            Msg::ShiftPress => Task::none(),
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
                        self.hunk_open = None;
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
                if let Inner::MouseClick(point) = msg
                    && shift_click
                {
                    return self.extend_selection(point);
                }
                if forgets_click_anchor(&msg) {
                    self.click_anchor = None;
                }
                let plain_click = matches!(msg, Inner::MouseClick(_));
                // Dica de parâmetros: abre ao digitar `(` ou `,` (o que o servidor disser); aberta, acompanha o cursor.
                let typed = match &msg {
                    Inner::CharacterInput(c) => Some(*c),
                    _ => None,
                };
                let cursor_before = self.inner.cursor_position();
                // Breakpoints andam junto com o texto quando a edição muda o número de linhas.
                let before = (!self.breakpoints.is_empty())
                    .then(|| (self.inner.cursor_position().0, marks::last_line(&self.inner)));
                let task = self.widget(msg);
                if plain_click {
                    self.click_anchor = Some(self.inner.cursor_position());
                }
                if let Some((cursor_before, lines_before)) = before {
                    let delta = marks::last_line(&self.inner) as isize - lines_before as isize;
                    let at = cursor_before.min(self.inner.cursor_position().0);
                    marks::shift_breakpoints(&mut self.breakpoints, at, delta);
                }
                if self.overlay.completion_visible {
                    self.refresh_completion_filter();
                }
                let trigger = typed.is_some_and(|c| self.lsp.as_ref().is_some_and(|l| l.client.signature_triggers().contains(&c)));
                if trigger || (self.signature_open && self.inner.cursor_position() != cursor_before) {
                    self.request_signature();
                }
                task
            }
            Msg::CloseSignature => {
                self.close_signature();
                Task::none()
            }
            Msg::OpenHunk(i) => {
                self.hunk_open = (self.hunk_open != Some(i) && i < self.hunks.len()).then_some(i);
                Task::none()
            }
            Msg::CloseHunk => {
                self.hunk_open = None;
                Task::none()
            }
            Msg::RevertHunk(i) => {
                if let Some(h) = self.hunks.get(i).cloned() {
                    let text = quick_diff::revert(&self.inner.content(), &h);
                    self.replace_text(&text);
                    self.hunk_open = None;
                    self.refresh_diff();
                }
                self.focus();
                Task::none()
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

    /// Shift+clique: seleciona da âncora (o último clique, ou o cursor) até o ponto clicado.
    ///
    /// O widget não faz isso sozinho. Aqui ele vira um arraste: o cursor vai para a âncora, um
    /// clique na margem (x < 0, onde o cursor não se mexe) começa a seleção ali, e o arraste até o
    /// ponto clicado a estende. Segurando o botão e movendo, a seleção continua acompanhando.
    fn extend_selection(&mut self, point: iced::Point) -> Task<EditorMessage> {
        let anchor = self.click_anchor.unwrap_or_else(|| self.inner.cursor_position());
        self.click_anchor = Some(anchor);
        // Sem rolar até a âncora: a tela fica onde a pessoa clicou.
        let _ = self.inner.set_cursor(anchor.0, anchor.1);
        let press = self.widget(Inner::MouseClick(iced::Point::new(-1.0, point.y)));
        let drag = self.widget(Inner::MouseDrag(point));
        Task::batch([press, drag])
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
        let marks = marks::Marks::new(
            &self.inner,
            &self.breakpoints,
            self.exec_line,
            &self.hunks,
            |line| EditorMessage(Msg::ToggleBreakpoint(line)),
            |i| EditorMessage(Msg::OpenHunk(i)),
        );
        let editor: Element<'_, EditorMessage> = match self.hunk_layer() {
            Some(hunk) => stack![widget, marks, hunk].into(),
            None => stack![widget, marks].into(),
        };
        let content: Element<'_, EditorMessage> = if self.lsp.is_some() {
            let overlay = view_lsp_overlay(
                &self.overlay,
                &self.inner,
                &self.theme,
                self.inner.font_size(),
                self.inner.line_height(),
                |m| EditorMessage(Msg::Overlay(m)),
            );
            match self.signature_layer() {
                Some(signature) => stack![editor, signature, overlay].into(),
                None => stack![editor, overlay].into(),
            }
        } else {
            editor
        };
        // Com o menu de autocomplete aberto, as setas, Enter, Tab e Esc são dele; senão, Esc fecha a dica.
        let completion = self.overlay.completion_visible;
        let hunk = self.hunk_open.is_some();
        KeyGate::new(content)
            .on_shift_press(EditorMessage(Msg::ShiftPress))
            .intercept(completion || self.signature.is_some() || hunk, move |key| {
                if !completion {
                    let close = if hunk { Msg::CloseHunk } else { Msg::CloseSignature };
                    return (key == Named::Escape).then_some(EditorMessage(close));
                }
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

    /// Substitui os diagnósticos de um servidor e redesenha os sublinhados de todos.
    pub fn set_diagnostics(&mut self, source: &str, diagnostics: &[tea_lsp::Diagnostic]) {
        use iced_code_editor::{Diagnostic as Mark, DiagnosticSeverity};
        if diagnostics.is_empty() {
            self.diagnostics.remove(source);
        } else {
            self.diagnostics.insert(source.to_string(), diagnostics.to_vec());
        }
        let all = self
            .diagnostics
            .values()
            .flatten()
            .map(|d| Mark {
                range: lsp::to_widget_range(d.range),
                severity: match d.severity {
                    tea_lsp::Severity::Error => DiagnosticSeverity::Error,
                    tea_lsp::Severity::Warning => DiagnosticSeverity::Warning,
                    tea_lsp::Severity::Info => DiagnosticSeverity::Information,
                    tea_lsp::Severity::Hint => DiagnosticSeverity::Hint,
                },
            })
            .collect();
        self.inner.set_diagnostics(all);
    }

    /// Liga o editor a um servidor LSP. Precisa de caminho.
    pub fn attach_lsp(&mut self, client: tea_lsp::Client, language_id: &str) {
        let Some(path) = self.path.clone().filter(|_| self.light.is_none()) else { return };
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
        self.diagnostics.clear();
        self.inner.set_diagnostics(Vec::new());
        self.overlay.clear_hover();
        self.overlay.clear_completions();
        self.hover = None;
        self.close_signature();
    }

    // ---- Marcas do git ---------------------------------------------------------

    /// Versão de referência (o último commit) para as marcas. `None` (arquivo fora do git, novo): sem marcas.
    pub fn set_diff_base(&mut self, base: Option<std::sync::Arc<str>>) {
        let base = base.map(|b| if b.contains("\r\n") { b.replace("\r\n", "\n").into() } else { b });
        if self.diff_base.as_deref() == base.as_deref() {
            return;
        }
        self.diff_base = base;
        self.diff_hash = 0;
        self.refresh_diff();
    }

    pub fn has_diff_base(&self) -> bool {
        self.diff_base.is_some()
    }

    /// Compara de novo com a referência (o app chama um instante depois de parar de digitar).
    pub fn refresh_diff(&mut self) {
        let Some(base) = self.diff_base.clone().filter(|_| !self.is_image()) else {
            self.hunks.clear();
            self.hunk_open = None;
            return;
        };
        let text = self.inner.content();
        let hash = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            text.hash(&mut h);
            h.finish() | 1
        };
        if hash == self.diff_hash {
            return;
        }
        self.diff_hash = hash;
        let hunks = quick_diff::hunks(&base, &text);
        if hunks != self.hunks {
            self.hunk_open = None;
            self.hunks = hunks;
        }
    }

    /// Trechos mudados (para testes e para quem quiser contar).
    pub fn hunks(&self) -> &[quick_diff::Hunk] {
        &self.hunks
    }

    /// Caixa do trecho aberto: como era, e o botão de desfazer. Fica logo abaixo do trecho.
    fn hunk_layer(&self) -> Option<Element<'_, EditorMessage>> {
        use iced::widget::{button, column, container, pin, row, space, text};
        let i = self.hunk_open?;
        let h = self.hunks.get(i)?;
        let lh = self.inner.line_height();
        let rows = marks::visible_rows(&self.inner);
        let last = marks::last_line(&self.inner);
        let end_line = match h.kind {
            quick_diff::HunkKind::Deleted => h.start.min(last),
            _ => h.start + h.len.saturating_sub(1),
        };
        let row_y = rows.iter().find(|r| r.line == end_line).map(|r| r.y)?;
        let fs = self.inner.font_size();
        let mono = self.code_font.font;
        let title = match h.kind {
            quick_diff::HunkKind::Added => fl!("hunk-added", count = h.len),
            quick_diff::HunkKind::Modified => fl!("hunk-modified", count = h.len),
            quick_diff::HunkKind::Deleted => fl!("hunk-deleted", count = h.old.len()),
        };
        let header = row![
            text(title).size(fs * 0.85).style(text::secondary),
            space::horizontal(),
            button(text(fl!("hunk-revert")).size(fs * 0.85))
                .on_press(EditorMessage(Msg::RevertHunk(i)))
                .padding([2, 8])
                .style(button::secondary),
            button(text("×").size(fs * 0.9)).on_press(EditorMessage(Msg::CloseHunk)).padding([2, 8]).style(button::text),
        ]
        .spacing(6)
        .align_y(iced::Center);
        let mut old = column![];
        for line in h.old.iter().take(200) {
            old = old.push(text(format!("- {line}")).font(mono).size(fs * 0.9).style(text::danger));
        }
        if h.old.is_empty() {
            old = old.push(text(fl!("hunk-added-note")).size(fs * 0.85).style(text::secondary));
        }
        let shown = h.old.len().clamp(1, 12) as f32;
        let body = tea_core::style::scrollable(old).height(iced::Length::Fixed(shown * lh * 0.95 + 4.0));
        let width = (self.inner.viewport_width() - marks::GUTTER_WIDTH - 12.0).clamp(160.0, 900.0);
        let card = container(column![header, body].spacing(4)).padding(6).width(width).style(|t: &Theme| {
            let p = t.extended_palette();
            container::Style {
                background: Some(p.background.weak.color.into()),
                border: iced::Border { color: p.primary.weak.color, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            }
        });
        let height = shown * lh * 0.95 + 4.0 + fs * 2.2 + 16.0;
        let below = row_y + lh;
        let top = if below + height > self.inner.viewport_height() && row_y > height { row_y - height } else { below };
        Some(pin(card).x(marks::GUTTER_WIDTH).y(top).into())
    }

    fn request_signature(&mut self) {
        let (Some(link), Some(path)) = (&self.lsp, &self.path) else { return };
        let (client, path) = (link.client.clone(), path.clone());
        self.inner.lsp_flush_pending_changes();
        let (line, column) = self.inner.cursor_position();
        client.signature_help(&path, tea_lsp::Position::new(line as u32, column as u32));
        self.signature_open = true;
    }

    /// Resposta do servidor. `None` (saiu da chamada, fechou o parêntese): esconde.
    pub fn show_signature(&mut self, help: Option<tea_lsp::SignatureHelp>) {
        if !self.signature_open {
            return;
        }
        self.signature_open = help.is_some();
        self.signature = help;
    }

    fn close_signature(&mut self) {
        self.signature_open = false;
        self.signature = None;
    }

    /// Caixa da dica, acima do cursor (ou abaixo, sem espaço em cima).
    fn signature_layer(&self) -> Option<Element<'_, EditorMessage>> {
        use iced::widget::text::Span;
        use iced::widget::{column, container, pin, rich_text, span, text};
        let help = self.signature.as_ref()?;
        let cursor = self.inner.cursor_screen_position()?;
        let (lh, fs, cw) = (self.inner.line_height(), self.inner.font_size(), self.inner.char_width().max(1.0));
        let y = cursor.y - self.inner.viewport_scroll();
        if y < 0.0 || y > self.inner.viewport_height() {
            return None;
        }
        let palette = self.theme.extended_palette();
        let chars: Vec<char> = help.label.chars().collect();
        let (s, e) = help.active.unwrap_or((0, 0));
        let piece = |a: usize, b: usize| chars[a.min(chars.len())..b.min(chars.len())].iter().collect::<String>();
        let bold = self.code_font.bold();
        let mut spans: Vec<Span<'_, (), iced::Font>> = Vec::new();
        if help.count > 1 {
            spans.push(span(format!("{}/{}  ", help.index + 1, help.count)).color(palette.background.strong.text));
        }
        spans.push(span(piece(0, s)).font(self.code_font.font));
        spans.push(span(piece(s, e)).font(bold).color(palette.primary.strong.color).underline(true));
        spans.push(span(piece(e, chars.len())).font(self.code_font.font));
        let mut body = column![rich_text(spans).size(fs)].spacing(4);
        if let Some(doc) = &help.doc {
            body = body.push(text(doc.as_str()).size(fs * 0.85).style(text::secondary));
        }

        // Tamanho estimado (para pôr acima do cursor sem medir): linhas quebram na largura máxima.
        let pad = 6.0;
        let max_w = (self.inner.viewport_width() - 16.0).max(120.0);
        let label_w = (chars.len() + 6) as f32 * cw;
        let doc_w = help.doc.as_ref().map_or(0.0, |d| d.chars().count() as f32 * cw * 0.85);
        let width = label_w.max(doc_w.min(max_w)).min(max_w) + pad * 2.0;
        let lines = |w: f32| (w / (width - pad * 2.0)).ceil().max(1.0);
        let doc_lines = if help.doc.is_some() { lines(doc_w) * 0.85 + 0.3 } else { 0.0 };
        let height = (lines(label_w) + doc_lines) * lh + pad * 2.0;
        let top = if y >= height + 2.0 { y - height - 2.0 } else { y + lh + 2.0 };
        let left = cursor.x.min(self.inner.viewport_width() - width).max(0.0);

        let card = container(body).padding(pad).max_width(width).style(move |t: &Theme| {
            let p = t.extended_palette();
            container::Style {
                background: Some(p.background.weak.color.into()),
                border: iced::Border { color: p.primary.weak.color, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            }
        });
        Some(pin(card).x(left).y(top).into())
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

    fn diagnostic_at(&self, point: iced::Point) -> Option<&tea_lsp::Diagnostic> {
        let p = self.inner.lsp_position_at_point(point)?;
        let position = (p.line, p.character);
        self.diagnostics
            .values()
            .flatten()
            .filter(|d| {
                let start = (d.range.start.line, d.range.start.character);
                let end = (d.range.end.line, d.range.end.character);
                if start == end {
                    position == start
                } else {
                    start <= position && position < end
                }
            })
            .min_by_key(|d| d.severity)
    }

    pub fn tick(&mut self, now: Instant) {
        let Some(h) = &self.hover else { return };
        if !h.is_due(now) {
            return;
        }
        let point = h.point;
        self.hover = None;
        if let Some(message) = self.diagnostic_at(point).map(|d| d.message.clone()) {
            if let Some((_, anchor)) = self.inner.lsp_hover_anchor_at_point(point) {
                self.overlay.set_hover_position(anchor);
                self.overlay.show_hover(message);
            }
            return;
        }
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

    /// Nome no cursor (letras, números e `_`), com a linha e a coluna onde ele começa (a partir de 0).
    pub fn word_at_cursor(&self) -> Option<(String, usize, usize)> {
        if self.is_image() {
            return None;
        }
        let (line, column) = self.inner.cursor_position();
        let text = self.inner.content();
        let chars: Vec<char> = text.split('\n').nth(line)?.chars().collect();
        let is_word = |c: &char| c.is_alphanumeric() || *c == '_';
        let column = column.min(chars.len());
        let start = chars[..column].iter().rposition(|c| !is_word(c)).map_or(0, |i| i + 1);
        let end = chars[column..].iter().position(|c| !is_word(c)).map_or(chars.len(), |i| column + i);
        (start < end).then(|| (chars[start..end].iter().collect(), line, start))
    }

    /// Pede para renomear o símbolo em `line`/`column` (a partir de 0). Resposta em `tea_lsp::Event::Rename`.
    pub fn request_rename(&mut self, line: usize, column: usize, new_name: &str) -> bool {
        let (Some(link), Some(path)) = (&self.lsp, &self.path) else { return false };
        self.inner.lsp_flush_pending_changes();
        link.client.rename(path, tea_lsp::Position::new(line as u32, column as u32), new_name);
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

/// Nome de sintaxe para o syntect (com as sintaxes extras do `two-face`): a extensão do arquivo ou, sem ela, o nome
/// (ex.: `Makefile`, `.env`). Alguns nomes que a extensão não diz: `.env.local` é `.env`, `Cargo.lock` é TOML.
fn syntax_for(path: Option<&Path>) -> String {
    let Some(path) = path else { return "txt".into() };
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if name.starts_with(".env.") {
        return ".env".into();
    }
    if ["Cargo.lock", "poetry.lock", "uv.lock", "Pipfile"].contains(&name.as_str()) {
        return "toml".into();
    }
    // Extensão composta de um language pack (`sum.mbt.md`): a gramática do pack declara `mbt.md`.
    if let Some(compound) = tea_core::languages::compound_extension(&name.to_lowercase()) {
        return compound.into();
    }
    let key = path
        .extension()
        .or_else(|| path.file_name())
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_else(|| "txt".into());
    match key.as_str() {
        "jsonc" | "json5" => "json".into(),
        "mdx" => "md".into(),
        // Ziex: a sintaxe do ZX (`tea_core::syntaxes`) cuida dos dois.
        "mdzx" => "zx".into(),
        _ => key,
    }
}

/// Bytes iguais no começo (em fronteira de caractere).
/// Edições para tirar espaços no fim das linhas e pôr a quebra de linha no fim (colunas em caracteres).
fn tidy_edits(text: &str, trim: bool, final_newline: bool) -> Vec<tea_lsp::TextEdit> {
    let edit = |line: usize, from: usize, to: usize, new_text: &str| tea_lsp::TextEdit {
        range: tea_lsp::Range {
            start: tea_lsp::Position::new(line as u32, from as u32),
            end: tea_lsp::Position::new(line as u32, to as u32),
        },
        new_text: new_text.to_string(),
    };
    let mut edits = Vec::new();
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.len() - 1;
    let needs_newline = final_newline && !text.is_empty() && !text.ends_with('\n');
    for (i, line) in lines.iter().enumerate() {
        let len = line.chars().count();
        let kept = if trim { line.trim_end_matches([' ', '\t']).chars().count() } else { len };
        let newline = i == last && needs_newline;
        if kept != len || newline {
            edits.push(edit(i, kept, len, if newline { "\n" } else { "" }));
        }
    }
    edits
}

/// Recuo com tabs ou espaços, pelo que a maioria das linhas recuadas usa (olha só o começo do arquivo).
fn indents_with_tabs(text: &str) -> Option<bool> {
    let (mut tabs, mut spaces) = (0usize, 0usize);
    for line in text.lines().take(2000) {
        if line.starts_with('\t') {
            tabs += 1;
        } else if line.starts_with("  ") && !line.trim_start().starts_with('*') {
            // Linha que começa com espaços e `*` costuma ser continuação de comentário `/* */`.
            spaces += 1;
        }
    }
    (tabs + spaces > 0).then_some(tabs > spaces)
}

fn common_prefix(a: &str, b: &str) -> usize {
    a.char_indices().zip(b.chars()).find(|((_, x), y)| x != y).map_or(a.len().min(b.len()), |((i, _), _)| i)
}

/// Bytes iguais no fim (em fronteira de caractere).
fn common_suffix(a: &str, b: &str) -> usize {
    a.chars().rev().zip(b.chars().rev()).take_while(|(x, y)| x == y).map(|(x, _)| x.len_utf8()).sum()
}

/// Posição (linha, coluna em caracteres, como o widget usa) de um byte do texto.
fn lsp_position(text: &str, byte: usize) -> iced_code_editor::LspPosition {
    let before = &text[..byte];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    iced_code_editor::LspPosition {
        line: u32::try_from(before.matches('\n').count()).unwrap_or(u32::MAX),
        character: u32::try_from(before[line_start..].chars().count()).unwrap_or(u32::MAX),
    }
}

/// Tamanho e data de um arquivo (`None`: não deu para ler).
fn file_stamp(path: &Path) -> Option<(u64, Option<std::time::SystemTime>)> {
    std::fs::metadata(path).ok().map(|m| (m.len(), m.modified().ok()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn compound_pack_extension_picks_its_grammar() {
        assert_eq!(syntax_for(Some(Path::new("docs/README.mbt.md"))), "mbt.md");
        assert_eq!(syntax_for(Some(Path::new("notes.md"))), "md");
    }

    /// Arquivos comuns que não eram coloridos (config, `.env`, TypeScript…) acham a sintaxe certa.
    #[test]
    fn common_files_get_a_syntax() {
        let name = |f: &str| CodeEditor::new("", &syntax_for(Some(Path::new(f)))).syntax_name();
        for (file, syntax) in [
            ("Cargo.toml", "TOML"),
            ("Cargo.lock", "TOML"),
            (".env", "DotENV"),
            (".env.local", "DotENV"),
            ("config.ini", "INI"),
            (".editorconfig", "INI"),
            ("Dockerfile", "Dockerfile"),
            (".gitignore", "Git Ignore"),
            ("a.ts", "TypeScript"),
            ("a.tsx", "TypeScriptReact"),
            ("a.jsonc", "JSON"),
            ("a.yaml", "YAML"),
            ("main.rs", "Rust"),
            ("Makefile", "Makefile"),
            ("notas.txt", "Plain Text"),
        ] {
            assert_eq!(name(file), syntax, "{file}");
        }
    }

    use super::*;

    #[test]
    fn git_marks_and_revert() {
        let mut ed = Editor::scratch("a\nB\nc\n");
        assert!(ed.hunks().is_empty(), "sem referência, sem marcas");
        ed.set_diff_base(Some("a\r\nb\r\nc\r\n".into()));
        assert_eq!(ed.hunks().len(), 1, "CRLF da referência não conta como mudança");
        assert_eq!(ed.hunks()[0].kind, quick_diff::HunkKind::Modified);
        let _ = ed.update(EditorMessage(Msg::OpenHunk(0)));
        assert_eq!(ed.hunk_open, Some(0));
        let _ = ed.update(EditorMessage(Msg::RevertHunk(0)));
        assert_eq!(ed.text(), "a\nb\nc\n");
        assert!(ed.hunks().is_empty() && ed.hunk_open.is_none());
        assert!(ed.is_modified(), "desfazer o trecho é uma edição (Ctrl+Z volta)");
        ed.set_diff_base(None);
        assert!(!ed.has_diff_base());
    }

    #[test]
    fn tidy_before_save() {
        let mut ed = Editor::scratch("fn a() {  \n\tx;\t\n}  ");
        ed.tidy();
        assert_eq!(ed.text(), "fn a() {  \n\tx;\t\n}  ", "opções desligadas: nada muda");
        ed.set_options(&Options { trim_trailing_whitespace: true, insert_final_newline: true, ..Options::default() });
        ed.tidy();
        assert_eq!(ed.text(), "fn a() {\n\tx;\n}\n");
        ed.tidy();
        assert_eq!(ed.text(), "fn a() {\n\tx;\n}\n", "já limpo");

        let mut ed = Editor::scratch("a  \nb");
        ed.set_options(&Options { insert_final_newline: true, ..Options::default() });
        ed.tidy();
        assert_eq!(ed.text(), "a  \nb\n", "só a linha final");
        let mut ed = Editor::scratch("");
        ed.set_options(&Options { insert_final_newline: true, ..Options::default() });
        ed.tidy();
        assert_eq!(ed.text(), "", "arquivo vazio fica vazio");
    }

    #[test]
    fn detects_tab_indent() {
        assert_eq!(indents_with_tabs("a\n\tb\n\tc\n  d"), Some(true));
        assert_eq!(indents_with_tabs("a\n    b\n/*\n * x\n */"), Some(false));
        assert_eq!(indents_with_tabs("a\nb"), None);
    }

    #[test]
    fn word_at_cursor() {
        let mut ed = Editor::from_text(Path::new("/p/a.rs"), "let conta_nova = soma(1);\n");
        let at = |ed: &mut Editor, col: usize| {
            let _ = ed.goto(1, col);
            ed.word_at_cursor().map(|(w, _, start)| (w, start))
        };
        assert_eq!(at(&mut ed, 7), Some(("conta_nova".into(), 4)), "no meio");
        assert_eq!(at(&mut ed, 5), Some(("conta_nova".into(), 4)), "no começo");
        assert_eq!(at(&mut ed, 15), Some(("conta_nova".into(), 4)), "logo depois do nome");
        assert_eq!(at(&mut ed, 16), None, "em cima do =");
        assert_eq!(at(&mut ed, 18), Some(("soma".into(), 17)));
    }

    #[test]
    fn replace_text_counts_as_a_change() {
        let mut ed = Editor::from_text(Path::new("/p/a.rs"), "fn velho() {}\nlinha 2\n");
        assert!(!ed.is_modified());
        ed.replace_text("fn novo() {}\r\n");
        assert_eq!(ed.text(), "fn novo() {}\n");
        assert!(ed.is_modified(), "recuperado: fica como alteração não salva");
        for (from, to) in [("abc", "xyz\n"), ("ação\nb\n", "ação\nção\nb\n"), ("", "novo"), ("um\ndois", "um")] {
            let mut e = Editor::from_text(Path::new("/p/b.rs"), from);
            e.replace_text(to);
            assert_eq!(e.text(), to, "{from:?} → {to:?}");
        }
    }

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
        ui.simulate([iced::Event::Keyboard(Key::ModifiersChanged(Modifiers::COMMAND))]);
        ui.simulate(iced_test::simulator::click());
        let msgs: Vec<String> = ui.into_messages().map(|m| format!("{:?}", m.0)).collect();
        assert!(msgs.iter().any(|m| m.contains("JumpClick")), "{msgs:?}");
    }

    /// Clique e Shift+clique selecionam do primeiro ponto ao segundo; um segundo Shift+clique
    /// estende a partir do mesmo começo. Digitar troca a seleção, e o texto que sobra mostra o trecho.
    #[test]
    fn shift_click_extends_the_selection() {
        use iced::keyboard::{Event as Key, Modifiers};
        let text = "abcdefgh\nijklmnop\nqrstuvwx\n";
        let mut editor = Editor::from_text(&tmp("shift.txt"), text);
        editor.focus();
        let (lh, cw) = (editor.inner.line_height(), editor.inner.char_width());
        let at = |line: usize, col: usize| {
            iced::Point::new(marks::GUTTER_WIDTH + 5.0 + cw * (col as f32 + 0.25), lh * (line as f32 + 0.5))
        };
        let click = |editor: &mut Editor, point, modifiers| {
            let msgs: Vec<EditorMessage> = {
                let mut ui = iced_test::simulator(editor.view());
                ui.simulate([iced::Event::Keyboard(Key::ModifiersChanged(modifiers))]);
                ui.point_at(point);
                ui.simulate(iced_test::simulator::click());
                ui.into_messages().collect()
            };
            for m in msgs {
                let _ = editor.update(m);
            }
        };
        click(&mut editor, at(0, 2), Modifiers::empty());
        click(&mut editor, at(1, 3), Modifiers::SHIFT);
        click(&mut editor, at(2, 4), Modifiers::SHIFT);
        let _ = editor.update(EditorMessage(Msg::Widget(Inner::CharacterInput('X'))));
        assert_eq!(editor.text(), "abXuvwx\n");

        // Sem Shift, o clique só move o cursor.
        let mut editor = Editor::from_text(&tmp("shift2.txt"), text);
        editor.focus();
        click(&mut editor, at(0, 2), Modifiers::empty());
        click(&mut editor, at(1, 3), Modifiers::empty());
        let _ = editor.update(EditorMessage(Msg::Widget(Inner::CharacterInput('X'))));
        assert_eq!(editor.text(), "abcdefgh\nijkXlmnop\nqrstuvwx\n");
    }

    #[test]
    fn diagnostics_from_each_server_reach_the_widget() {
        use tea_lsp::{Diagnostic, Position, Range, Severity};
        let diagnostic = |severity, message: &str| Diagnostic {
            range: Range {
                start: Position::new(0, 4),
                end: Position::new(0, 10),
            },
            severity,
            message: message.into(),
            source: None,
            code: None,
        };
        let mut ed = Editor::scratch("let answer = 42;\n");
        ed.set_diagnostics("rust-analyzer", &[diagnostic(Severity::Error, "erro")]);
        ed.set_diagnostics("clippy", &[diagnostic(Severity::Warning, "aviso")]);
        assert_eq!(ed.inner.diagnostics().len(), 2);
        assert!(
            ed.inner
                .diagnostics()
                .iter()
                .any(|d| d.severity == iced_code_editor::DiagnosticSeverity::Error)
        );
        let point = iced::Point::new(
            marks::GUTTER_WIDTH + 5.0 + ed.inner.char_width() * 5.5,
            ed.inner.line_height() * 0.5,
        );
        assert_eq!(ed.diagnostic_at(point).map(|d| d.message.as_str()), Some("erro"));
        assert!(
            ed.diagnostic_at(iced::Point::new(marks::GUTTER_WIDTH + 5.0, 1.0))
                .is_none()
        );

        ed.set_diagnostics("rust-analyzer", &[]);
        assert_eq!(
            ed.inner.diagnostics().len(),
            1,
            "limpar um servidor não apaga os outros"
        );
        assert_eq!(
            ed.inner.diagnostics()[0].severity,
            iced_code_editor::DiagnosticSeverity::Warning
        );

        ed.set_diagnostics("clippy", &[]);
        assert!(ed.inner.diagnostics().is_empty());
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

    /// Uma linha gigante (JSON minificado) abre no modo leve: sem destaque e sem desligar a quebra de linha.
    #[test]
    fn long_line_opens_in_light_mode() {
        let json = format!("[{}0]\n", "1,".repeat(LONG_LINE / 2));
        let mut ed = Editor::from_text(Path::new("/p/dados.json"), &json);
        assert!(ed.is_light());
        assert!(ed.language().contains("JSON"), "{}", ed.language());
        ed.set_options(&Options { wrap: false, ..Options::default() });
        assert!(ed.inner.wrap_enabled(), "a quebra de linha fica ligada");
        assert_eq!(ed.text(), json, "o texto não muda");

        let normal = Editor::from_text(Path::new("/p/dados.json"), &"[1,2]\n".repeat(10_000));
        assert!(!normal.is_light(), "muitas linhas curtas não é linha longa");
    }

    #[test]
    fn non_utf8_is_an_error() {
        let path = tmp("bin.dat");
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        assert!(Editor::open(&path).is_err());
    }

    #[test]
    fn image_tab_notices_the_file_changed() {
        let dir = std::env::temp_dir().join(format!("idetea-editor-stamp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("balao.png");
        ::image::RgbaImage::from_pixel(2, 2, ::image::Rgba([255, 0, 0, 255])).save(&path).unwrap();
        let editor = Editor::open(&path).unwrap();
        assert!(editor.is_image() && !editor.file_changed_on_disk());
        // Mesmo tamanho, data nova (como regerar o sprite com outra cor).
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(3);
        std::fs::File::options().write(true).open(&path).unwrap().set_modified(later).unwrap();
        assert!(editor.file_changed_on_disk());
        assert!(!Editor::open(&path).unwrap().file_changed_on_disk(), "relida: em dia");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
