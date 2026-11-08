# RFC 0013 — Avaliação de Portabilidade para macOS e Windows

- **Data:** 2026-11-08
- **Status:** Implementada
- **Escopo:** Mapeamento exato no código-fonte de tudo o que impede ou degrada a execução da IDE TEA em **macOS** (`aarch64-apple-darwin` / `x86_64-apple-darwin`) e **Windows** (`x86_64-pc-windows-msvc`), com plano incremental de adaptação.

---

## 1. Resumo Executivo

A boa notícia arquitetural é que as dependências pesadas da IDE TEA (`iced` 0.14 com `wgpu`, `iced_term` / `alacritty_terminal`, `syntect`, `trash`, `rfd`, `notify`, `hayro`, `image`, `wasmtime`) **já são multiplataforma** e suportam Linux, macOS e Windows nativamente.

Os pontos que hoje prendem a IDE TEA ao Linux estão concentrados na camada de integração com o sistema operacional da própria IDE:

| Plataforma |Compila hoje (`cargo build`)? | Esforço estimado | Resumo do estado atual |
|---|---|---|---|
| **macOS** (`aarch64` / `x86_64`) | **Sim (quase 100%)** | **Baixo (~1 dia)** | Como o macOS é `cfg(unix)`, `/bin/sh`, permissões POSIX, `setsid`, `afplay` (`player.rs`) e `open` (`os.rs`) **já funcionam**. Precisa apenas de 5 ajustes de usabilidade/runtime (`Cmd` vs `Ctrl`, `PATH` do Finder, `lldb-dap` no lugar do `gdb`, `kill(pid, 0)` sem `/proc` e bundle `.app`). |
| **Windows** (`x86_64-pc-windows-msvc`) | **Não compila hoje** | **Médio (~2 a 3 dias)** | Falha ao compilar em **4 arquivos** que usam `std::os::unix::*` ou `libc` sem `#[cfg(unix)]`. Em runtime, precisa de 6 adaptações: diretórios (`APPDATA`/`USERPROFILE`), busca no `PATH` com `.exe`/`.cmd`, remoção da dependência de `/bin/sh` nos terminais de task/agente, verificação de PID, encerramento de árvore de processos e áudio. |

---

## 2. Diagnóstico para Windows (`x86_64-pc-windows-msvc`)

### 2.1 Erros diretos de compilação no Windows (Falta de `#[cfg(unix)]`)

Hoje `cargo check --target x86_64-pc-windows-msvc` falha exatamente nestes pontos:

1. **`crates/ext/tea-ext-ui/src/player.rs` (linhas 21–24, 117–143, 149–158):**
   - Linha 22: `use std::os::unix::fs::PermissionsExt;` dentro de `executable()`.
   - Linha 120: `std::os::unix::process::CommandExt::process_group(&mut cmd, 0);` dentro de `spawn()`.
   - Linha 139: `unsafe { libc::kill(-pid, libc::SIGKILL) };` dentro de `kill()` (além de `libc` só estar em `[target.'cfg(unix)'.dependencies]` ou nem estar no `Cargo.toml` para Windows).
   - Linha 150 (testes): `use std::os::unix::fs::PermissionsExt;` e script `#!/bin/sh`.
2. **`crates/ext/tea-ext-ui/src/store.rs` (linhas 70–79, 538):**
   - Linha 75: `use std::os::unix::fs::PermissionsExt;` em `write_check_script()` sem `#[cfg(unix)]`.
   - Linha 538 (testes): `PermissionsExt` sem `#[cfg(unix)]`.
3. **`crates/ext/tea-ext-ui/src/novo.rs` (linhas 46–56):**
   - Linha 51: `use std::os::unix::fs::PermissionsExt;` em `in_path()` sem `#[cfg(unix)]`.
4. **`crates/tea-app/src/debug_build.rs` (linhas 123–133, 211):**
   - Linha 124: `use std::os::unix::fs::PermissionsExt;` em `zig_executables()` sem `#[cfg(unix)]`.
   - Linha 211 (testes): `PermissionsExt` sem `#[cfg(unix)]`.
5. **Testes unitários com `std::os::unix::fs::symlink` sem `#[cfg(unix)]`:**
   - `crates/tea-skills/src/lib.rs:549`
   - `crates/ext/tea-ext-ui/src/history.rs:501`
   - `crates/ext/tea-ext-ui/src/lib.rs:1561–1562`

---

### 2.2 Adaptações de Runtime no Windows

Mesmo após compilar, 6 comportamentos precisam ser adaptados para que a IDE funcione de ponta a ponta no Windows:

