#!/usr/bin/env bash
# Faz uma pergunta no painel Interfaces da IDE TEA e espera a resposta.
#
# Uso: rode na raiz de um projeto aberto na IDE, com a extensão ligada
#      ([extensions] interfaces = { enabled = true }).
#   examples/interfaces/pergunta.sh            # pergunta, espera, mostra a resposta e fecha
#   examples/interfaces/pergunta.sh revisao    # depois da resposta, manda uma revisão 2 antes de fechar
#
# O que mostra: o agente escreve .idetea/ui/<nome>.json (temporário + mv), a IDE grava
# .idetea/ui/responses/<nome>/<seq>-<event>.json, o agente lê e apaga a resposta, e apaga o documento no fim.
set -euo pipefail

name=pergunta
ui=.idetea/ui
responses="$ui/responses/$name"
mkdir -p "$ui"

write_doc() { # $1 = revisão, $2 = texto de cima
    cat > "$ui/.$name.json.tmp" <<JSON
{
  "format": "idetea-ui",
  "version": 1,
  "revision": $1,
  "title": "Pergunta do script",
  "source": "examples/interfaces/pergunta.sh",
  "data": { "linguagem": "rust", "testes": true },
  "components": [
    { "id": "intro", "component": "text", "text": "$2" },
    { "id": "ling", "component": "select", "label": "Linguagem", "value": "/linguagem",
      "options": [ { "value": "rust", "label": "Rust" }, { "value": "python", "label": "Python" }, "go" ] },
    { "id": "testes", "component": "checkbox", "label": "Gerar testes", "value": "/testes" },
    { "id": "nome", "component": "input", "label": "Nome do módulo", "placeholder": "ex.: parser", "value": "/nome" },
    { "id": "ok", "component": "button", "label": "Criar", "action": { "name": "criar", "context": ["/linguagem", "/testes", "/nome"] } },
    { "id": "cancelar", "component": "button", "label": "Cancelar", "style": "secondary", "action": { "name": "cancelar" } }
  ]
}
JSON
    mv "$ui/.$name.json.tmp" "$ui/$name.json" # a IDE nunca vê o arquivo pela metade
}

wait_response() { # mostra a primeira resposta que chegar e apaga (é a confirmação para a IDE)
    echo "Esperando a resposta no painel Interfaces..."
    while :; do
        for f in "$responses"/*.json; do
            [ -e "$f" ] || continue
            echo "--- $f"
            cat "$f"
            rm -- "$f"
            return
        done
        sleep 0.3
    done
}

write_doc 1 "Escolha como criar o módulo."
wait_response

if [ "${1:-}" = revisao ]; then
    sleep 2
    write_doc 2 "Revisão 2: o agente mudou este texto. O que você digitou continua; responda de novo."
    wait_response
fi

sleep 2
rm -f -- "$ui/$name.json" # fecha a interface
echo "Interface fechada."
