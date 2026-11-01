//! Mede quadros de verdade (layout, desenho e texto, num renderizador sem janela) com arquivos grandes.
//!
//! `cargo run --release -p tea-editor --example large_render -- [rust|json|js|todos] [bytes]`
//!
//! O estado da tela (a rolagem) fica de um quadro para o outro, como no app. `QUADROS=pasta` grava o último
//! quadro de cada caso em PNG, para conferir o que foi desenhado. Um quadro acima de 1 s é marcado.

use std::path::Path;
use std::time::{Duration, Instant};

use iced::advanced::renderer::Headless;
use iced::advanced::{clipboard, renderer};
use iced::mouse::{self, ScrollDelta};
use iced::theme::Base;
use iced::{Event, Size, Theme};
use iced_runtime::user_interface::{Cache, UserInterface};
use tea_editor::{Editor, EditorMessage};

const SIZE: Size = Size::new(1200.0, 800.0);

fn rust(bytes: usize) -> String {
    let mut s = String::with_capacity(bytes + 200);
    let mut n = 0;
    while s.len() < bytes {
        s.push_str(&format!("/// Função gerada número {n}.\npub fn func_{n}(x: u64) -> u64 {{\n"));
        s.push_str("    let mut acc = x; // comentário\n");
        s.push_str("    for i in 0..10 { acc = acc.wrapping_mul(31).wrapping_add(i); }\n");
        s.push_str("    if acc % 2 == 0 { acc / 2 } else { \"ímpar\".len() as u64 }\n}\n\n");
        n += 1;
    }
    s
}

/// JSON minificado: tudo numa linha.
fn json_line(bytes: usize) -> String {
    let mut s = String::with_capacity(bytes + 200);
    s.push('[');
    let mut n = 0;
    while s.len() < bytes {
        s.push_str(&format!("{{\"id\":{n},\"nome\":\"item {n}\",\"ativo\":true,\"tags\":[\"a\",\"b\"],\"cor\":\"#ff8800\"}},"));
        n += 1;
    }
    s.push_str("{}]\n");
    s
}

/// JavaScript minificado: uma linha gigante entre linhas normais.
fn js_line(bytes: usize) -> String {
    let mut s = String::from("// bundle\n\"use strict\";");
    let mut n = 0;
    while s.len() < bytes {
        s.push_str(&format!("function f{n}(a,b){{var c=a+b*{n};if(c>10){{return c-1}}return \"x\"+c}};"));
        n += 1;
    }
    s.push_str("\nconsole.log(1);\n");
    s
}

/// A tela: o renderizador e o estado dos widgets (rolagem, foco) entre um quadro e outro.
struct Screen {
    renderer: iced::Renderer,
    cache: Option<Cache>,
    cursor: mouse::Cursor,
}

impl Screen {
    fn new() -> Self {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
            None,
        ))
        .expect("renderizador sem janela");
        let cursor = mouse::Cursor::Available(iced::Point::new(SIZE.width / 2.0, SIZE.height / 2.0));
        Self { renderer, cache: None, cursor }
    }

    /// Passa os eventos, desenha um quadro e devolve o tempo e as mensagens. `png`: grava o quadro.
    fn frame(&mut self, editor: &Editor, events: &[Event], png: Option<&Path>) -> (Duration, Vec<EditorMessage>) {
        let t = Instant::now();
        let theme = Theme::TokyoNight;
        let mut ui = UserInterface::build(editor.view(), SIZE, self.cache.take().unwrap_or_default(), &mut self.renderer);
        let mut msgs = Vec::new();
        let redraw = Event::Window(iced::window::Event::RedrawRequested(Instant::now()));
        let _ = ui.update(events, self.cursor, &mut self.renderer, &mut clipboard::Null, &mut msgs);
        let _ = ui.update(&[redraw], self.cursor, &mut self.renderer, &mut clipboard::Null, &mut msgs);
        let base = theme.base();
        ui.draw(&mut self.renderer, &theme, &renderer::Style { text_color: base.text_color }, self.cursor);
        self.cache = Some(ui.into_cache());
        let px = Size::new(SIZE.width as u32, SIZE.height as u32);
        let rgba = self.renderer.screenshot(px, 1.0, base.background_color);
        let elapsed = t.elapsed();
        if let Some(png) = png {
            save_png(png, px, &rgba);
        }
        (elapsed, msgs)
    }
}

