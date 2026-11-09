# Language pack: Hare

## Suporte

- Arquivos `.ha`, com a gramática embutida `hare.sublime-syntax`.
- LSP padrão: [`hare-lsp -S`](https://git.sr.ht/~whynothugo/hare-lsp), quando `hare-lsp` estiver instalado no `PATH`.
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
- Toolchain `hare` e `hare-lsp` não estavam instalados na máquina de implementação; comandos e respostas LSP ainda não
  foram executados. Registrar aqui as versões e os resultados do primeiro smoke test real.
