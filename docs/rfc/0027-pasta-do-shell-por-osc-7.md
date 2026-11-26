# RFC 0027 — A pasta do shell pelo OSC 7

- **Status:** adiada no painel em 2026-11-26: o ganho em Linux e macOS é pequeno, e o do Windows depende de
  testar o ConPTY. Fica a análise, para quando o Windows entrar.
- **Continua:** [RFC 0025](0025-links-e-caminhos-no-terminal.md), que deixou o OSC 7 de fora, e o #29 (os shells
  reabrem na pasta onde estavam).

Marcadores: **[Fato]** conferido no código ou na fonte da crate (`alacritty_terminal` 0.25.1, `vte` 0.15.0);
**[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

O terminal precisa saber a pasta atual de cada shell em dois lugares: para resolver um caminho relativo num
`Ctrl+clique` (RFC 0025) e para reabrir o shell na mesma pasta (#29). Hoje ele pergunta ao sistema qual é a pasta do
processo do shell. O OSC 7 é o shell avisando a pasta a cada `cd` (`ESC ] 7 ; file://host/caminho BEL`), e é o que
o GNOME Terminal, o WezTerm, o kitty e o foot usam.

Esta RFC propõe ler o OSC 7 quando o shell o manda e preferi-lo à pergunta ao sistema. A pergunta ao sistema
continua como reserva.

## O que existe hoje

- **[Fato]** `tea_core::os::process_cwd(pid)`: `/proc/<pid>/cwd` no Linux, `proc_pidinfo` no macOS, `None` no
  Windows. O pid é o do shell que a aba abriu (`Backend::shell_pid`, só em unix).
- **[Fato]** O `vte` 0.15 não trata o OSC 7: o código cai em `unhandled` (só um `debug!`). O `alacritty_terminal`
  lê o PTY numa thread própria (`EventLoop`) e passa os bytes direto ao parser; o `iced_term` não vê os bytes.
- **[Fato]** O `EventLoop::new` aceita qualquer `T: EventedPty`. Em unix, o leitor do `Pty` é um `File`
  (`Pty::file()`), que dá para duplicar (`try_clone`): um embrulho em volta do `Pty` pode ler pelo mesmo
  descritor e olhar os bytes antes de passá-los adiante, sem patch no `alacritty_terminal`.
- **[Fato]** No Windows, o leitor é um `ReadPipe` dentro do `Pty`, sem jeito público de duplicar.
- **[Fato]** O fish 4 (o desta máquina é o 4.9.3) manda OSC 7 sozinho. O bash e o zsh não mandam, a não ser com o
  `vte.sh` de algumas distribuições (só quando `VTE_VERSION` existe) ou com uma linha no `.bashrc`/`.zshrc`. O
  PowerShell e o `cmd` não mandam.

### O que a pergunta ao sistema já acerta, e o que erra

Em Linux e macOS, a pasta do processo do shell já é exata a cada `cd` daquele shell. Ela erra quando o shell que o
usuário vê não é o processo que a aba abriu:

- um shell dentro do outro (`bash` dentro do fish, `nix-shell`, `poetry shell`, `sudo -s`);
- um shell em outro lugar (`ssh`, `docker exec`, `toolbox enter`): a pasta é de outra máquina;
- no Windows, sempre: não há a pergunta, e vale a pasta onde a aba abriu.

**[Fato]** Então o ganho em Linux e macOS é pequeno (os casos acima). O ganho grande seria no Windows, que hoje não
tem nada, mas lá o shell não manda OSC 7 sem configuração e o leitor do PTY não se deixa embrulhar.

## Proposta

1. **Ler o OSC 7 (unix).** O `iced_term` embrulha o `Pty` e passa os bytes por um leitor que procura
   `ESC ] 7 ;` até `BEL` ou `ESC \`, também quando a sequência chega partida em duas leituras. Guarda o último
   caminho. Mudança no `UPSTREAM.md`.
2. **Só pastas desta máquina.** `file://` com host vazio, `localhost` ou o nome desta máquina; o caminho com os
   `%XX` decodificados. De outro host (`ssh`) o aviso é ignorado e vale a reserva, porque o caminho não existe
   aqui.
3. **Ordem:** a pasta do OSC 7, se o shell mandou uma que existe; senão, a pasta do processo; senão, a pasta onde a
   aba abriu; senão, a raiz. A mesma ordem nos links (RFC 0025) e ao gravar as abas (#29).
4. **Os shells.** **[Recomendação]** Sem mexer na configuração do shell: o fish já funciona, e
   `docs/terminais-de-agente.md` ganha as linhas para bash e zsh, para quem quiser. Injetar a integração
   automaticamente (`PROMPT_COMMAND` no bash, um `ZDOTDIR` próprio no zsh, como o VS Code faz) é mais invasivo e
   pode brigar com o `.bashrc` do usuário; fica para depois, se fizer falta.
5. **Windows.** **[Hipótese]** O ConPTY repassa sequências OSC que não conhece; não testado. Para ler no Windows
   seria preciso embrulhar o `ReadPipe` com `unsafe` ou um patch no `alacritty_terminal`, e ainda configurar o
   `prompt` do PowerShell. **[Recomendação]** Fica para uma etapa própria, depois de testar o ConPTY numa máquina
   Windows.

## Riscos

- Um programa pode mandar um OSC 7 falso (um `cat` de um arquivo com a sequência). O caminho só vale se existir e for
  uma pasta, e só muda para onde um `Ctrl+clique` resolve caminhos e onde o shell reabre: nada é executado.
- O leitor custa uma passada pelos bytes de saída, sem cópia.
- Um OSC 7 velho: depois de sair de um shell interno que mandava OSC 7, a pasta fica a do último aviso até o shell
  de fora mandar outro (o fish manda a cada prompt; um bash sem a linha, nunca). **[Recomendação]** Esquecer o OSC 7
  quando o processo do shell da aba já não tem filhos em primeiro plano não é simples de saber; aceitar o caso.

## Decisão (painel, 2026-11-26)

Não fazer agora. Quando voltar, começar pelo Windows (testar se o ConPTY repassa o OSC 7), que é onde está o
ganho.
