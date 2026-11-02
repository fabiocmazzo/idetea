#!/usr/bin/env bash
# Teste guiado do componente `file` das interfaces: o agente abre um arquivo para o usuário editar, e a resposta
# só chega depois de a IDE salvar o arquivo.
#
# Uso (da raiz do repositório):
#   examples/interfaces/arquivos.sh                 # projeto de teste em /tmp/idetea-teste-arquivos
#   examples/interfaces/arquivos.sh /outra/pasta
#
# Cria um projeto de teste (com a extensão Interfaces ligada só nele), abre a IDE com `cargo run` e vai fazendo
# o papel do agente, passo a passo. Feche a IDE no fim.
set -euo pipefail

repo=$(cd "$(dirname "$0")/../.." && pwd)
P=${1:-/tmp/idetea-teste-arquivos}
name=mensagem-commit
ui="$P/.idetea/ui"
responses="$ui/responses/$name"

passo() { printf '\n\033[1m== %s\033[0m\n' "$*"; }
espere_enter() { read -r -p "   (Enter para seguir) " _; }

write_doc() { # $1 = revisão, $2 = caminho do rascunho
    cat > "$ui/.$name.json.tmp" <<JSON
{
  "format": "idetea-ui", "version": 1, "revision": $1,
  "title": "Mensagem do commit", "source": "examples/interfaces/arquivos.sh",
  "components": [
    { "id": "intro", "component": "text", "text": "Revisão $1. Escrevi um rascunho da mensagem: ajuste no editor e clique em Commitar." },
    { "id": "msg", "component": "file", "path": "$2", "line": 3, "column": 1,
      "label": "Editar a mensagem", "open": true },
    { "id": "codigo", "component": "file", "path": "src/main.rs", "line": 2, "label": "A mudança principal" },
    { "id": "ok", "component": "button", "label": "Commitar", "action": { "name": "commitar" } },
    { "id": "no", "component": "button", "label": "Cancelar", "style": "secondary", "action": { "name": "cancelar" } }
  ]
}
JSON
    mv "$ui/.$name.json.tmp" "$ui/$name.json" # a IDE nunca vê o arquivo pela metade
}

passo "Preparando o projeto de teste em $P"
rm -rf -- "$P"
mkdir -p "$ui/rascunhos" "$P/src"
printf '[extensions]\ninterfaces = { enabled = true }\n' > "$P/.idetea/config.toml"
printf 'fn main() {\n    println!("olá");\n}\n' > "$P/src/main.rs"
# Um Cargo.toml só para o rust-analyzer não reclamar do projeto de teste.
printf '[package]\nname = "teste-arquivos"\nversion = "0.1.0"\nedition = "2021"\n' > "$P/Cargo.toml"
printf '# Mensagem do commit\n\nescreva aqui\n' > "$ui/rascunhos/commit.md"

passo "Compilando e abrindo a IDE (log em $P.log)"
(cd "$repo" && cargo build -q -p tea-app)
(cd "$repo" && cargo run -q -p tea-app -- --no-splash "$P" >"$P.log" 2>&1 &)
echo "   Espere a janela abrir."
espere_enter

passo "1. O agente pede a mensagem do commit"
write_doc 1 .idetea/ui/rascunhos/commit.md
cat <<'TXT'
   Confira:
   - o painel Interfaces aparece com "Mensagem do commit";
   - o rascunho .idetea/ui/rascunhos/commit.md abre sozinho, com o cursor na linha 3.
   Agora escreva algo na linha 3, NÃO salve, e clique em "Commitar" no painel.
TXT
echo "   Esperando o clique..."
until ls "$responses"/*.json >/dev/null 2>&1; do sleep 0.3; done
echo "   Resposta recebida:"
cat "$responses"/*.json; echo
echo "   O rascunho no disco (tem que ter o que você escreveu: a IDE salvou antes de responder):"
sed 's/^/     | /' "$ui/rascunhos/commit.md"
rm -f -- "$responses"/*.json # o agente leu
espere_enter

passo "2. Uma revisão nova enquanto você digita"
cat <<'TXT'
   Clique na aba src/main.rs e comece a digitar sem parar.
   Em 5 segundos chega a revisão 2, que pede para abrir o rascunho de novo.
TXT
for i in 5 4 3 2 1; do printf '   %s...\r' "$i"; sleep 1; done
write_doc 2 .idetea/ui/rascunhos/commit.md
cat <<'TXT'
   Revisão 2 escrita. Confira:
   - o cursor continuou em src/main.rs (o rascunho não abriu por cima);
   - apareceu o aviso "O agente pediu para abrir...";
   - clicar em "Editar a mensagem" no painel abre o rascunho;
   - clicar em "A mudança principal" abre src/main.rs na linha 2.
TXT
espere_enter

passo "3. Caminho fora do projeto"
write_doc 3 ../fora.md
echo '   Confira: o painel mostra um erro em "components[1].path" (o caminho tem "..").'
espere_enter

passo "Fim"
rm -f -- "$ui/$name.json" # fecha a interface
echo "   Interface fechada. Feche a IDE (a aba src/main.rs tem alterações: pode descartar)."
echo "   O projeto de teste fica em $P (apague com: rm -rf $P $P.log)."
