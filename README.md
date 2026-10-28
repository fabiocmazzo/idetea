# IDE TEA

**Just code. Add ice.**

IDE rápida e extensível em Rust. Veja [SPEC.md](SPEC.md).

```sh
cargo run --release -- [--no-splash] [pasta|arquivo...]

# benchmark do editor com arquivo grande (sem renderizar)
cargo run --release -p tea-editor --example large_file -- 50000 [--write grande.rs]
```

Atalhos: `Ctrl+Shift+P` (ou `F1`) abre a paleta com todos os comandos e atalhos. Config: comando "Config: abrir config global".
