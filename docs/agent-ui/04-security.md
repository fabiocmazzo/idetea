# 04 — Segurança e confiabilidade

O requisito que manda: **nada que o agente escreve pode derrubar ou travar a interface da IDE.** Confiar no modelo
não é confiar no código que ele produz, e o mesmo vale para o JSON e para a mídia.

## Modelo de confiança

- **O agente é um processo do usuário.** Ele já pode ler e escrever qualquer arquivo do projeto e rodar comandos no
  terminal dele. A IDE não protege o usuário do agente nesse plano; quem faz isso é o próprio agente (permissões do
  Claude Code, do Codex).
- **O que a IDE protege:** a si mesma (o processo gráfico), o usuário de ações escondidas (um clique tem que fazer o
  que mostra) e o que está fora do projeto.
- **Um repositório clonado não instala código** ([RFC 0002](../rfc/0002-plugins-wasm.md)): continua valendo. Um
  módulo do agente (nível 3) não é um plugin: só roda com confirmação, e nunca sozinho por estar na pasta.

## Os riscos, nível a nível

| Risco | Hoje | Proposta |
| --- | --- | --- |
| JSON enorme ou fundo | 256 KB, 200 componentes, 16 níveis em `data` (recusa com o campo) | continua; `columns` sem aninhar; `show_if` sem expressão |
| Imagem feita para estourar memória | cabeçalho antes de decodificar, 4096 px, 64 Mpx por documento | o mesmo limite para `compare` e para anexos |
| WAV malformado | leitor próprio que só lê o necessário, com erro em vez de pânico | continua |
| Caminho para fora do projeto | ponteiros só em `data`; `path` não sai da raiz; não segue symlinks | anexos são copiados **para** o projeto, nunca lidos de fora sem o usuário escolher |
| Expressão que nunca termina | não existe | a linguagem não tem laço; limite de 256 nós por expressão e de 200 valores calculados por documento; avaliação na thread da interface, porque termina em microssegundos |
| Ciclo entre valores | não existe | o documento é recusado ao ler |
| Divisão por zero, `NaN`, infinito | não existe | vira `null` com o motivo; nunca pânico |
| Módulo WASM preso ou guloso | os plugins já têm prazo por época | combustível + prazo de 100 ms por chamada, `StoreLimits` de 64 MB, uma chamada por vez, numa thread; estourou, o componente mostra o erro e o módulo fica desligado até a próxima revisão |
| Módulo que lê arquivo ou abre rede | — | sem WASI de arquivo, rede, relógio ou aleatório; só a função `calcular` |
| Processo longo | os terminais pedidos já são aprovados e ficam numa aba que o usuário vê e fecha | continua; nenhum processo nasce do painel |
| Laço de eventos (o agente reescreve a cada resposta) | cada revisão é um arquivo novo; o painel só redesenha o que mudou | continua; uma revisão a cada 250 ms no máximo, que é o intervalo do `stat` |
| Escrita concorrente | um escritor por arquivo, gravação pelo temporário | anexos: só a IDE escreve; o agente só apaga |
| Interfaces órfãs | `idetea ui fix` lista e limpa | também anexos sem resposta |
| Roubar o foco do editor | `open: true` só abre um arquivo; `OpenFileWhenIdle` não interrompe quem está digitando | `compare` e `chart` nunca pegam o teclado |
| Captura de tela com segredo | não existe | a captura é **só da janela da IDE** e só quando o usuário clica; o painel mostra a miniatura antes de enviar, com "remover" |

## Permissões por projeto

**[Recomendação]** Uma só, por enquanto: `[ui] modules = "ask"` no config do projeto (`ask`, padrão; `never`). Sem
`always`: um módulo novo sempre passa pela confirmação. As outras capacidades (níveis 0 a 2) não precisam de
permissão, porque não fazem nada fora do painel.

## Falhas

- O painel nunca mostra um documento meio lido: a gravação é pelo temporário, e um documento inválido mostra o erro
  no lugar.
- Uma falha de mídia, de expressão ou de módulo fica **no componente** (mensagem no lugar), e o resto do documento
  funciona.
- IDE fechada no meio: o documento e as respostas ficam no disco, como hoje; um anexo só existe depois do clique.
