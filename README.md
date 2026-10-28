# IDE TEA

IDE rápida e extensível em Rust. Veja [SPEC.md](SPEC.md).

```sh
cargo run --release -- [pasta|arquivo]

# benchmark do editor com arquivo grande (sem renderizar)
cargo run --release -p tea-editor --example large_file -- 50000 [--write grande.rs]
```

Atalhos (Fase 0): `Ctrl+S` salvar, `Ctrl+W` fechar aba, `Ctrl+B` painel esquerdo, `Ctrl+Shift+R` recarregar lista.