#### 1. Pastas do usuário (`HOME` vs `USERPROFILE` / `APPDATA` / `LOCALAPPDATA`)
- **Problema:** no Windows, a variável `HOME` não é definida por padrão pelo sistema (usa-se `USERPROFILE`, `APPDATA` e `LOCALAPPDATA`). Enquanto `crates/tea-core/src/config.rs:504` já tem `.or_else(|| std::env::var_os("APPDATA"))`, vários outros módulos falham se `HOME` não existir:
  - `crates/tea-app/src/ui_state.rs:101` (`state_dir`)
  - `crates/tea-app/src/recent.rs:69` (abreviação `~`)
  - `crates/tea-app/src/main.rs:4534` (expansão de `~/...`)
  - `crates/ext/tea-ext-ui/src/lib.rs:156` (`state_dir`)
  - `crates/ext/tea-ext-ui/src/novo.rs:216, 260` (pasta padrão de novos projetos)
  - `crates/tea-skills/src/lib.rs:260` (`cache_dir`)
  - `crates/tea-plugins/src/lib.rs:27` (`plugins_dir`)
- **Solução recomendada:** centralizar em `crates/tea-core/src/os.rs` cinco funções canônicas usadas por todas as crates:
  - `home_dir()` → `HOME` (Unix) ou `USERPROFILE` (Windows)
  - `config_dir()` → `XDG_CONFIG_HOME` → `APPDATA` (Windows) → `~/.config`
  - `state_dir()` → `XDG_STATE_HOME` → `LOCALAPPDATA` (Windows) → `~/.local/state`
  - `data_dir()` → `XDG_DATA_HOME` → `LOCALAPPDATA` (Windows) → `~/.local/share`
  - `cache_dir()` → `XDG_CACHE_HOME` → `LOCALAPPDATA\Temp` / `LOCALAPPDATA` (Windows) → `~/.cache`

#### 2. Descoberta de executáveis no `PATH` (`.exe`, `.cmd`, `.bat`)
- **Problema:** no Windows, executáveis e wrappers de pacotes npm/pip têm extensão (`rust-analyzer.exe`, `pyright.cmd`, `typescript-language-server.cmd`, `claude.cmd`, `git.exe`).
  - `crates/tea-lsp/src/lib.rs:254` (`command_in_path`) faz apenas `dir.join(cmd).is_file()`, o que retorna `false` para `rust-analyzer` na tela de Configurações de LSP no Windows.
  - `crates/ext/tea-ext-ui/src/novo.rs:46` (`in_path`) procura `claude` / `codex` sem extensão e testa `0o111`.
  - `crates/tea-app/src/debug_build.rs:123` (`zig_executables`) deve procurar arquivos `.exe` em `zig-out/bin/` no Windows.
- **Solução recomendada:** criar `tea_core::os::find_in_path(name: &str) -> Option<PathBuf>`, que no Unix verifica `is_file() && mode & 0o111 != 0`, e no Windows verifica `name` e, se não tiver extensão, testa as extensões de `PATHEXT` (`.exe`, `.cmd`, `.bat`, `.com`).

#### 3. Terminal Integrado, Tasks e Terminais de Agente (`/bin/sh` hardcoded)
- **Problema:**
  1. Em `crates/ext/tea-ext-terminal/src/lib.rs:204`, abrir uma aba de terminal usa `std::env::var("SHELL").unwrap_or("/bin/sh")`. No Windows, deve usar `powershell.exe` (ou `pwsh.exe` se existir, com fallback para `%COMSPEC%` / `cmd.exe`).
  2. Em `crates/ext/tea-ext-terminal/src/lib.rs:247–248` (`run_task`), `488–491` (`AGENT_SCRIPT`) e `crates/tea-app/src/debug_build.rs:68` (`Build::Shell`), a IDE invoca `/bin/sh -c` com scripts POSIX (`TASK_SCRIPT` e `AGENT_SCRIPT`) que usam `trap`, `printf` e `mv` para gravar `.idetea/agents/<nome>.status.json`.
  3. Em `crates/ext/tea-ext-ui/src/store.rs:70`, `.idetea/ui/check` é gravado apenas como script `#!/bin/sh`.
