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
mod table_view;

pub use tea_core::encoding::TextEncoding;
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
    /// Vista de tabela de um CSV, TSV ou JSONL.
    Table(table_view::Msg),
    /// Esc com a dica de parâmetros aberta.
    CloseSignature,
    /// Clique na marca do git (índice do trecho), fechar a caixa, desfazer o trecho.
    OpenHunk(usize),
    CloseHunk,
    RevertHunk(usize),
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
                    | Inner::ColumnSelectStart(_)
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
    /// Os diagnósticos mostrados no hover aberto, para o texto do servidor ir embaixo deles.
    hover_diagnostics: Option<String>,
    /// Diagnósticos por servidor: uma resposta vazia limpa só quem a enviou.
    diagnostics: BTreeMap<String, Vec<tea_lsp::Diagnostic>>,
    /// O texto ao qual os diagnósticos guardados se referem, e a revisão dele (ver `follow_edits`).
    diagnostics_text: Option<(u64, String)>,
    /// Inlay hints do servidor (RFC 0018), andando com o texto como os diagnósticos.
    inlay_hints: Vec<tea_lsp::InlayHint>,
    /// O texto ao qual os hints se referem: o do pedido, até a resposta; depois, o da última edição seguida.
    inlay_text: Option<(u64, String)>,
    /// O pedido de hints mais recente (uma resposta de um pedido anterior é descartada) e o texto dele.
    inlay_token: u64,
    inlay_request_text: Option<(u64, String)>,
    inlay_enabled: bool,
    /// Semantic tokens do servidor (RFC 0019), andando com o texto como os inlay hints.
    semantic_tokens: Vec<tea_lsp::SemanticToken>,
    semantic_text: Option<(u64, String)>,
    semantic_token: u64,
    semantic_request_text: Option<(u64, String)>,
    semantic_enabled: bool,
    /// Linhas com breakpoint (começando em 0). Andam junto com o texto ao editar.
    breakpoints: BTreeSet<usize>,
    /// Linha onde o programa parou no debug (começando em 0).
    exec_line: Option<usize>,
    /// Aba de imagem: mostra o arquivo em vez do texto e nunca grava nele.
    image: Option<image_file::ImageFile>,
    /// Aba de PDF: como a de imagem, só mostra.
    pdf: Option<pdf_file::PdfFile>,
    /// CSV, TSV ou JSONL: a vista de tabela (RFC 0020), outra vista deste texto.
    table: Option<table_view::TableView>,
    /// O encoding do arquivo (RFC 0021): lido ao abrir, usado ao salvar.
    encoding: TextEncoding,
    /// O limite de tamanho da vista de tabela com que a aba abriu (ver [`Editor::set_table_limit`]).
    table_limit: usize,
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
    /// Fonte do código (`[editor] font`...). Começa com a do widget, que [`Self::set_code_font`] troca.
    code_font: tea_core::fonts::CodeFont,
}

