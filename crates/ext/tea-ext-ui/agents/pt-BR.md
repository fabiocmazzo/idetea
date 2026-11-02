## Perguntas pela IDE (painel Interfaces)

Quando precisar que eu escolha entre opções, confirme algo ou preencha poucos campos, você pode criar uma
interface na IDE TEA em vez de perguntar no chat. Este texto basta; o formato completo está em
https://github.com/fabiocmazzo/idetea/blob/main/docs/interfaces/formato.md

1. Escreva `.idetea/ui/<nome>.json` (nome: minúsculas, números, `_` e `-`). Escreva num temporário e renomeie,
   para a IDE nunca ver o arquivo pela metade: `cat > .idetea/ui/.<nome>.json.tmp` e depois `mv` para o nome final.
   ```json
   {"format": "idetea-ui", "version": 1, "revision": 1, "title": "…", "source": "<seu nome>",
    "data": {"opcao": "a", "testes": true},
    "components": [
      {"id": "q", "component": "text", "text": "…"},
      {"id": "op", "component": "select", "label": "…", "value": "/opcao", "options": ["a", {"value": "b", "label": "B"}]},
      {"id": "t", "component": "checkbox", "label": "Escrever testes", "value": "/testes"},
      {"id": "n", "component": "input", "label": "Nome", "placeholder": "opcional", "value": "/nome"},
      {"id": "ok", "component": "button", "label": "Confirmar", "action": {"name": "ok", "context": ["/opcao", "/testes", "/nome"]}},
      {"id": "no", "component": "button", "label": "Cancelar", "style": "secondary", "action": {"name": "cancelar"}}
    ]}
   ```
   - Componentes: `text` (`style`: `muted`, `warning`, `error`), `select`, `checkbox` (booleano), `input` (texto)
     e `button` (`style`: `secondary`, `danger`). `value` é um ponteiro JSON para um valor em `data`; `data` traz
     os valores iniciais.
   - Na resposta só vêm os caminhos listados no `context` do botão clicado; um campo nunca preenchido vem `null`.
   - `action.name` é um texto livre que volta na resposta para você saber qual botão foi clicado.
   - Depois de um clique, os botões ficam desligados até você escrever uma revisão nova.
2. Espere a resposta em `.idetea/ui/responses/<nome>/`. Use bash (no zsh, `compgen` não existe ou se comporta
   diferente):
   ```sh
   timeout 600 bash -c 'until ls .idetea/ui/responses/<nome>/*.json >/dev/null 2>&1; do sleep 1; done'
   ```
   Cada arquivo é uma resposta:
   `{"action": "ok", "values": {"/opcao": "b", "/testes": true, "/nome": null}, "revision": 1, "seq": 1, ...}`.
   `action` é o `action.name` do botão; `revision` é a revisão que eu via ao clicar; `seq` é a ordem (pode haver
   mais de um arquivo; leia em ordem de nome). Leia e **apague o arquivo da resposta**: é assim que a IDE sabe que
   você recebeu.
3. Para mostrar um resultado ou fazer outra pergunta, reescreva o mesmo arquivo com `revision` maior. Um
   documento só com `text`, sem botões, serve para mostrar algo sem esperar resposta.
4. Ao terminar, apague `.idetea/ui/<nome>.json`. O resto da pasta (`.gitignore`, `responses/`) é da IDE; deixe.

Não coloque segredos no documento. Se não vier resposta em 10 minutos (o painel pode estar desligado), pergunte
no chat.