- **Solução arquitetural recomendada (limpa e 100% em Rust):**
  - Substituir os scripts shell inline `TASK_SCRIPT` e `AGENT_SCRIPT` por um subcomando interno oculto do próprio binário `idetea` (ex.: `idetea __task-runner` e `idetea __agent-runner`).
  - Assim, o `iced_term` abre o `idetea __agent-runner`, que em **Rust puro** grava o `.status.json` (`running` → executa o comando no shell do SO: `/bin/sh -c` no Unix ou `cmd.exe /C` / `powershell.exe -Command` no Windows → captura o exit code → grava `exited` atomicamente → aguarda `Enter`).
  - Isso elimina de uma vez qualquer fragilidade de escape de shell/sinais em `AGENT_SCRIPT` e funciona de forma idêntica em Linux, macOS e Windows.

#### 4. Verificação de PID vivo sem `/proc`
- **Problema:**
  - `crates/tea-app/src/backup.rs:130` (`is_running`) lê `/proc/{pid}/exe`. Fora do Linux, retorna sempre `false`, fazendo qualquer backup de outra janela parecer órfão.
  - `crates/tea-app/src/ide_state.rs:106` (`alive`) e `crates/ext/tea-ext-terminal/src/agents.rs:286` (`alive`) olham se `/proc` existe; se não existir (macOS e Windows), assumem `true` para sempre (se a IDE cair no macOS ou Windows, `idetea status` continuará achando que o PID morto está vivo!).
- **Solução recomendada:** criar `tea_core::os::process_alive(pid: u32) -> bool`:
  - Em `#[cfg(unix)]` (Linux e macOS): `unsafe { libc::kill(pid as i32, 0) == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM) }`.
  - Em `#[cfg(windows)]`: consultar via Win32 `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)` + `GetExitCodeProcess` (`STILL_ACTIVE`).

#### 5. Encerramento de árvore de processos (`git`, `plugins`, `player`)
- **Problema:** `crates/ext/tea-ext-git/src/git.rs:544` e `crates/tea-plugins/src/process.rs:91` matam apenas o processo pai no Windows (`child.kill()`), deixando eventuais filhos (`ssh.exe`, `git-credential-manager.exe`) órfãos.
- **Solução recomendada:** no bloco `#[cfg(windows)]`, executar `Command::new("taskkill").args(["/F", "/T", "/PID", &child.id().to_string()])` antes de `child.kill()`.

#### 6. Tocador de áudio e abertura de links no Windows
- Em `crates/ext/tea-ext-ui/src/player.rs`: no Windows, tocar arquivos `.wav` usando `powershell -NoProfile -NonInteractive -Command "(New-Object Media.SoundPlayer '<path>').PlaySync()"` (zero dependências externas).
- Em `crates/tea-core/src/os.rs:23`: para `open_url` no Windows, usar `rundll32 url.dll,FileProtocolHandler <url>` para evitar que URLs com `&` ou `mailto:` falhem no `explorer`.

---

## 3. Diagnóstico para macOS (`aarch64-apple-darwin` / `x86_64-apple-darwin`)

Como o macOS é Unix (`cfg(unix)`), todo o código POSIX já compila e roda. Além disso, `player.rs` já usa `afplay` no macOS e `os.rs` já usa `open`. O que falta são **5 ajustes específicos da plataforma**:

### 3.1 Atalhos de Teclado (`Cmd` vs `Ctrl`)
- **Onde fica:** `crates/tea-core/src/keymap.rs` e `crates/tea-editor/src/lib.rs`.
- **Problema:** no macOS, os atalhos padrão do sistema usam `Cmd` (`Super` / `Logo` no `iced`), como `Cmd+S`, `Cmd+P`, `Cmd+Shift+P`, `Cmd+O`, `Cmd+W`, `Cmd+F`, `Cmd+Z`, `Cmd+C`, `Cmd+V`, `Cmd+=`, `Cmd+-`, enquanto o terminal integrado usa `Cmd+C`/`Cmd+V` para copiar/colar e mantém `Ctrl+C` para enviar `SIGINT` ao processo no PTY.
- **Solução:** em `Keymap::default()`, mapear os atalhos padrão com `#[cfg(target_os = "macos")]` para `cmd+...` (ou aceitar um modificador lógico `mod+...` que resolve para `Cmd` no macOS e `Ctrl` no Linux/Windows).

### 3.2 Variável `PATH` quando aberta pelo Finder / Dock / Spotlight
- **Onde fica:** `crates/tea-app/src/main.rs` (início do `main()`).
- **Problema:** no macOS, aplicativos GUI abertos fora do terminal herdam um `PATH` mínimo (`/usr/bin:/bin:/usr/sbin:/sbin`). Sem `/opt/homebrew/bin`, `/usr/local/bin` e `~/.cargo/bin`, a IDE não encontra `cargo`, `rust-analyzer`, `zig`, `git` do Homebrew nem `claude`.
- **Solução:** no início de `main()` sob `#[cfg(target_os = "macos")]`, executar `/usr/libexec/path_helper -s` ou acrescentar `~/.cargo/bin`, `~/.local/bin`, `/opt/homebrew/bin` e `/usr/local/bin` ao `PATH` do processo.