/// Arquivo maior que isto (em bytes) não abre como tabela (RFC 0020): fica só o texto. O padrão de `[table] max_mb`.
pub const TABLE_MAX_BYTES: usize = 50 * 1024 * 1024;

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
        Self::open_with(path, None, encoding_rs::WINDOWS_1252)
    }

    /// Abre no encoding do arquivo (RFC 0021): o BOM, o `hint` (o `charset` do `.editorconfig`, ou o encoding com que
    /// a aba estava, ao recarregar), UTF-8, e por fim `fallback` (`[editor] fallback_encoding`). Binário é recusado.
    pub fn open_with(path: &Path, hint: Option<TextEncoding>, fallback: &'static encoding_rs::Encoding) -> io::Result<Self> {
        if (image_file::is_image(path) || pdf_file::is_pdf(path)) && path.is_file() {
            return Self::open(path);
        }
        match std::fs::read(path) {
            Ok(bytes) => {
                let (text, encoding) = tea_core::encoding::decode(&bytes, hint, fallback).map_err(decode_error)?;
                let mut editor = Self::from_text(path, &text);
                editor.encoding = encoding;
                Ok(editor)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let mut editor = Self::build(Some(path), "", true);
                editor.encoding = hint.unwrap_or_default();
                Ok(editor)
            }
            Err(e) => Err(e),
        }
    }

    /// "Reabrir com…": lê o arquivo neste encoding. Bytes que não valem nele são erro (nada de `�`, que corromperia
    /// o arquivo ao salvar). Um BOM do próprio encoding fica (salvar não o tira).
    pub fn open_as(path: &Path, encoding: TextEncoding) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bom = matches!(encoding_rs::Encoding::for_bom(&bytes), Some((e, _)) if e == encoding.encoding);
        let encoding = TextEncoding { bom, ..encoding };
        let text = tea_core::encoding::decode_as(&bytes, encoding).map_err(decode_error)?;
        let mut editor = Self::from_text(path, &text);
        editor.encoding = encoding;
        Ok(editor)
    }

    /// O encoding em que o arquivo foi lido e será gravado.
    pub fn encoding(&self) -> TextEncoding {
        self.encoding
    }

    /// "Salvar com…": grava no encoding escolhido, que passa a ser o da aba. Se um caractere não cabe, nada muda.
    pub fn save_with_encoding(&mut self, encoding: TextEncoding) -> io::Result<()> {
        let before = self.encoding;
        self.encoding = encoding;
        let saved = self.save();
        if saved.is_err() {
            self.encoding = before;
        }
        saved
    }

    /// Cria o editor a partir de um texto já lido (útil para medir sem I/O).
    pub fn from_text(path: &Path, text: &str) -> Self {
        let mut editor = Self::build(Some(path), text, false);
        // CSV, TSV e JSONL abrem como tabela (RFC 0020), até um tamanho: acima dele, só o texto.
        if text.len() <= TABLE_MAX_BYTES
            && editor.light.is_none()
            && let Some(format) = tea_table::Format::detect(path, text)
        {
            let content = editor.inner.content();
            editor.table = Some(table_view::TableView::new(format, &content, editor.text_revision()));
            editor.count_edits();
        }
        editor
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
            tea_core::i18n::Language::Spanish => iced_code_editor::Language::Spanish,
            tea_core::i18n::Language::French => iced_code_editor::Language::French,
            tea_core::i18n::Language::German => iced_code_editor::Language::German,
            tea_core::i18n::Language::Italian => iced_code_editor::Language::Italian,
            tea_core::i18n::Language::ChineseSimplified => iced_code_editor::Language::ChineseSimplified,
            // O widget de edição não tem japonês: a busca dentro do editor fica em inglês.
            tea_core::i18n::Language::English | tea_core::i18n::Language::Japanese => {
                iced_code_editor::Language::English
            }
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
            diagnostics_text: None,
            inlay_hints: Vec::new(),
            inlay_text: None,
            inlay_token: 0,
            inlay_request_text: None,
            inlay_enabled: true,
            semantic_tokens: Vec::new(),
            semantic_text: None,
            semantic_token: 0,
            semantic_request_text: None,
            semantic_enabled: true,
            overlay: LspOverlayState::new(),
            hover: None,
            hover_diagnostics: None,
            diagnostics: BTreeMap::new(),
            breakpoints: BTreeSet::new(),
            exec_line: None,
            image: None,
            pdf: None,
            table: None,
            table_limit: TABLE_MAX_BYTES,
            encoding: TextEncoding::UTF8,
            file_stamp: None,
            signature_open: false,
            signature: None,
            diff_base: None,
            hunks: Vec::new(),
            diff_hash: 0,
            hunk_open: None,
            light: None,
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
        self.follow_edits();
        self.sync_table();
    }

    /// Linha e coluna do cursor, começando em 1.
    pub fn cursor(&self) -> (usize, usize) {
        let (l, c) = self.inner.cursor_position();
        (l + 1, c + 1)
    }

    /// A seleção do cursor principal: a primeira e a última linha (começando em 1) e o texto. Uma seleção que
    /// termina no começo de uma linha não conta essa linha. Sem seleção: `None`.
    pub fn selection(&self) -> Option<(usize, usize, String)> {
        let ((l0, _), (l1, c1)) = self.inner.selection_range()?;
        let text = self.inner.selected_text()?;
        let last = if c1 == 0 && l1 > l0 { l1 - 1 } else { l1 };
        Some((l0 + 1, last + 1, text))
    }

    /// Os diagnósticos (de todos os servidores) que tocam as linhas `first..=last` (começando em 1), na ordem das
    /// linhas.
    pub fn diagnostics_on(&self, first: usize, last: usize) -> Vec<&tea_lsp::Diagnostic> {
        let mut found: Vec<&tea_lsp::Diagnostic> = self
            .diagnostics
            .values()
            .flatten()
            .filter(|d| (d.range.start.line as usize + 1) <= last && (d.range.end.line as usize + 1) >= first)
            .collect();
        found.sort_by_key(|d| (d.range.start.line, d.range.start.character));
        found
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
        // No encoding da aba; um caractere que não cabe recusa o salvar (nada de `?` no lugar).
        let bytes = tea_core::encoding::encode(&text, self.encoding).map_err(|u| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                fl!(
                    "not-in-encoding",
                    ch = u.ch.to_string(),
                    line = u.line,
                    column = u.column,
                    encoding = self.encoding.label()
                ),
            )
        })?;
        std::fs::write(path, bytes)?;
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
        style.scroller_color = theme.extended_palette().background.strongest.color.scale_alpha(0.6);
        self.inner.set_theme(style);
        self.theme = theme.clone();
        self.push_inlay_hints();
        // As cores dos tokens vêm do tema do widget.
        self.push_semantic_tokens();
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
        let task = self.update_inner(message);
        self.follow_edits();
        self.sync_table();
        task
    }

    /// Relê a tabela se o texto mudou desde a última leitura (só com a tabela na tela: no texto, digitar não custa
    /// uma leitura por tecla).
    fn sync_table(&mut self) {
        let revision = self.text_revision();
        if self.table.as_ref().is_some_and(|t| t.showing && t.revision() != revision) {
            let text = self.inner.content();
            if let Some(t) = &mut self.table {
                t.reload(&text, revision);
            }
        }
    }

    /// O arquivo pode ser visto como tabela (CSV, TSV, JSONL).
    pub fn has_table(&self) -> bool {
        self.table.is_some()
    }

    /// A aba mostra a tabela.
    pub fn showing_table(&self) -> bool {
        self.table.as_ref().is_some_and(|t| t.showing)
    }

    /// Alterna entre o texto e a tabela (`table.toggle`). Do texto para a tabela, a tabela é relida; da tabela para o
    /// texto, o cursor vai para a célula escolhida.
    pub fn toggle_table(&mut self) -> Task<EditorMessage> {
        if self.showing_table() {
            return self.update(EditorMessage(Msg::Table(table_view::Msg::ShowText)));
        }
        let Some(t) = &mut self.table else { return Task::none() };
        t.showing = true;
        self.sync_table();
        // A célula onde estava o cursor do texto.
        let (line, column) = self.inner.cursor_position();
        let text = self.inner.content();
        let at = byte_at(&text, line, column);
        let Some(t) = &mut self.table else { return Task::none() };
        match t.select_at(at) {
            table_view::Action::Task(task) => task.map(|m| EditorMessage(Msg::Table(m))),
            _ => Task::none(),
        }
    }

    /// Até que tamanho (em bytes) o arquivo abre como tabela (`[table] max_mb`; 0 desliga a vista). Acima, a aba
    /// fica só no texto; abaixo, um CSV, TSV ou JSONL que não tinha a vista passa a ter.
    pub fn set_table_limit(&mut self, max: usize) {
        if self.table_limit == max {
            return;
        }
        self.table_limit = max;
        let Some(path) = self.path.clone().filter(|_| self.light.is_none() && self.image.is_none() && self.pdf.is_none()) else {
            return;
        };
        let text = self.inner.content();
        if text.len() > max {
            self.table = None;
            return;
        }
        if self.table.is_none()
            && let Some(format) = tea_table::Format::detect(&path, &text)
        {
            self.table = Some(table_view::TableView::new(format, &text, self.text_revision()));
            self.count_edits();
        }
    }

    fn update_table(&mut self, msg: table_view::Msg) -> Task<EditorMessage> {
        let size = self.inner.font_size();
        let Some(t) = &mut self.table else { return Task::none() };
        t.font_size = size;
        let wrap = |task: Task<table_view::Msg>| task.map(|m| EditorMessage(Msg::Table(m)));
        match t.update(msg) {
            table_view::Action::None => Task::none(),
            table_view::Action::Task(task) => wrap(task),
            table_view::Action::Focus(task) => {
                self.focus();
                wrap(task)
            }
            table_view::Action::Edit(edits) => {
                // As trocas são sobre o texto lido pela tabela, que é o de agora (`sync_table` roda depois de cada
                // mensagem); se não for, a edição é descartada e a tabela relida. Todas juntas: um passo só no
                // desfazer.
                let text = self.inner.content();
                if t.revision() != self.text_revision() {
                    return Task::none();
                }
                let changes: Vec<LspTextChange> = edits
                    .into_iter()
                    .map(|edit| LspTextChange {
                        range: iced_code_editor::LspRange {
                            start: lsp_position(&text, edit.range.start),
                            end: lsp_position(&text, edit.range.end),
                        },
                        text: edit.text,
                    })
                    .collect();
                self.inner.apply_lsp_text_edits(&changes);
                self.focus();
                Task::none()
            }
            table_view::Action::ShowText(at) => {
                t.showing = false;
                let text = self.inner.content();
                self.focus();
                match at {
                    Some(byte) if byte <= text.len() => {
                        let p = lsp_position(&text, byte);
                        self.goto(p.line as usize + 1, p.character as usize + 1)
                    }
                    _ => Task::none(),
                }
            }
        }
    }

    fn update_inner(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        match message.0 {
            Msg::Table(m) => self.update_table(m),
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
        if let Some(table) = self.table.as_ref().filter(|t| t.showing) {
            let focused = self.is_focused();
            let view = table
                .view(self.code_font.font, self.inner.font_size(), &self.theme)
                .map(|m| EditorMessage(Msg::Table(m)));
            return KeyGate::new(view)
                .intercept(focused, move |key| table.wants(key).then_some(EditorMessage(Msg::Table(table_view::Msg::Key(key)))))
                .shift(focused, |on| EditorMessage(Msg::Table(table_view::Msg::Shift(on))))
                .ctrl(focused, |c, shift| match (c, shift) {
                    ("c", csv) => Some(EditorMessage(Msg::Table(table_view::Msg::Copy(csv)))),
                    ("a", false) => Some(EditorMessage(Msg::Table(table_view::Msg::SelectAll))),
                    ("z", false) => Some(EditorMessage(Msg::Widget(Inner::Undo))),
                    ("y", false) | ("z", true) => Some(EditorMessage(Msg::Widget(Inner::Redo))),
                    _ => None,
                })
                .into();
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
        // As edições de antes valem para os outros servidores; a lista nova já é do texto atual.
        self.follow_edits();
        if diagnostics.is_empty() {
            self.diagnostics.remove(source);
        } else {
            self.diagnostics.insert(source.to_string(), diagnostics.to_vec());
        }
        self.diagnostics_text = (!self.diagnostics.is_empty()).then(|| (self.text_revision(), self.text()));
        self.push_diagnostics();
    }

    /// Leva os diagnósticos guardados para onde o texto editado os deixou, sem esperar o servidor (o `cargo check`
    /// do rust-analyzer só roda ao salvar). Chamado depois de cada mensagem e de cada edição feita pela IDE.
    /// Compara o texto de agora com o dos diagnósticos (e não usa os trechos que o widget manda ao servidor: são
    /// linhas inteiras reescritas, às vezes o documento todo com o fim desencontrado).
    fn follow_edits(&mut self) {
        let revision = self.text_revision();
        let stale = |t: &Option<(u64, String)>| t.as_ref().is_some_and(|(seen, _)| *seen != revision);
        if !stale(&self.diagnostics_text) && !stale(&self.inlay_text) && !stale(&self.semantic_text) {
            return;
        }
        let new = self.text();
        if let Some((_, old)) = self.diagnostics_text.take_if(|(seen, _)| *seen != revision) {
            let (range, text) = tea_lsp::diff(&old, &new);
            for d in self.diagnostics.values_mut().flatten() {
                d.range = tea_lsp::shift_range(d.range, range, &text);
            }
            self.diagnostics_text = Some((revision, new.clone()));
            self.push_diagnostics();
        }
        if let Some((_, old)) = self.inlay_text.take_if(|(seen, _)| *seen != revision) {
            let (range, text) = tea_lsp::diff(&old, &new);
            for h in &mut self.inlay_hints {
                h.position = tea_lsp::shift_range(tea_lsp::Range { start: h.position, end: h.position }, range, &text).start;
            }
            self.inlay_text = Some((revision, new.clone()));
            self.push_inlay_hints();
        }
        if let Some((_, old)) = self.semantic_text.take_if(|(seen, _)| *seen != revision) {
            let (range, text) = tea_lsp::diff(&old, &new);
            for t in &mut self.semantic_tokens {
                t.range = tea_lsp::shift_range(t.range, range, &text);
            }
            self.semantic_text = Some((revision, new));
            self.push_semantic_tokens();
        }
    }

    fn push_diagnostics(&mut self) {
        use iced_code_editor::{Diagnostic as Mark, DiagnosticSeverity};
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
                message: d.message.clone(),
                source: d.source.clone(),
            })
            .collect();
        self.inner.set_diagnostics(all);
    }

    /// Liga ou desliga os inlay hints (`[lsp] inlay_hints`, `lsp.toggle_inlay_hints`). Ligando, pede ao servidor.
    pub fn set_inlay_hints_enabled(&mut self, on: bool) {
        if self.inlay_enabled == on {
            return;
        }
        self.inlay_enabled = on;
        self.inlay_hints.clear();
        self.inlay_text = None;
        self.inlay_request_text = None;
        self.push_inlay_hints();
        if on {
            self.request_inlay_hints();
        }
    }

    /// Pede ao servidor os inlay hints do arquivo. Resposta em `tea_lsp::Event::InlayHints`, para
    /// [`Self::set_inlay_hints`]. Chamado quando o servidor termina de analisar (diagnósticos novos, o `refresh`
    /// dele) e ao abrir a aba.
    pub fn request_inlay_hints(&mut self) -> bool {
        if !self.inlay_enabled {
            return false;
        }
        let (Some(link), Some(path)) = (&self.lsp, &self.path) else { return false };
        self.inner.lsp_flush_pending_changes();
        self.inlay_token += 1;
        // O arquivo todo (o cliente limita ao fim do documento): o trecho visível muda a cada rolagem e pediria de
        // novo; os servidores respondem rápido mesmo em arquivos grandes.
        let end = tea_lsp::Position::new(u32::MAX, 0);
        link.client.inlay_hints(path, tea_lsp::Range { start: tea_lsp::Position::new(0, 0), end }, self.inlay_token);
        // A resposta vem em posições deste texto: as edições até ela chegar movem os hints.
        self.inlay_request_text = Some((self.text_revision(), self.text()));
        true
    }

    /// A resposta de [`Self::request_inlay_hints`]. Uma resposta velha (`token` de um pedido anterior) é descartada.
    pub fn set_inlay_hints(&mut self, token: u64, hints: Vec<tea_lsp::InlayHint>) {
        if token != self.inlay_token || !self.inlay_enabled {
            return;
        }
        let Some(text) = self.inlay_request_text.take() else { return };
        self.inlay_hints = hints;
        self.inlay_text = Some(text);
        if self.inlay_text.as_ref().is_some_and(|(seen, _)| *seen != self.text_revision()) {
            self.follow_edits();
        } else {
            self.push_inlay_hints();
        }
    }

    fn push_inlay_hints(&mut self) {
        let palette = self.theme.extended_palette();
        let color = palette.background.base.text.scale_alpha(0.5);
        let items = self
            .inlay_hints
            .iter()
            .map(|h| iced_code_editor::VirtualText {
                line: h.position.line as usize,
                column: h.position.character as usize,
                text: format!(
                    "{}{}{}",
                    if h.padding_left { " " } else { "" },
                    h.label,
                    if h.padding_right { " " } else { "" }
                ),
                color,
            })
            .collect();
        self.inner.set_virtual_text("inlay-hints", items);
    }

    /// Liga ou desliga os semantic tokens (`[lsp] semantic_tokens`). Ligando, pede ao servidor.
    pub fn set_semantic_tokens_enabled(&mut self, on: bool) {
        if self.semantic_enabled == on {
            return;
        }
        self.semantic_enabled = on;
        self.semantic_tokens.clear();
        self.semantic_text = None;
        self.semantic_request_text = None;
        self.push_semantic_tokens();
        if on {
            self.request_semantic_tokens();
        }
    }

    /// Pede ao servidor os semantic tokens do arquivo. Resposta em `tea_lsp::Event::SemanticTokens`, para
    /// [`Self::set_semantic_tokens`]. Pedido nos mesmos momentos que os inlay hints.
    pub fn request_semantic_tokens(&mut self) -> bool {
        if !self.semantic_enabled {
            return false;
        }
        let (Some(link), Some(path)) = (&self.lsp, &self.path) else { return false };
        self.inner.lsp_flush_pending_changes();
        self.semantic_token += 1;
        link.client.semantic_tokens(path, self.semantic_token);
        self.semantic_request_text = Some((self.text_revision(), self.text()));
        true
    }

    /// A resposta de [`Self::request_semantic_tokens`]. Uma resposta velha é descartada.
    pub fn set_semantic_tokens(&mut self, token: u64, tokens: Vec<tea_lsp::SemanticToken>) {
        if token != self.semantic_token || !self.semantic_enabled {
            return;
        }
        let Some(text) = self.semantic_request_text.take() else { return };
        self.semantic_tokens = tokens;
        self.semantic_text = Some(text);
        if self.semantic_text.as_ref().is_some_and(|(seen, _)| *seen != self.text_revision()) {
            self.follow_edits();
        } else {
            self.push_semantic_tokens();
        }
    }

    /// Pinta os tokens com a cor que o tema do widget dá ao escopo de cada um (`semantic_scopes`); um token sem
    /// escopo, ou cujo escopo o tema não pinta, fica com a cor da gramática.
    fn push_semantic_tokens(&mut self) {
        let mut colors: HashMap<&'static str, Option<iced::Color>> = HashMap::new();
        let mutable = self.inner.scope_color(MUTABLE);
        if mutable == self.inner.scope_color("variable.other.readwrite") {
            colors.insert(MUTABLE, None);
        }
        let mut styles = Vec::new();
        for t in &self.semantic_tokens {
            let (start, end) = (t.range.start, t.range.end);
            // Uma edição pode ter juntado o token a outra linha ou apagado ele: some até a resposta nova.
            if start.line != end.line || start.character >= end.character {
                continue;
            }
            let color = semantic_scopes(&t.kind, &t.modifiers)
                .iter()
                .find_map(|scope| *colors.entry(scope).or_insert_with(|| self.inner.scope_color(scope)));
            if let Some(color) = color {
                styles.push(iced_code_editor::RangeStyle {
                    line: start.line as usize,
                    start: start.character as usize,
                    end: end.character as usize,
                    color,
                });
            }
        }
        self.inner.set_range_styles("semantic-tokens", styles);
    }

    /// Sem servidor, a revisão do texto não andaria: a tabela liga um cliente que só conta as edições. Um servidor de
    /// verdade toma o lugar dele (e conta igual).
    fn count_edits(&mut self) {
        if self.table.is_none() || self.lsp.is_some() {
            return;
        }
        let Some(path) = self.path.clone().filter(|_| self.light.is_none()) else { return };
        let uri = tea_lsp::uri::path_to_uri(&path);
        let counter = lsp::Counter { revision: self.revision.clone() };
        self.inner.attach_lsp(Box::new(counter), LspDocument::new(uri, "plaintext"));
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

    /// O arquivo no disco é o texto desta aba, gravado por outro programa (um agente, o git): avisa o servidor como
    /// um save. Sem isso, o rust-analyzer não roda o `cargo check` de novo e continua mostrando os erros de antes.
    pub fn lsp_saved_elsewhere(&mut self) {
        if self.lsp.is_some() {
            self.inner.lsp_did_save();
        }
    }

    pub fn detach_lsp(&mut self) {
        if self.lsp.take().is_some() {
            self.inner.detach_lsp();
            self.count_edits();
        }
        self.diagnostics.clear();
        self.diagnostics_text = None;
        self.inner.set_diagnostics(Vec::new());
        self.inlay_hints.clear();
        self.inlay_text = None;
        self.push_inlay_hints();
        self.semantic_tokens.clear();
        self.semantic_text = None;
        self.push_semantic_tokens();
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
                border: iced::Border { color: p.primary.weak.color, width: 1.0, radius: tea_core::style::RADIUS_CONTROL.into() },
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
                border: iced::Border { color: p.primary.weak.color, width: 1.0, radius: tea_core::style::RADIUS_CONTROL.into() },
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

    /// Os diagnósticos sob `point`, o mais grave primeiro.
    fn diagnostics_at(&self, point: iced::Point) -> Vec<&tea_lsp::Diagnostic> {
        let Some(p) = self.inner.lsp_position_at_point(point) else { return Vec::new() };
        let position = (p.line, p.character);
        let mut found: Vec<_> = self
            .diagnostics
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
            .collect();
        found.sort_by_key(|d| d.severity);
        found
    }

    /// O texto do hover para diagnósticos: a mensagem de cada um, com a origem (`rustc`, `clippy`) quando houver.
    fn diagnostics_hover(diagnostics: &[&tea_lsp::Diagnostic]) -> String {
        diagnostics
            .iter()
            .map(|d| match &d.source {
                Some(source) => format!("{} *({source})*", d.message),
                None => d.message.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn tick(&mut self, now: Instant) {
        let Some(h) = &self.hover else { return };
        if !h.is_due(now) {
            return;
        }
        let point = h.point;
        self.hover = None;
        let Some((_, anchor)) = self.inner.lsp_hover_anchor_at_point(point) else { return };
        let diagnostics = self.diagnostics_at(point);
        // Os diagnósticos aparecem já; o hover do servidor (o tipo, a documentação) vem embaixo quando chegar.
        self.hover_diagnostics = (!diagnostics.is_empty()).then(|| Self::diagnostics_hover(&diagnostics));
        let requested = self.lsp.is_some() && self.inner.lsp_request_hover_at(point);
        if let Some(text) = self.hover_diagnostics.clone() {
            self.overlay.set_hover_position(anchor);
            self.overlay.show_hover(text);
        } else if requested {
            self.overlay.set_hover_position(anchor);
        }
    }

    /// A resposta do servidor ao hover. Com diagnósticos no ponto, ela vai embaixo deles.
    pub fn show_hover(&mut self, text: String) {
        if self.overlay.hover_position.is_none() {
            return;
        }
        match &self.hover_diagnostics {
            Some(diagnostics) => self.overlay.show_hover(format!("{diagnostics}\n\n---\n\n{text}")),
            None => self.overlay.show_hover(text),
        }
    }

    /// Hover no cursor (comando, sem mouse).
    pub fn request_hover(&mut self) {
        self.hover_diagnostics = None;
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

    /// Pede as ações de código para a seleção, ou para o cursor sem seleção. Resposta em
    /// `tea_lsp::Event::CodeActions`.
    pub fn request_code_actions(&mut self) -> bool {
        let (Some(link), Some(path)) = (&self.lsp, &self.path) else { return false };
        self.inner.lsp_flush_pending_changes();
        let pos = |(l, c): (usize, usize)| tea_lsp::Position::new(l as u32, c as u32);
        let range = match self.inner.selection_range() {
            Some((start, end)) => tea_lsp::Range { start: pos(start), end: pos(end) },
            None => {
                let at = pos(self.inner.cursor_position());
                tea_lsp::Range { start: at, end: at }
            }
        };
        link.client.code_actions(path, range);
        true
    }

    /// Manda já ao servidor as mudanças que esperavam a pausa na digitação (antes de executar um comando dele).
    pub fn flush_lsp_changes(&mut self) {
        if self.lsp.is_some() {
            self.inner.lsp_flush_pending_changes();
        }
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
        self.follow_edits();
        self.sync_table();
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
        self.follow_edits();
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
/// O erro de leitura de um arquivo que não virou texto, para o aviso.
fn decode_error(e: tea_core::encoding::DecodeError) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        match e {
            tea_core::encoding::DecodeError::Binary => fl!("binary-file"),
            tea_core::encoding::DecodeError::Invalid(name) => fl!("not-valid-in", encoding = name),
        },
    )
}

/// O byte da linha e coluna (em caracteres, começando em 0) do texto; além do fim da linha, o fim dela.
fn byte_at(text: &str, line: usize, column: usize) -> usize {
    let start = if line == 0 {
        0
    } else {
        match text.match_indices('\n').nth(line - 1) {
            Some((i, _)) => i + 1,
            None => return text.len(),
        }
    };
    let rest = &text[start..];
    let end = rest.find('\n').unwrap_or(rest.len());
    start + rest[..end].char_indices().nth(column).map_or(end, |(i, _)| i)
}

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

/// O escopo de uma variável mutável. Só vale quando o tema dá a ele uma cor própria, diferente da de uma variável
/// comum (o `base16-ocean` não dá: pinta todo `variable.*` igual).
const MUTABLE: &str = "variable.other.mutable";

/// Os escopos TextMate de um semantic token, em ordem de preferência (RFC 0019): o primeiro que o tema pinta
/// dá a cor. Vazio para o que a gramática já pinta bem (comentários, strings, números, operadores) e para tipos que
/// não têm um escopo claro.
fn semantic_scopes(kind: &str, modifiers: &[String]) -> &'static [&'static str] {
    let mutable = modifiers.iter().any(|m| m == "mutable");
    match kind {
        "function" | "method" => &["entity.name.function"],
        "macro" => &["entity.name.function.macro", "entity.name.function"],
        "struct" | "class" | "enum" | "interface" | "type" | "typeParameter" | "typeAlias" | "union" => {
            &["entity.name.type"]
        }
        "parameter" if mutable => &[MUTABLE, "variable.parameter"],
        "parameter" => &["variable.parameter"],
        // Uma variável comum fica com a cor da gramática: os temas pintam todo `variable.*` da mesma cor, e pintar
        // todas as variáveis do arquivo é ruído, não informação.
        "variable" if mutable => &[MUTABLE],
        "property" => &["variable.other.property"],
        "enumMember" => &["variable.other.enummember"],
        "namespace" => &["entity.name.namespace"],
        "keyword" | "selfKeyword" | "selfTypeKeyword" => &["keyword"],
        "lifetime" => &["storage.modifier.lifetime"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn compound_pack_extension_picks_its_grammar() {
        assert_eq!(syntax_for(Some(Path::new("docs/README.mbt.md"))), "mbt.md");
        assert_eq!(syntax_for(Some(Path::new("notes.md"))), "md");
    }

    /// `[table] max_mb`: acima do limite a aba fica só no texto; com um limite maior, a vista volta.
    #[test]
    fn table_limit_turns_the_view_off_and_on() {
        let mut ed = Editor::from_text(Path::new("/p/a.csv"), "a,b\n1,2\n");
        ed.set_table_limit(4);
        assert!(!ed.has_table());
        ed.set_table_limit(TABLE_MAX_BYTES);
        assert!(ed.has_table() && ed.showing_table());
        assert_eq!(byte_at("ab\ncé\n", 1, 1), 4);
        assert_eq!(byte_at("ab\ncé\n", 1, 9), 6, "além do fim da linha: o fim");
        assert_eq!(byte_at("ab\n", 5, 0), 3);
    }

    /// Sem servidor LSP (um CSV não tem), a célula editada aparece na tabela e no texto.
    #[test]
    fn table_follows_its_own_edit_without_a_server() {
        use table_view::Msg as T;
        let mut ed = Editor::from_text(Path::new("/p/a.csv"), "nome,cidade\nana,Recife\n");
        assert!(ed.showing_table());
        for m in [
            T::StartEdit(tea_table::RowRef::Row(0), 1),
            T::EditInput("Rio, RJ".into()),
            T::CommitEdit,
        ] {
            let _ = ed.update(EditorMessage(Msg::Table(m)));
        }
        assert_eq!(ed.text(), "nome,cidade\nana,\"Rio, RJ\"\n");
        let t = ed.table.as_ref().unwrap();
        assert_eq!(t.value(tea_table::RowRef::Row(0), 1).as_deref(), Some("Rio, RJ"), "a tabela releu o texto");
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

    /// O sublinhado acompanha o texto editado antes de o servidor mandar a lista nova (o `cargo check` do
    /// rust-analyzer só roda ao salvar): Enter acima do erro e um `use` aplicado no topo descem ele junto.
    #[test]
    fn diagnostics_follow_the_text_until_the_server_answers() {
        use tea_lsp::{Diagnostic, Position, Range, Severity};
        let mut editor = Editor::from_text(&tmp("segue.rs"), "fn main() {\n    let t = soma(2, 3);\n}\n");
        // Um servidor que sai logo (o próprio binário de teste): a ponte anota as edições mesmo assim.
        let server = tea_lsp::ServerConfig {
            name: "falso".into(),
            command: std::env::current_exe().unwrap().to_string_lossy().into_owned(),
            args: vec!["--list".into()],
            languages: vec![("rs".into(), "rust".into())],
        };
        let client = tea_lsp::Client::start(&server, &std::env::temp_dir(), std::sync::Arc::new(|_| {})).unwrap();
        editor.attach_lsp(client.clone(), "rust");
        let soma = Range { start: Position::new(1, 12), end: Position::new(1, 16) };
        let d = Diagnostic { range: soma, severity: Severity::Error, message: "soma?".into(), source: None, code: None };
        editor.set_diagnostics("rustc", &[d]);
        let lines = |editor: &Editor| {
            let w = editor.inner.diagnostics().iter().map(|d| (d.range.start.line, d.range.start.character)).collect::<Vec<_>>();
            (w, editor.diagnostics_on(1, 99).iter().map(|d| d.range.start.line).collect::<Vec<_>>())
        };

        // Enter no começo do arquivo (o cursor começa lá).
        editor.focus();
        let _ = editor.update(EditorMessage(Msg::Widget(Inner::Enter)));
        assert_eq!(lines(&editor), (vec![(2, 12)], vec![2]));

        // Uma ação de código que põe o `use` no topo.
        let use_line = tea_lsp::TextEdit {
            range: Range { start: Position::new(0, 0), end: Position::new(0, 0) },
            new_text: "use std::collections::HashMap;\n\n".into(),
        };
        editor.apply_formatting(&[use_line]);
        assert_eq!(lines(&editor), (vec![(4, 12)], vec![4]));

        // A lista nova do servidor substitui (já é do texto atual).
        editor.set_diagnostics("rustc", &[]);
        assert!(lines(&editor).0.is_empty());
        client.kill_now();
    }

    #[test]
    fn inlay_hints_reach_the_widget_and_follow_the_text() {
        use tea_lsp::{InlayHint, InlayHintKind, Position};
        let mut editor = Editor::from_text(&tmp("hints.rs"), "fn main() {\n    let t = soma(2, 3);\n}\n");
        let server = tea_lsp::ServerConfig {
            name: "falso".into(),
            command: std::env::current_exe().unwrap().to_string_lossy().into_owned(),
            args: vec!["--list".into()],
            languages: vec![("rs".into(), "rust".into())],
        };
        let client = tea_lsp::Client::start(&server, &std::env::temp_dir(), std::sync::Arc::new(|_| {})).unwrap();
        editor.attach_lsp(client.clone(), "rust");
        let hint = |line, character, label: &str, kind| InlayHint {
            position: Position::new(line, character),
            label: label.into(),
            kind,
            padding_left: false,
            padding_right: kind == InlayHintKind::Parameter,
        };
        let shown = |editor: &Editor| {
            editor.inner.virtual_text("inlay-hints").iter().map(|v| (v.line, v.column, v.text.clone())).collect::<Vec<_>>()
        };

        assert!(editor.request_inlay_hints());
        let token = editor.inlay_token;
        // Enter no começo antes da resposta: ela vem nas posições do texto do pedido.
        editor.focus();
        let _ = editor.update(EditorMessage(Msg::Widget(Inner::Enter)));
        editor.set_inlay_hints(token, vec![hint(1, 9, ": i32", InlayHintKind::Type), hint(1, 17, "a:", InlayHintKind::Parameter)]);
        assert_eq!(shown(&editor), vec![(2, 9, ": i32".into()), (2, 17, "a: ".into())]);
        assert_eq!(editor.text(), "\nfn main() {\n    let t = soma(2, 3);\n}\n", "nunca no texto");

        // Outro Enter: andam junto.
        let _ = editor.update(EditorMessage(Msg::Widget(Inner::Enter)));
        assert_eq!(shown(&editor)[0].0, 3);

        // Uma resposta velha não substitui a nova.
        assert!(editor.request_inlay_hints());
        editor.set_inlay_hints(token, Vec::new());
        assert_eq!(shown(&editor).len(), 2);

        editor.set_inlay_hints_enabled(false);
        assert!(shown(&editor).is_empty());
        assert!(!editor.request_inlay_hints());
        client.kill_now();
    }

    #[test]
    fn semantic_tokens_color_ranges_and_follow_the_text() {
        use tea_lsp::{Position, Range, SemanticToken};
        let mut editor = Editor::from_text(&tmp("semantic.rs"), "fn soma(a: i32) -> i32 {\n    let mut t = a;\n    t\n}\n");
        let server = tea_lsp::ServerConfig {
            name: "falso".into(),
            command: std::env::current_exe().unwrap().to_string_lossy().into_owned(),
            args: vec!["--list".into()],
            languages: vec![("rs".into(), "rust".into())],
        };
        let client = tea_lsp::Client::start(&server, &std::env::temp_dir(), std::sync::Arc::new(|_| {})).unwrap();
        editor.attach_lsp(client.clone(), "rust");
        let token = |line, start, end, kind: &str, modifiers: &[&str]| SemanticToken {
            range: Range { start: Position::new(line, start), end: Position::new(line, end) },
            kind: kind.into(),
            modifiers: modifiers.iter().map(|m| m.to_string()).collect(),
        };
        let shown = |editor: &Editor| {
            editor.inner.range_styles("semantic-tokens").iter().map(|s| (s.line, s.start, s.end)).collect::<Vec<_>>()
        };
        let parameter = editor.inner.scope_color("variable.parameter").expect("o tema pinta parâmetros");

        assert!(editor.request_semantic_tokens());
        let req = editor.semantic_token;
        // Enter no começo antes da resposta: ela vem nas posições do texto do pedido.
        editor.focus();
        let _ = editor.update(EditorMessage(Msg::Widget(Inner::Enter)));
        editor.set_semantic_tokens(
            req,
            vec![
                token(0, 8, 9, "parameter", &["declaration"]),
                token(1, 12, 13, "variable", &["declaration", "mutable"]),
                // A gramática já pinta: fica de fora.
                token(0, 0, 2, "comment", &[]),
            ],
        );
        let styles = shown(&editor);
        assert!(styles.contains(&(1, 8, 9)), "{styles:?}");
        // O tema padrão não tem uma cor própria para variável mutável: ela fica com a da gramática.
        assert_eq!(editor.inner.scope_color("variable.other.mutable"), editor.inner.scope_color("variable.other.readwrite"));
        assert!(!styles.contains(&(2, 12, 13)), "{styles:?}");
        assert!(!styles.iter().any(|s| s.1 == 0), "sem estilo para o comentário: {styles:?}");
        assert_eq!(editor.inner.range_styles("semantic-tokens")[0].color, parameter);

        // Outro Enter: andam junto.
        let _ = editor.update(EditorMessage(Msg::Widget(Inner::Enter)));
        assert!(shown(&editor).contains(&(2, 8, 9)), "{:?}", shown(&editor));

        // Uma resposta velha não substitui a nova.
        assert!(editor.request_semantic_tokens());
        editor.set_semantic_tokens(req, Vec::new());
        assert!(!shown(&editor).is_empty());

        editor.set_semantic_tokens_enabled(false);
        assert!(shown(&editor).is_empty());
        assert!(!editor.request_semantic_tokens());
        client.kill_now();
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
        ed.set_diagnostics("clippy", &[Diagnostic { source: Some("clippy".into()), ..diagnostic(Severity::Warning, "aviso") }]);
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
        // Os dois no mesmo ponto, o mais grave primeiro, com a origem quando o servidor diz.
        let found = ed.diagnostics_at(point);
        assert_eq!(found.iter().map(|d| d.message.as_str()).collect::<Vec<_>>(), ["erro", "aviso"]);
        let text = Editor::diagnostics_hover(&found);
        assert_eq!(text, "erro\n\naviso *(clippy)*");
        assert!(ed.diagnostics_at(iced::Point::new(marks::GUTTER_WIDTH + 5.0, 1.0)).is_empty());

        // O hover do servidor vai embaixo deles.
        ed.overlay.set_hover_position(point);
        ed.hover_diagnostics = Some(text);
        ed.show_hover("`answer: i32`".into());
        assert_eq!(ed.overlay.hover_text.as_deref(), Some("erro\n\naviso *(clippy)*\n\n---\n\n`answer: i32`"));
        // Pedido pelo comando (sem mouse): só o do servidor.
        ed.request_hover();
        ed.overlay.set_hover_position(point);
        ed.show_hover("`answer: i32`".into());
        assert_eq!(ed.overlay.hover_text.as_deref(), Some("`answer: i32`"));

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
    fn binary_is_still_an_error() {
        let path = tmp("bin.dat");
        std::fs::write(&path, [0x89, b'P', b'N', b'G', 0x00, 0x00, 0xff]).unwrap();
        assert!(Editor::open(&path).is_err());
        // Um BOM de UTF-16 com um byte sobrando também não é texto.
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        assert!(Editor::open(&path).is_err());
    }

    /// RFC 0021: um Latin-1 abre, salva igual byte a byte, e um caractere que não cabe recusa o salvar.
    #[test]
    fn legacy_encoding_opens_and_saves_the_same_bytes() {
        let path = tmp("latin1.txt");
        let bytes = b"a\xe7\xe3o\r\nfim\r\n";
        std::fs::write(&path, bytes).unwrap();
        let mut ed = Editor::open(&path).unwrap();
        assert_eq!(ed.text(), "ação\nfim\n");
        assert_eq!(ed.encoding().label(), "windows-1252");
        ed.save().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes, "o mesmo arquivo");
        ed.replace_text("ação 🙂\n");
        let err = ed.save().unwrap_err();
        assert!(err.to_string().contains("🙂"), "{err}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes, "recusado: o arquivo não mudou");
        ed.save_with_encoding(TextEncoding::UTF8).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), "ação 🙂\r\n".as_bytes());
        assert!(ed.encoding().is_utf8());
        // A sequência do teste manual: o emoji recusado, tirado, e então Salvar com UTF-8.
        std::fs::write(&path, bytes).unwrap();
        let mut ed = Editor::open(&path).unwrap();
        ed.replace_text("ação 🙂\nfim\n");
        assert!(ed.save().is_err());
        ed.replace_text("ação\nfim\n");
        ed.save_with_encoding(TextEncoding::UTF8).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), "ação\r\nfim\r\n".as_bytes());
        // UTF-8 com BOM mantém o BOM.
        std::fs::write(&path, b"\xef\xbb\xbfoi\n").unwrap();
        let mut ed = Editor::open(&path).unwrap();
        assert_eq!(ed.text(), "oi\n");
        ed.save().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"\xef\xbb\xbfoi\n");
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
