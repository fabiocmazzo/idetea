//! `cargo run --release -p tea-ext-search --example bench -- <pasta> <busca>`
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Instant;
use tea_ext_search::engine::{self, Options};

fn main() {
    let mut args = std::env::args().skip(1);
    let root = args.next().unwrap_or_else(|| ".".into());
    let query = args.next().unwrap_or_else(|| "fn ".into());
    let m = engine::build_matcher(&query, &Options::default()).unwrap();
    let t = Instant::now();
    let r = engine::search(root.as_ref(), &m, &Arc::new(AtomicBool::new(false)));
    println!(
        "{} resultados em {} arquivos, {} lidos, truncado={} em {:?}",
        r.total, r.files.len(), r.files_scanned, r.truncated, t.elapsed()
    );
}
