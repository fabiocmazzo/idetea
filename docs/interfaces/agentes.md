# Instruções para agentes

Cole o bloco abaixo no `AGENTS.md` ou `CLAUDE.md` do projeto para que um agente de código saiba perguntar
coisas ao usuário pelo painel **Interfaces** da IDE TEA, em vez de só pelo chat. Serve quando a pergunta tem
opções claras (escolher entre caminhos, confirmar uma ação arriscada, preencher alguns campos).

````markdown
## Perguntas pela IDE (painel Interfaces)

Quando precisar que eu escolha entre opções, confirme algo ou preencha poucos campos, você pode criar uma
interface na IDE TEA em vez de perguntar no chat. Formato completo: docs/interfaces/formato.md.

1. Escreva `.idetea/ui/<nome>.json` (nome: minúsculas, números, `-`; escreva num `.tmp` e renomeie com `mv`):
   ```json
   {"format": "idetea-ui", "version": 1, "revision": 1, "title": "…", "source": "<seu nome>",
    "data": {"opcao": "a"},
    "components": [
      {"id": "q", "component": "text", "text": "…"},
      {"id": "op", "component": "select", "label": "…", "value": "/opcao", "options": ["a", "b"]},
      {"id": "ok", "component": "button", "label": "Confirmar", "action": {"name": "ok", "context": ["/opcao"]}},
      {"id": "no", "component": "button", "label": "Cancelar", "style": "secondary", "action": {"name": "cancelar"}}
    ]}
   ```
   Componentes: `text`, `button`, `checkbox`, `select`, `input`. Valores ficam em `data` e são apontados por
   ponteiros JSON (`"/opcao"`).
2. Espere aparecer `.idetea/ui/responses/<nome>/*.json` (olhe a cada segundo). A resposta tem `action` (o
   `action.name` do botão) e `values` (caminho → valor). Leia, **apague o arquivo da resposta**, e siga.
3. Para mudar a pergunta, reescreva o documento com `revision` maior. Ao terminar, apague `.idetea/ui/<nome>.json`.

Não coloque segredos no documento. Se o painel não estiver ligado (nenhuma resposta depois de alguns minutos),
pergunte no chat.
````
