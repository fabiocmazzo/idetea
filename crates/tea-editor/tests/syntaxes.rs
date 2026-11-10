//! Num processo à parte: o conjunto de sintaxes do editor é montado uma vez por processo.

#[test]
fn embedded_and_user_syntaxes_register_and_bad_ones_are_reported() {
    let dir = std::env::temp_dir().join(format!("idetea-user-syntaxes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let good = "name: Tiny\nfile_extensions: [tiny]\nscope: source.tiny\ncontexts:\n  main:\n    - match: '\\bok\\b'\n      scope: keyword.tiny\n";
    std::fs::write(dir.join("a-ruim.sublime-syntax"), "name: [aberto").unwrap();
    std::fs::write(dir.join("b-boa.sublime-syntax"), good).unwrap();
    let user_hare = "name: Hare do usuário\nfile_extensions: [ha]\nscope: source.hare\ncontexts:\n  main: []\n";
    std::fs::write(dir.join("c-hare.sublime-syntax"), user_hare).unwrap();

    tea_editor::syntaxes::register(Some(&dir));
    let problems = tea_editor::syntaxes::problems();
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].0.ends_with("a-ruim.sublime-syntax"));

    // A primeira aba monta o conjunto (com as registradas) e colore.
    let _editor = iced_code_editor::CodeEditor::new("pub fn main() void { return (<p>oi</p>); }", "zx");
    let name = |file: &str| iced_code_editor::CodeEditor::new("", file).syntax_name();
    assert_eq!(name("mbt"), "MoonBit");
    assert_eq!(name("mbti"), "MoonBit");
    assert_eq!(name("mbt.md"), "MoonBit Markdown");
    assert_eq!(name("ha"), "Hare do usuário", "a sintaxe do usuário substitui a do pack");
    let _ = std::fs::remove_dir_all(&dir);
}
