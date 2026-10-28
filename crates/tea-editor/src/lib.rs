//! Fachada sobre o widget de edição.
//!
//! O resto da IDE só usa os tipos deste crate. Trocar o `iced-code-editor`
//! por outro widget deve mexer só aqui.

use std::io;
use std::path::{Path, PathBuf};

use iced::{Element, Task, Theme};
use iced_code_editor::{CodeEditor, Message as Inner, theme};

/// Mensagem interna do editor. Opaca para quem está de fora.
#[derive(Debug, Clone)]
pub struct EditorMessage(Inner);

impl EditorMessage {
    /// O editor pediu para salvar (`Ctrl+S` com foco nele, ou `:w` no modo vim).
    pub fn is_save_request(&self) -> bool {
        matches!(self.0, Inner::WriteRequested)
    }
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
}

impl Editor {
    pub fn scratch(text: &str) -> Self {
        Self::build(None, text, false)
    }

    /// Abre um arquivo. Se ele não existir, abre vazio e cria ao salvar.
    pub fn open(path: &Path) -> io::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => {
                let text = String::from_utf8(bytes).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "arquivo não é UTF-8 (ainda não suportado)")
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
        inner.set_language(iced_code_editor::Language::PortugueseBR);
        Self { path: path.map(Path::to_path_buf), inner, line_ending, is_new }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn file_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "sem título".into())
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
        self.inner.syntax_name()
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn save(&mut self) -> io::Result<()> {
        let path = self.path.clone().ok_or_else(|| io::Error::other("arquivo sem caminho"))?;
        self.write_to(&path)
    }

    /// Salva em outro caminho e passa a editar esse arquivo.
    pub fn save_as(&mut self, path: &Path) -> io::Result<()> {
        self.write_to(path)?;
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
        Ok(())
    }

    /// O arquivo foi renomeado ou movido por fora (ex.: árvore de arquivos).
    pub fn set_path(&mut self, path: PathBuf) {
        self.inner.set_syntax(&syntax_for(Some(&path)));
        self.path = Some(path);
    }

    pub fn is_new_file(&self) -> bool {
        self.is_new
    }

    pub fn set_theme(&mut self, theme: &Theme) {
        self.inner.set_theme(theme::from_iced_theme(theme));
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

    pub fn run(&mut self, action: EditorAction) -> Task<EditorMessage> {
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
        self.inner.update(&msg).map(EditorMessage)
    }

    pub fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        self.inner.update(&message.0).map(EditorMessage)
    }

    pub fn view(&self) -> Element<'_, EditorMessage> {
        self.inner.view().map(EditorMessage)
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
