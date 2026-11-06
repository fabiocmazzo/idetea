# Lições do iced (da IDE TEA)

O que a IDE TEA aprendeu com o iced 0.14, para levar a outro projeto. Cada item diz o problema que apareceu e o
jeito que funcionou. No fim há sugestões para um cliente de banco de dados (marcadas assim: são ideias, não
coisas testadas aqui).

## 1. Arquitetura

**Um `App` com `boot`, `update`, `view`, `subscription`, `theme` e `title`:**

```rust
iced::application(App::boot, App::update, App::view)
    .title(App::title)
    .subscription(App::subscription)
    .theme(App::theme)
    .font(ICON_FONT_BYTES)                       // fontes extras (ícones) entram aqui
    .window(window::Settings {
        size: saved_size.unwrap_or(window::Settings::default().size),
        platform_specific: window::settings::PlatformSpecific {
            application_id: "dev.exemplo.app".into(),   // Wayland: liga a janela ao .desktop (ícone, agrupamento)
            ..Default::default()
        },
        ..Default::default()
    })
    .exit_on_close_request(false)                // para perguntar "salvar antes de sair?"
    .run()
```

- **O `boot` não recebe parâmetros.** Argumentos da linha de comando vão num `OnceLock` preenchido no `main`
  antes do `run()`, e o `boot` lê de lá.
- **`exit_on_close_request(false)` + `window::close_requests()`** na subscription: o X da janela vira uma
  mensagem, e o app decide (diálogo de não salvos, gravar sessão) e sai com `iced::exit()`.
- **Um `update` fino:** `update` chama `handle(message)` e depois faz o que vale para toda mensagem (gravar a
  sessão se mudou, publicar estado). Assim nada depende de lembrar de chamar em cada braço do `match`.
- **Partes independentes como "extensões" com o próprio `Message`:** cada uma tem `update`/`view`/`subscription`,
  e o app embrulha com `.map(move |m| Message::Ext(i, m))`. Conversam só por comandos e eventos, nunca uma
  chamando a outra. Isso deixou cada painel testável sozinho e permitiu desligar qualquer um.
- **Isole widgets de terceiros atrás de uma fachada.** Só o crate `tea-editor` usa o `iced-code-editor`; o resto
  da IDE usa os tipos dele. Quando foi preciso contornar comportamentos do widget (abaixo), mudou um arquivo.

## 2. Trabalho fora da interface (só threads, sem runtime async)

O `update` precisa voltar rápido; tudo que espera (disco, git, rede, processo) vai para uma thread.

**Uma tarefa que devolve um resultado:** thread + `oneshot`, entregue como mensagem.

```rust
fn spawn<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
    done: impl Fn(T) -> Message + Send + 'static,
) -> Task<Message> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || { let _ = tx.send(f()); });
    Task::future(async move { rx.await.ok() }).and_then(move |r| Task::done(done(r)))
}
```

**Um fluxo contínuo (eventos de um processo, de um servidor):** `mpsc::unbounded` e `Task::run(rx, Message::X)`
no `boot`, ou uma `Subscription`.

**Observar algo enquanto for preciso:** `Subscription::run_with(dados, fn)`. A thread termina quando o canal
fecha (`tx.is_closed()`), isto é, quando a subscription some.

```rust
#[allow(clippy::ptr_arg)] // `run_with` exige `fn(&D)` com o mesmo tipo dos dados.
fn watch(root: &PathBuf) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let root = root.clone();
    std::thread::spawn(move || loop {
        if tx.is_closed() { return; }
        // ... olha, e se mudou: tx.unbounded_send(Message::Changed(...))
        std::thread::sleep(Duration::from_millis(500));
    });
    rx
}
```

- `run_with` pede uma **`fn`, não uma closure**; os dados servem de chave (dados iguais = mesma subscription).
- `Subscription::map` também não aceita closures que capturam: para levar um índice, `sub.with(i).map(|(i, m)|
  Message::Ext(i, m))`.