fn save_png(path: &Path, size: Size<u32>, rgba: &[u8]) {
    let file = std::fs::File::create(path).expect("criar png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), size.width, size.height);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().and_then(|mut w| w.write_image_data(rgba)).expect("gravar png");
}

fn apply(editor: &mut Editor, msgs: Vec<EditorMessage>) -> Duration {
    let t = Instant::now();
    for m in msgs {
        let _ = editor.update(m);
    }
    t.elapsed()
}

fn wheel(lines: f32) -> Event {
    Event::Mouse(mouse::Event::WheelScrolled { delta: ScrollDelta::Lines { x: 0.0, y: -lines } })
}

fn run(name: &str, ext: &str, text: &str, wrap: bool) {
    println!(
        "\n== {name}: {:.1} MB, {} linhas, linha maior {} KB, quebra de linha {}",
        text.len() as f64 / 1e6,
        text.lines().count(),
        text.lines().map(str::len).max().unwrap_or(0) / 1000,
        if wrap { "ligada" } else { "desligada" }
    );
    let t = Instant::now();
    let mut ed = Editor::from_text(Path::new(&format!("/tmp/grande.{ext}")), text);
    ed.set_options(&tea_editor::Options { wrap, ..Default::default() });
    ed.focus();
    let report = |label: &str, d: Duration| {
        let mark = if d > Duration::from_secs(1) { "  <-- TRAVA" } else { "" };
        println!("{label:<44} {d:>10.2?}{mark}");
    };
    report(if ed.is_light() { "abrir (modo leve)" } else { "abrir" }, t.elapsed());

    let mut screen = Screen::new();
    report("1º quadro", screen.frame(&ed, &[], None).0);
    report("2º quadro (cache)", screen.frame(&ed, &[], None).0);

    // Rolar até o meio e até o fim com um salto grande da rodinha (em linhas da tela: uma linha gigante
    // quebrada vira milhares, ~100 caracteres cada).
    let total = text.lines().map(|l| l.len() / 100 + 1).sum::<usize>() as f32;
    for (label, lines) in [("rolar até o meio", total / 2.0), ("rolar até o fim", total)] {
        let (d, msgs) = screen.frame(&ed, &[wheel(lines)], None);
        report(&format!("{label}: evento + update"), d + apply(&mut ed, msgs));
        let mut worst = Duration::ZERO;
        let start = Instant::now();
        for _ in 0..5 {
            worst = worst.max(screen.frame(&ed, &[], None).0);
        }
        report(&format!("{label}: pior de 5 quadros (total {:.1?})", start.elapsed()), worst);
    }

    // Subir de pouco em pouco (o uso normal).
    let mut worst = Duration::ZERO;
    for _ in 0..10 {
        let (d, msgs) = screen.frame(&ed, &[wheel(-3.0)], None);
        worst = worst.max(d + apply(&mut ed, msgs));
    }
    report("rolar 3 linhas: pior de 10", worst);

    // Digitar: clica no meio da tela e escreve.
    let click: Vec<Event> = iced_test::simulator::click().collect();
    let (_, msgs) = screen.frame(&ed, &click, None);
    apply(&mut ed, msgs);
    let mut worst = Duration::ZERO;
    let start = Instant::now();
    for c in "abc12".chars() {
        let keys: Vec<Event> = iced_test::simulator::typewrite(&c.to_string()).collect();
        let (d, msgs) = screen.frame(&ed, &keys, None);
        worst = worst.max(d + apply(&mut ed, msgs));
    }
    report(&format!("digitar 5 letras: pior quadro (total {:.1?})", start.elapsed()), worst);
    println!("  modificado: {}", ed.is_modified());

    let png = std::env::var_os("QUADROS").map(|dir| {
        Path::new(&dir).join(format!("{name}-{}.png", if wrap { "quebra" } else { "sem-quebra" }))
    });
    screen.frame(&ed, &[], png.as_deref());

    let t = Instant::now();
    ed.refresh_diff();
    let _ = ed.text();
    report("texto inteiro (salvar, marcas do git)", t.elapsed());
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_else(|| "todos".into());
    // Tamanho em bytes (opcional): muda o do caso escolhido.
    let size = |default: usize| std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(default);
    let all = which == "todos";
    run("pequeno (referência)", "rs", &rust(4_000), true);
    if all || which == "rust" {
        let t = rust(size(50_000_000));
        run("Rust 50 MB", "rs", &t, true);
        run("Rust 50 MB", "rs", &t, false);
    }
    if all || which == "json" {
        let t = json_line(size(10_000_000));
        run("JSON numa linha", "json", &t, true);
        run("JSON numa linha", "json", &t, false);
    }
    if all || which == "js" {
        let t = js_line(size(1_000_000));
        run("JS minificado", "js", &t, true);
        run("JS minificado", "js", &t, false);
    }
}
