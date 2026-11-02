# Plugin de exemplo: Contador

Um painel com um contador, comandos na paleta e a contagem dos itens na raiz do projeto (lida pelo WASI). Serve de
modelo para escrever um plugin; a API está em `crates/tea-plugins/wit/plugin.wit` e as decisões em
`docs/rfc/0002-plugins-wasm.md`.

## Compilar e instalar

Da raiz do repositório (o plugin é um projeto à parte: compilar o workspace inteiro para WASM falha no `winit`):

```sh
rustup target add wasm32-wasip2                      # uma vez
cargo build --release --target wasm32-wasip2 --manifest-path examples/plugins/contador/Cargo.toml
mkdir -p ~/.local/share/idetea/plugins/contador
cp examples/plugins/contador/plugin.toml \
   examples/plugins/contador/target/wasm32-wasip2/release/contador.wasm \
   ~/.local/share/idetea/plugins/contador/
```

A IDE precisa ter sido compilada com a feature `plugins` (protótipo, fora do padrão):

```sh
cargo run -p tea-app --features plugins -- <pasta do projeto>
```

`IDETEA_PLUGIN_LOG=1` mostra o log dos plugins no terminal.

## O que dá para fazer

- Paleta: "Contador: mostrar o painel", "Contador: zerar", "Contador: rodar o comando da config".
- Painel: `+1` e `Zerar`; salvar um arquivo na IDE conta no painel.
- "Contador: travar (demonstra o prazo)": o plugin entra num laço infinito; a IDE o interrompe em meio segundo e o
  desliga, com o motivo no painel.

## Config e comandos externos

O botão "Rodar" (ou o comando "Contador: rodar o comando da config") roda um programa na raiz do projeto com
`host.run`, sem travar a IDE, e mostra as primeiras 20 linhas da saída no painel. O programa vem da config do
plugin, no `config.toml` da IDE (global ou do projeto):

```toml
[plugins.contador]
comando = ["git", "status", "--short"]   # o padrão; sem shell: para pipes, ["sh", "-c", "..."]
```

Salvar o config avisa o plugin (`config-changed`); o painel mostra o comando novo.