- **Subscription só enquanto precisa:** `window::frames()` só com animação na tela; o "tique" de cópias de
  segurança só com arquivos alterados. Uma subscription parada não custa nada; uma ligada à toa acorda o app.

**Esperar o usuário parar de digitar (debounce):** um contador de geração. Cada edição soma 1 e agenda a
mensagem com o número; quando ela chega, só vale se ainda for o último.

```rust
fn text_later(&mut self) -> Task<Message> {
    self.text_wait += 1;
    let wait = self.text_wait;
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(250)); let _ = tx.send(wait); });
    Task::future(async move { rx.await.ok() }).and_then(|w| Task::done(Message::TextDue(w)))
}
// no update:
Message::TextDue(w) => if w == self.text_wait { self.send_text() } else { Task::none() },
```

O mesmo padrão descarta respostas velhas (uma busca que voltou depois de outra mais nova).

## 3. Teclado e foco

- **`iced::event::listen_with`** vê também as teclas que um widget já consumiu (`Status::Captured`). É o jeito de
  ter atalhos globais e ainda saber se o campo de texto usou a tecla.
- **Um painel que come o teclado inteiro (terminal) engole os atalhos globais.** A solução foi um widget
  embrulho (`PassKeys`) que não entrega ao conteúdo algumas teclas (`Ctrl+Shift+…`), que seguem "não usadas"
  para o atalho do app.
- **Com Shift, o iced manda a letra maiúscula** (`Key::Character("P")`): normalize antes de comparar com o
  atalho configurado.
- **Dê `Id` estável aos campos** (`text_input(...).id(format!("campo-{}", id_do_item))`). Sem isso, um componente
  novo acima do campo faz o foco "pular" para outro.
- Foco num widget de canvas de terceiros pode precisar de mais que `request_focus` (o editor só aceitava teclas
  depois de receber também o evento de foco ganho). Confira com um teste de tecla logo depois de abrir.
- **Clique fora tira o foco:** o app guarda se o ponteiro está sobre um painel e, num `ButtonPressed`, tira o foco
  do editor. O iced não faz isso sozinho entre widgets independentes.

## 4. Desenho, imagens e texto

- **Imagem que muda no disco:** crie o `image::Handle` **pelos bytes** (`Handle::from_bytes`), não pelo caminho.
  Pelo caminho, o iced reaproveita a imagem do cache e a aba recarregada continua mostrando a versão velha.
- **Peças do `markdown` não vão entre threads:** faça o parse (`markdown::parse`) e guarde os `Item`s junto de quem
  desenha; refaça quando o texto muda (não a cada `view`).
- **Fluent/i18n:** desligue as marcas Unicode de isolamento (`set_use_isolating(false)`); o iced as desenha como
  quadradinhos em volta das variáveis.
- **Sobreposições (diálogo, paleta, menu de contexto):** `stack![conteúdo, camada]` no `view`, com a camada
  capturando os cliques de fora.
- **Tema:** derive todas as cores do `Theme` (`theme.extended_palette()`) em vez de cores fixas; assim os temas
  do próprio iced (Nord, Dracula…) funcionam de graça.
- **Fonte embutida e medidas:** o `.font(bytes)` da aplicação só carrega a fonte **depois** do `boot`. Um widget
  que mede o texto ao receber a fonte (o `iced-code-editor` mede a largura de um caractere em `set_font`) mede uma
  fonte substituta se for criado no `boot`, e o realce sai deslocado (meio pixel por caractere). Carregue antes de
  `iced::application(...)`: `iced::advanced::graphics::text::font_system().write()` e `load_font(Cow::Borrowed(bytes))`.

## 5. Widgets de terceiros: o que nos mordeu

- **`iced_term` (terminal):**
  - Um terminal novo tem um tamanho provisório de 80×50 px até o widget medir. Qualquer saída antes disso sai
    espremida em ~10 colunas. **Mande um `Resize` com uma estimativa** logo ao criar.
  - Ele guarda o desenho com a posição na tela; quando o layout em volta muda (uma barra de abas aparece), o
    desenho velho sai no lugar antigo. Reaplicar a fonte limpa esse cache.
