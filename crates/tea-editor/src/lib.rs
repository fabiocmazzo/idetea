//! Fachada sobre o widget de edição.
//!
//! O resto da IDE só usa os tipos deste crate. Trocar o `iced-code-editor`
//! por outro widget deve mexer só aqui.

use std::io;
use std::path::{Path, PathBuf};

use iced::{Element, Task};
use iced_code_editor::CodeEditor;

/// Mensagem interna do editor. Opaca para quem está de fora.
#[derive(Debug, Clone)]
pub struct EditorMessage(iced_code_editor::Message);

pub struct Editor {
    path: Option<PathBuf>,
    inner: CodeEditor,
}

impl Editor {
    pub fn scratch(text: &str) -> Self {
        Self { path: None, inner: CodeEditor::new(text, "txt") }
    }

    pub fn open(path: &Path) -> io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_text(path, &text))
    }

    /// Cria o editor a partir de um texto já lido (útil para medir sem I/O).
    pub fn from_text(path: &Path, text: &str) -> Self {
        let syntax = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");
        let mut inner = CodeEditor::new(text, syntax);
        // A command palette será da IDE, não do widget.
        inner.set_command_palette_enabled(false);
        Self { path: Some(path.to_path_buf()), inner }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn title(&self) -> String {
        let name = self
            .path
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "sem título".into());
        if self.is_modified() { format!("● {name}") } else { name }
    }

    pub fn is_modified(&self) -> bool {
        self.inner.is_modified()
    }

    pub fn text(&self) -> String {
        self.inner.content()
    }

    /// Linha e coluna do cursor, começando em 1.
    pub fn cursor(&self) -> (usize, usize) {
        let (l, c) = self.inner.cursor_position();
        (l + 1, c + 1)
    }

    pub fn syntax_name(&self) -> &'static str {
        self.inner.syntax_name()
    }

    pub fn save(&mut self) -> io::Result<()> {
        let path = self
            .path
            .as_deref()
            .ok_or_else(|| io::Error::other("arquivo sem caminho"))?;
        std::fs::write(path, self.inner.content())?;
        self.inner.mark_saved();
        Ok(())
    }

    /// Dá foco total ao editor. Só `request_focus` não basta: sem
    /// `CanvasFocusGained` o widget ignora o teclado até levar um clique.
    pub fn focus(&mut self) {
        self.inner.request_focus();
        let _ = self.inner.update(&iced_code_editor::Message::CanvasFocusGained);
    }

    pub fn lose_focus(&mut self) {
        self.inner.lose_focus();
    }

    pub fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        self.inner.update(&message.0).map(EditorMessage)
    }

    pub fn view(&self) -> Element<'_, EditorMessage> {
        self.inner.view().map(EditorMessage)
    }
}
