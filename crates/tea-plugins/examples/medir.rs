//! Mede o custo de um plugin: `cargo run --release -p tea-plugins --example medir -- <plugin.wasm>`.
//! Compilar o `.wasm` na hora, guardar o compilado e reabrir, e uma chamada.

use std::path::Path;
use std::time::Instant;

use tea_plugins::runtime::{Instance, UiAction};

fn main() {
    let wasm = std::env::args().nth(1).expect("caminho do .wasm");
    let project = std::env::temp_dir();
    for i in 0..3 {
        let t = Instant::now();
        let mut p = Instance::load(Path::new(&wasm), &project).unwrap();
        let load = t.elapsed();
        let t = Instant::now();
        p.activate().unwrap();
        let activate = t.elapsed();
        let click = UiAction { name: "mais".into(), source: "mais".into(), values: "{}".into() };
        let t = Instant::now();
        for _ in 0..1000 {
            p.on_action(&click).unwrap();
        }
        println!("rodada {i}: carregar {load:?}, activate {activate:?}, clique (média de 1000) {:?}", t.elapsed() / 1000);
    }
    let t = Instant::now();
    let bytes = tea_plugins::runtime::precompile(Path::new(&wasm)).unwrap();
    println!("pré-compilar: {:?} ({} KB)", t.elapsed(), bytes.len() / 1024);
    let t = Instant::now();
    let _p = unsafe { Instance::load_precompiled(&bytes, &project) }.unwrap();
    println!("abrir pré-compilado: {:?}", t.elapsed());
}