- **`iced-code-editor`:** a seleção não é pública (não dá para ler o texto selecionado sem mudar o crate). As
  sintaxes vêm do `syntect` padrão (sem TOML, TypeScript, `.env`…); a feature `two-face` traz as do `bat`.
- **Texto selecionável:** no iced 0.14, `text` e `markdown` não selecionam. Para copiar, mostre o texto num
  `text_editor` só de leitura: aplique só as ações com `!action.is_edit()`; o Ctrl+C o próprio widget trata (escreve
  na área de transferência sem passar pela sua mensagem). Um botão alterna entre formatado e selecionável.
- **Regra geral:** antes de adotar um widget, teste os casos de borda que você vai ter (tamanho antes da primeira
  medida, foco, arquivo enorme, recarregar) num exemplo pequeno.

## 6. Desempenho

- **Meça quadros sem janela.** Um exemplo com `iced::advanced::renderer::Headless` monta a interface com
  `UserInterface::build`, desenha e mede o tempo, inclusive gravando o quadro em PNG. Foi assim que apareceram
  os quadros de segundos com uma linha gigante (JSON minificado) — e o "modo leve" que resolveu.
- **Nunca faça trabalho pesado no `view`.** O `view` roda a cada mudança; prepare os dados no `update` (ou numa
  thread) e o `view` só monta widgets.
- **Listas grandes:** só desenhe o que aparece. Um `scrollable` com milhares de filhas custa layout de todas.
- **Medidas que enganam:** a primeira execução depois de compilar inclui o disco frio. Compare com a segunda.

## 7. Testes

- **Teste o `update`, não a tela.** Crie o estado, mande mensagens, confira o estado e os arquivos. Quase todos os
  testes da IDE são assim, sem janela, e rodam no CI.
- **Um `Context` de teste** (ou o tipo que o app passa às partes) torna as extensões testáveis sem o app inteiro.
- O que só se vê na tela (foco, desenho, tempo) fica para um teste manual curto, com passos escritos.

## 8. Distribuição (Linux)

- O executável do iced/wgpu **só depende da libc**: Wayland, X11, Vulkan e xkbcommon são carregados na hora.
  Compilar no Ubuntu 22.04 (glibc mais antiga) faz o binário rodar em mais distros.
- `.tar.gz` com um `install.sh` (executável em `~/.local/bin` + `.desktop` + ícone) e um AppImage cobrem quase
  todo mundo. O `application_id` da janela precisa bater com o nome do `.desktop` para o ícone aparecer no Wayland.
- `[profile.dist]` herdando `release` com `strip = true` e `lto = "thin"` reduz bem o tamanho.

## 9. Sugestões para um cliente de banco de dados (não testadas aqui)

- **Driver e runtime:** se o driver for async (`sqlx`, `tokio-postgres`), ligue a feature `tokio` do iced para que
  `Task::perform` rode no runtime certo; com drivers síncronos (`postgres`, `rusqlite`, `mysql`), o padrão de
  threads acima basta. Não misture os dois sem motivo.
- **Uma conexão por aba de consulta, numa thread dona dela,** recebendo pedidos por canal. Assim cancelar
  (`pg_cancel_backend`, `KILL QUERY`) e mostrar "rodando há 12 s" ficam simples, e uma consulta lenta não trava
  as outras.
- **Resultados grandes:** traga em páginas (cursor ou `LIMIT/OFFSET`) e desenhe só as linhas visíveis (grade
  própria num `canvas`, ou `scrollable` com janela manual a partir de `on_scroll`). Guarde as células já
  formatadas como texto no `update`, não no `view`.
- **Editor de SQL:** o `iced-code-editor` com `two-face` já colore SQL; autocomplete de tabelas e colunas pode vir
  de um LSP de SQL (ex.: `sqls`) ou do próprio esquema lido do banco.
- **Senhas:** use o chaveiro do sistema (crate `keyring`), nunca o arquivo de config em texto.
- **Testes:** suba o banco num contêiner (ou SQLite em memória) e teste o `update` com mensagens, como acima.