### 3.3 Depuração no macOS (`lldb-dap` em vez de `gdb`)
- **Onde fica:** `crates/tea-dap/src/lib.rs:33` (`AdapterConfig::gdb()`).
- **Problema:** o `gdb` não tem suporte oficial em Macs com Apple Silicon (`aarch64-apple-darwin`) e exige assinatura de certificado de sistema no macOS Intel. Em compensação, o macOS traz nativamente o `lldb-dap` (via Xcode Command Line Tools: `xcrun -f lldb-dap` ou Homebrew LLVM), que fala **exatamente o mesmo protocolo DAP** que o `tea-dap` já usa!
- **Solução:** adicionar `AdapterConfig::default_for_os()` em `tea-dap`: no macOS, procurar `lldb-dap` (ou `xcrun --find lldb-dap`) primeiro, com fallback para `gdb`.

### 3.4 Verificação de PID vivo (`kill(pid, 0)`)
- Como visto na seção 2.2 (item 4), trocar a checagem de `/proc/<pid>` em `ide_state.rs`, `agents.rs` e `backup.rs` por `libc::kill(pid as i32, 0)` resolve imediatamente a detecção de processos vivos no macOS.

### 3.5 Empacotamento `.app` e `--install-desktop` no macOS
- **Onde fica:** `crates/tea-app/src/desktop.rs` e `packaging/dist.sh`.
- No macOS, `idetea --install-desktop` pode criar a estrutura de bundle `~/Applications/IDE TEA.app/Contents/{Info.plist,MacOS/idetea,Resources/idetea.icns}` (ou um symlink em `/usr/local/bin/idetea`), e o script de release pode gerar `idetea-macos-aarch64.tar.gz` contendo o `.app` e o binário CLI.

---

## 4. Plano Incremental de Implementação Sugerido

Podemos dividir o trabalho em **3 etapas pequenas e testáveis**:

1. **Etapa 1 — Camada `tea_core::os` e compilação limpa em Windows e macOS (`cargo check` multiplataforma):**
   - Centralizar `home_dir`, `config_dir`, `state_dir`, `data_dir`, `cache_dir`, `find_in_path` (com suporte a `PATHEXT` no Windows) e `process_alive` (com `kill(pid, 0)` no Unix e Win32/tasklist no Windows) em `tea-core::os`.
   - Proteger com `#[cfg(unix)]` e adicionar os equivalentes `#[cfg(windows)]` nos 4 arquivos que hoje quebram a compilação no Windows (`player.rs`, `store.rs`, `novo.rs`, `debug_build.rs`) e nos testes unitários.
2. **Etapa 2 — Terminal, Tasks, Agentes e Debug multiplataforma:**
   - Adaptar `tea-ext-terminal` para escolher o shell padrão do SO (`SHELL` / `/bin/sh` no Unix; `pwsh.exe` / `powershell.exe` / `cmd.exe` no Windows) e usar o próprio `idetea` como runner multiplataforma de tasks e terminais de agente.
   - Adicionar suporte a `lldb-dap` em `tea-dap` para macOS, enriquecimento de `PATH` no macOS e atalhos `Cmd` no `keymap.rs`.
3. **Etapa 3 — CI e Releases multiplataforma (`.github/workflows/ci.yml` e `release.yml`):**
   - Adicionar jobs de `cargo test` para `macos-14` (ARM64) e `windows-latest` (x86_64) no GitHub Actions, e empacotamento `.zip` (Windows) e `.tar.gz` / `.app` (macOS) no `release.yml`.

---

## 5. Andamento

- **Etapa 1 concluída:**
  - `crates/tea-core/src/os.rs` expandido com `home_dir()`, `config_dir()`, `state_dir()`, `data_dir()`, `cache_dir()`, `is_executable()`, `find_in_path()` (com suporte a `PATHEXT` no Windows), `process_alive(pid)` (`/proc` no Linux, `libc::kill(pid, 0)` no macOS/Unix, `tasklist` no Windows) e `open_url()` com `rundll32 url.dll,FileProtocolHandler` no Windows.
  - Caminhos de configuração, estado, dados, cache e busca em `PATH` unificados em `tea-core`, `tea-app`, `tea-lsp`, `tea-skills`, `tea-plugins`, `tea-ext-ui` e `tea-ext-terminal`.
  - Chamadas e testes específicos de Unix (`PermissionsExt`, `CommandExt`, `symlink`, `libc::kill`) protegidos por `#[cfg(unix)]`, com equivalentes `#[cfg(windows)]` (`cmd /C`, `taskkill /F /T /PID`, `.idetea/ui/check.cmd`, `PATHEXT`).
