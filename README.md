# IDE TEA

**Just code. Add ice.**

IDE rápida e extensível em Rust. Veja [SPEC.md](SPEC.md).

```sh
cargo run --release -- [--no-splash] [pasta|arquivo...]

# benchmark do editor com arquivo grande (sem renderizar)
cargo run --release -p tea-editor --example large_file -- 50000 [--write grande.rs]
```

Com [`just`](https://github.com/casey/just): `just run [args]` (ou `just run-release`) instala o ícone e roda.
No Wayland o painel do desktop (COSMIC, GNOME, KDE) só mostra o ícone da janela se existir um `idetea.desktop`
com o mesmo app id da janela (`idetea`); `just desktop` instala ele e o ícone em `~/.local/share`
(só uma vez, `cargo run` direto também passa a mostrar o ícone). `just undesktop` remove.

Atalhos: `Ctrl+Shift+P` abre a paleta com todos os comandos; `F1` abre a ajuda (atalhos e guia); `Ctrl+,` abre as configurações. `Ctrl+Shift+T` abre o terminal
(nele, `Ctrl+letra` vai para o shell; os atalhos `Ctrl+Shift` da IDE continuam valendo). `Ctrl+Shift+G` mostra as mudanças do git.
Config: comando "Config: abrir config global".


teste



