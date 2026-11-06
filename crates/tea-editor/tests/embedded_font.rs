//! A largura do caractere que o widget mede tem que ser a da fonte embutida, e não a de uma fonte
//! substituta. Arquivo à parte: roda num processo só dele, com a fonte ainda não carregada (como no
//! começo da IDE).

use iced::Font;
use iced_code_editor::CodeEditor;

fn measured() -> f32 {
    let mut editor = CodeEditor::new("fn main() {}", "rs");
    editor.set_font(Font::with_name(tea_core::fonts::DEFAULT_FAMILY));
    editor.set_font_size(14.0, false);
    editor.char_width()
}

#[test]
fn width_is_measured_with_the_embedded_font() {
    let before = measured();
    tea_core::fonts::load_embedded();
    let after = measured();
    eprintln!("antes de carregar: {before}, depois: {after}");
    // JetBrains Mono: avanço de 600/1000 em.
    assert!((after - 14.0 * 0.6).abs() < 0.01, "{after}");
}
