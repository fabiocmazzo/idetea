# Language pack: Hare

## Suporte

- Arquivos `.ha`, com a gramática embutida `hare.sublime-syntax`.
- LSP padrão: [`hare-lsp -S`](https://git.sr.ht/~whynothugo/hare-lsp), quando `hare-lsp` estiver instalado no `PATH`. Sem
  ele, o aviso da IDE aponta o repositório para compilar.
- O `hare-lsp` não vem com o Hare nem tem pacote no Arch/AUR: compile-o do repositório (precisa da biblioteca
  `hare-json`, no AUR) e coloque em `~/.local/bin`.
- O servidor oferece definição, hover, símbolos do documento e diagnósticos de parsing. Não promete validação semântica,
  completion, references, rename ou formatting.
- Não há tarefas declarativas: um `Makefile` não identifica sozinho um projeto Hare. Os alvos do Makefile e as tarefas
  configuradas pelo usuário continuam disponíveis.
- Debug e instalação do toolchain não fazem parte deste pack.

## Procedência

Gramática: <https://github.com/ravengraves/hare-sublime-syntax>, revisão
`826727cd2958d5d90ba012852afdea480bd621cb`, copiada sem modificações. Licença Unlicense em `LICENSE.txt`.
A gramática deriva de <https://github.com/aDotInTheVoid/hare-highlighing-vscode> e sua geração upstream informada é de
2022-04-27; exemplos da linguagem atual devem continuar no teste de regressão.

## Validação

- 2026-11-09: manifesto, extensão, recursos embutidos, parsing da gramática pelo editor e escopos de palavras-chave,
  tipos, números, strings e comentários cobertos por testes automáticos.
- 2026-11-09, smoke test real: `hare 0.26.0.1` (Arch), `hare-json 0.26.0.0` e `hare-lsp` `e3db7b8` (branch `main`,
  compilado com `make build PREFIX=/usr`). Num projeto com dois módulos e `Makefile`, conferidos na IDE: cores,
  hover, definição entre módulos (`F12`), símbolos/Outline, diagnóstico de sintaxe e alvos do Makefile nas tarefas.
  Erro de tipo não gera diagnóstico, como esperado. O servidor também anuncia inlay hints, que a IDE ainda não mostra.
- Limitação conhecida: o `hare-lsp` aborta (SIGABRT, `hare/lex/lex.ha:947:15: attempted to unlex more than one token`)
  com qualquer `;` faltando num comando, por exemplo `let x = 1` seguido de outra linha. A falha é do parser da
  biblioteca padrão (`hare::parse`), reproduzida sem o `hare-lsp` em Hare 0.26.0.1 e ainda presente no `master`
  (`27d9d92`); o `harec` dá um erro normal para o mesmo texto. A IDE avisa na primeira queda e religa o servidor quando
  a digitação pausa. Relato para o upstream (projeto Hare) em preparação.