- **Etapa 2 concluída:**
  - `crates/ext/tea-ext-terminal/src/lib.rs`: `default_shell()` seleciona `$SHELL`/`/bin/sh` no Unix e `pwsh.exe`/`powershell.exe`/`%COMSPEC%`/`cmd.exe` no Windows; `run_task` e `allow` (terminais de tarefas e de agentes) possuem scripts equivalentes para Windows (`WINDOWS_TASK_SCRIPT` e `WINDOWS_AGENT_SCRIPT` via PowerShell) mantendo gravação atômica de status e exibição do código de saída.
  - `crates/tea-dap/src/lib.rs` e `crates/tea-app/src/debug.rs`: adicionado `AdapterConfig::lldb_dap()` e `AdapterConfig::detect()`, priorizando `lldb-dap` no macOS (incluindo o caminho do Xcode Command Line Tools) e fazendo fallback automático entre `gdb` e `lldb-dap` em todos os sistemas.
  - `crates/tea-core/src/os.rs` e `crates/tea-app/src/main.rs`: `ensure_gui_path()` garante que `~/.cargo/bin`, `~/.local/bin` e, no macOS, `/opt/homebrew/bin`, `/opt/homebrew/sbin` e `/usr/local/bin` estejam no `PATH` quando a IDE é iniciada pelo Finder/Dock ou menu do sistema.
  - `crates/tea-core/src/keymap.rs`: suporte ao modificador lógico `mod`, fallback automático de `Cmd+<tecla>` para `Ctrl+<tecla>` no macOS (`macos_ctrl_fallback`) e suporte a `Cmd+Shift` / `Cmd+PageUp/PageDown` em `wins_over_panels` no macOS.
- **Etapa 3 concluída:**
  - `.github/workflows/ci.yml`: matriz de CI expandida para `ubuntu-latest`, `macos-latest` (ARM64) e `windows-latest` (x86_64) executando `cargo clippy` e `cargo test`.
  - `.github/workflows/release.yml`: adicionados jobs `build-macos` (`idetea-<version>-aarch64-macos.tar.gz` com `IDE TEA.app` assinado via `codesign --sign -` e binário CLI) e `build-windows` (`idetea-<version>-x86_64-windows.zip` com `idetea.exe` e suporte condicional à SignPath Foundation), integrados ao job `publish` com `SHA256SUMS`.
  - `crates/tea-app/src/desktop.rs`: suporte a `--install-desktop` e `--uninstall-desktop` no macOS criando/removendo o bundle `~/Applications/IDE TEA.app` com assinatura ad-hoc.

---

## 6. Assinatura de Código (Code Signing) Gratuita para Open Source

### 6.1 macOS — Assinatura Ad-Hoc (`codesign --sign -`)
- No Apple Silicon (`aarch64-apple-darwin`), o kernel exige que todo executável Mach-O e bundle `.app` tenha uma assinatura válida no Mach-O header.
- Tanto o job `build-macos` em `.github/workflows/release.yml` quanto `idetea --install-desktop` em `crates/tea-app/src/desktop.rs` executam `codesign --force --deep --sign - "IDE TEA.app"`, que é **100% gratuito**, não exige conta paga da Apple e evita o erro *"IDE TEA.app está danificado"* ao alterar o bundle.
- Para quem baixa o `.tar.gz` diretamente pelo navegador no macOS (sem certificado pago de US$ 99/ano da Apple), basta remover o atributo de quarentena uma única vez (`xattr -cr "IDE TEA.app"`) ou instalar via `cargo install` / Homebrew.

### 6.2 Windows — SignPath Foundation (Gratuito para OSS)
- A **[SignPath Foundation](https://signpath.org/)** oferece assinatura de código **100% gratuita** para projetos open-source com licença aprovada pela OSI (como a nossa `Apache-2.0`) e builds reprodutíveis no GitHub Actions, com reputação imediata no **Microsoft SmartScreen**.
- O workflow `.github/workflows/release.yml` já contém o passo condicional `signpath/github-action-submit-signing-request@v1` que é ativado automaticamente assim que a variável de repositório `SIGNPATH_ORGANIZATION_ID` e o segredo `SIGNPATH_API_TOKEN` forem configurados no GitHub.
