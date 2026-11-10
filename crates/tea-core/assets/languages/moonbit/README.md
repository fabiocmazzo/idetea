# Language pack: MoonBit

## Suporte

- Arquivos `.mbt` e `.mbti`, com `languageId` `moonbit` e a gramática embutida `moonbit.sublime-syntax`.
- LSP padrão: [`moon lsp`](https://docs.moonbitlang.com/en/stable/toolchain/moon/commands.html), quando `moon` estiver
  no `PATH`. A IDE acrescenta ao próprio `PATH` o `bin` de `$MOON_HOME` (padrão `~/.moon/bin`), onde o instalador
  oficial coloca o toolchain, para funcionar também quando é aberta pelo menu do sistema. Sem o `moon`, o aviso traz o
  link de instalação.
- Tarefas `moon check`, `moon build`, `moon test` e `moon fmt --check`, ativadas pelo `moon.mod` mais próximo do arquivo
  ativo e executadas nesse diretório. Sem arquivo ativo, usa-se o `moon.mod` da raiz.
- `moon.mod.json` é legado e não ativa tarefas. `moon.work` não cria um segundo modelo de workspaces nesta etapa.
- `.mbt.md`, instalação do toolchain e debug declarativo não fazem parte deste pack.

## Procedência

Gramática: <https://github.com/hyrious/moonbit-syntax-highlight>, arquivo `MoonBit.sublime-syntax`, revisão
`ed723f8f60c294a5df83a28f0689cdb56fa8f27a`, copiada sem modificações. Licença MIT em `LICENSE.txt`.

## Validação

- 2026-11-09: manifesto, extensões, recursos embutidos, parsing da gramática, escopos léxicos e interpolação de strings,
  merge/disable do LSP, descoberta de módulo aninhado, fallback de raiz e precedência das tarefas cobertos por testes
  automáticos.
- 2026-11-09: `moon 0.1.20261104` (`940aa18`, `moonc 0.10.20+65efd5b93`) executou `moon check`, `moon build`,
  `moon test` e `moon fmt --check` com sucesso num módulo Wasm temporário. No dogfooding, o LSP foi confirmado em
  `.mbt` e `.mbti`, e as tarefas `moon check` e `moon fmt --check` terminaram com código 0 pela IDE no módulo aninhado.
