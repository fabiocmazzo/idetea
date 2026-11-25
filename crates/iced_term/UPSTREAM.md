# iced_term no workspace

Cópia do [iced_term](https://github.com/Harzu/iced_term) 0.8.0 (licença MIT, em `LICENSE`), do commit
`e0e90be2c160bd68f1bf544a51bf772d75b98ffc`, publicado no crates.io. Trazida para cá pela
[ADR 0004](../../docs/decisions/0004-iced-term-no-workspace.md) para a RFC 0024 (histórico configurável, busca e,
depois, um backend que não seja dono do PTY).

O código de `src/` entrou sem mudança (commit "vendor iced_term 0.8.0"). Cada mudança depois disso fica listada
aqui, com o motivo, para dar para comparar com o upstream ou mandar a ele quando fizer sentido.

## Mudanças

- **`BackendSettings::scrollback`** (RFC 0024, M1.4): o número de linhas do histórico, que ia fixo no
  `term::Config::default()` (10 000). O padrão continua 10 000.
- **`BackendCommand::Search { text, older }`, `BackendCommand::ClearSelection` e `Action::SearchResult(bool)`**
  (RFC 0024, M1.4): busca de texto literal (com o "smart case" do alacritty) no histórico, a partir da seleção ou
  do fim, selecionando e rolando até a ocorrência. Usa o `RegexSearch`/`search_next` do alacritty, que o upstream
  só usava para achar URLs. Com testes (`search_tests`).
- **`Terminal::text(max_lines)` e `BackendSettings::preamble`** (RFC 0024, M2): o texto das últimas linhas
  (histórico e tela, sem cores) e uma saída mostrada antes do programa começar, passada pelo parser antes de o
  laço do pty existir (o programa nunca a recebe). Juntos, mostram o texto da sessão anterior. Com teste.
- **Barra de rolagem** (2026-11-24): uma faixa de 8 px à direita, reservada (as colunas param antes dela), com o
  polegar na cor `background.strong` do tema. Aparece quando há histórico e some na tela alternativa (vim, htop).
  Arrastar o polegar rola junto (`BackendCommand::ScrollTo(offset)`, novo), clicar na trilha pula uma página, e
  um clique nela nunca começa seleção. A conta do polegar (`thumb`, `offset_for`) tem testes.
- **Tamanho ao trocar de terminal** (2026-11-24): o widget reaproveita o estado quando outro terminal ocupa o mesmo
  lugar na tela (troca de aba), e só redimensionava quando o tamanho do layout mudava. Um terminal criado ou
  medido em outro lugar ficava com as colunas antigas (o texto quebrava em ~37 colunas num painel largo). Agora o
  widget compara também com `Backend::layout_size()` (novo).
- **Links e caminhos** (RFC 0025, 2026-11-24): o `Ctrl+clique` não abre mais nada sozinho (o upstream chamava
  `open::that` e fazia `panic!` quando falhava; a crate `open` saiu). O backend devolve `Action::OpenLink(Link)`, e
  quem embute decide. O link sob o mouse vem do OSC 8 da célula (`Cell::hyperlink()`) ou do texto da linha lógica
  (as linhas quebradas por `WRAPLINE` juntas), pelo módulo novo `links` (URLs, `caminho:linha:coluna`, `-->` do
  rustc, `File "x", line N` do Python, `at f (x:l:c)` do Node, `x(l,c)` do MSBuild; funções puras, com testes). Um
  caminho só vira link se `BackendSettings::resolve_path` (novo) o achar no disco; o resolvedor recebe o pid do shell
  (guardado ao criar o pty, só em unix). Com `Ctrl` apertado, uma caixa mostra o destino
  (`RenderableContent::hovered_link`). A `URL_REGEX` e o `regex_match_at` do upstream saíram.
