ui-panel = Interfaces
cmd-ui-show = Interfaces: mostrar o painel
ui-empty = Nenhuma interface aberta. Um agente pode criar uma escrevendo um arquivo em .idetea/ui/ (veja docs/interfaces/formato.md). Para ativar ou desligar: [extensions] interfaces = {"{"} enabled = true {"}"} na configuração.
ui-tracked = { $path } está versionado no git e foi ignorado por segurança: interfaces vêm só do agente rodando nesta máquina, nunca de um repositório clonado.
ui-invalid = Documento inválido: { $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = { $count ->
    [one] Resposta enviada; aguardando o agente ler.
   *[other] { $count } respostas enviadas; aguardando o agente ler.
}
ui-received = Resposta recebida pelo agente.
ui-agent-waiting = O agente está esperando a resposta.
ui-agent-stopped = { $minutes ->
    [0] O agente acabou de parar de esperar: a resposta fica guardada até ele voltar a olhar.
   *[other] O agente parou de esperar há { $minutes } min: a resposta fica guardada até ele voltar a olhar.
}
ui-new = Nova interface do agente: { $title }
ui-updated = Interface atualizada: { $title }
ui-open-later = O agente pediu para abrir { $path }: está no painel Interfaces (não abri para não atrapalhar a digitação).
ui-write-failed = Não deu para gravar a resposta: { $error }
ui-prepare-failed = Não deu para preparar a pasta .idetea/ui: { $error }
ui-not-a-file = não é um arquivo comum
ui-surface-mismatch = o campo surface é "{ $found }", mas o arquivo se chama { $expected }.json
cmd-ui-teach = Interfaces: ensinar os agentes deste projeto
ui-teach-hint = Para um agente de código saber usar este painel, grave as instruções no AGENTS.md do projeto.
ui-teach-button = Ensinar os agentes deste projeto
ui-teach-title = Ensinar os agentes deste projeto?
ui-teach-detail = A IDE vai gravar as instruções do painel Interfaces em: { $files }. O texto fica entre marcadores; rodar de novo só atualiza esse trecho e não mexe no resto. São arquivos do projeto: revise antes do commit.
ui-teach-file-new = { $name } (novo)
ui-teach-write = Gravar
ui-cancel = Cancelar
ui-teach-up-to-date = As instruções para agentes já estão atualizadas.
ui-teach-done = Instruções gravadas em { $files }.
ui-teach-failed = Não deu para gravar as instruções: { $error }
cmd-ui-new-project = Projeto: novo…
new-title = Novo projeto
new-intro = A IDE cria a pasta, o git, as skills do modelo e as instruções do painel, e abre o projeto numa janela nova. Quem cria o projeto de verdade é o seu agente, seguindo o pedido inicial do **AGENTS.md**. Com um agente escolhido abaixo, a janela nova pede para abri-lo num terminal da IDE (você confirma o comando).
new-field-parent = Criar dentro de
new-field-name = Nome da pasta
new-field-name-hint = ex.: meu-app
new-field-model = Modelo
new-field-description = O que é o projeto (opcional)
new-field-description-hint = uma frase; vai no pedido inicial para o agente
new-field-git = Criar o repositório git
new-field-agent = Agente a abrir num terminal da IDE
new-agent-none = Nenhum (abro eu)
new-agent-title = Agente: { $agent }
new-bad-agent = nome de agente que não vale: "{ $agent }" (só letras, dígitos, - e _)
new-create = Criar e abrir
new-model = { $model ->
    [vazio] Vazio (o agente pergunta o que é)
    [rust] Rust
    [site] Site
    [python] Python
    [fastapi] FastAPI (API web em Python)
   *[other] { $model }
}
new-creating = Criando { $dir } (as skills vêm pela rede)…
new-created = Projeto criado em { $dir }; abrindo numa janela nova.
new-open-failed = Projeto criado em { $dir }, mas não deu para abrir a janela: { $error }
new-failed = Não deu para criar o projeto: { $error }
new-bad-model = modelo desconhecido: { $model }
new-bad-dir = pasta que não vale: { $dir } (use um caminho completo)
new-bad-name = nome de pasta que não vale: "{ $name }"
new-not-empty = { $dir } já existe e tem arquivos: escolha outro nome
new-done-dir = pasta: { $dir }
new-done-git = git: repositório criado
new-done-agents = AGENTS.md (com o pedido inicial) e CLAUDE.md, com as instruções do painel
new-done-agent = pedido de terminal para { $agent }: confirme na janela nova
new-open-hint = Pronto. Abra com: idetea { $dir }
new-usage = idetea new: "{ $arg }" não vale. Uso: idetea new <pasta> [--modelo id] [--descricao texto] [--sem-git] [--agente nome]; modelos: { $models }
cmd-ui-fix = Interfaces: consertar este projeto
ui-fix-title = Consertar este projeto?
ui-fix-will = A IDE vai:
ui-fix-wont = Só para você saber (não mexo):
ui-fix-button = Consertar
ui-fix-nothing = Nada a consertar nas interfaces deste projeto.
ui-fix-done = { $count ->
    [one] Consertado: 1 item.
   *[other] Consertado: { $count } itens.
}
ui-fix-failed = Não deu para consertar: { $error }
ui-fix-hint = Para consertar os itens com "-": idetea ui fix --apply
ui-fix-usage = idetea ui fix: argumento desconhecido "{ $arg }" (uso: idetea ui fix [--apply])
ui-fix-abandoned = apagar { $path }: ninguém espera a resposta há { $since }
ui-fix-unread = { $path }: { $count ->
    [one] 1 resposta que o agente não leu
   *[other] { $count } respostas que o agente não leu
} (fica guardada; peça ao agente para ler, ou apague à mão)
ui-fix-leftover = apagar { $path }: sobrou de uma escrita ou espera que não terminou
ui-fix-check = regravar .idetea/ui/check: ele chama { $target }, que não existe mais
ui-fix-instructions-older = regravar as instruções do { $file }: estão na versão { $found }, mais velha que a desta IDE
ui-fix-instructions-unversioned = regravar as instruções do { $file }: são de uma IDE antiga, sem a linha de versão
ui-fix-instructions-newer = { $file } tem instruções de uma IDE mais nova ({ $found }): abra o projeto nela (não volto atrás)
ui-outdated = Esta IDE está desatualizada (foi recompilada depois de abrir): as respostas avisam o agente. Paleta: "Reiniciar a IDE".
ui-audio-failed = Não deu para tocar o som: { $error }
ui-check-ok = { $file }: ok ({ $count ->
        [one] 1 componente
       *[other] { $count } componentes
    })
ui-check-error = { $file }: erro: { $error }
ui-check-warning = aviso: { $warning }
ui-check-media = mídia: { $path }: { $problem }
ui-check-name = o nome do arquivo não vale ("{ $name }"): letras minúsculas, dígitos, _ e -, terminando em .json
ui-check-usage = uso: idetea ui check <arquivo.json>…
ui-check-instructions-older = instruções: o { $file } tem a versão { $found } e esta IDE tem a { $ours }: peça ao usuário para rodar "Interfaces: ensinar os agentes deste projeto" e releia o arquivo antes de publicar
ui-check-instructions-newer = instruções: o { $file } tem a versão { $found }, de uma IDE mais nova que esta ({ $ours }): o painel aberto pode não entender o que o arquivo ensina
ui-check-instructions-unversioned = instruções: o { $file } tem um bloco antigo, sem a linha de versão; esta IDE tem a { $ours }: peça ao usuário para rodar "Interfaces: ensinar os agentes deste projeto" e releia o arquivo antes de publicar
ui-teach-newer-file = { $name } (versão { $version })
ui-teach-newer = Atenção: { $files } foi gravado por uma IDE mais nova que esta (que tem a versão { $ours } e não conhece aquela). Gravar volta as instruções para a versão desta IDE; o normal é abrir o projeto com a IDE nova.
ui-teach-write-anyway = Gravar mesmo assim
ask-title = Pergunta
ask-choose = Escolha
ask-ok = Confirmar
ask-yes = Sim
ask-no = Não
ask-not-object = o documento precisa ser um objeto JSON
ask-bad-arg = idetea ask: "{ $arg }" não vale.
ask-usage = Uso: idetea ask <documento.json | -> | --text T [--options "a|b|c" | --confirm | --input Rótulo] [--title T] [--name nome] [--value /ponteiro] [--json] [--keep] [--timeout segundos]; idetea ask --resume <nome>; idetea ask --close <nome>
ask-invalid = idetea ask: o painel recusaria a pergunta: { $error }
ask-no-such = idetea ask: não há pergunta "{ $name }" no painel
ask-timeout = idetea ask: sem resposta no prazo; a pergunta "{ $name }" continua no painel (idetea ask --resume { $name } volta a esperar)
ask-outdated = idetea ask: a resposta veio de uma IDE recompilada depois de aberta (ide.outdated); se estiver testando uma mudança, peça para reiniciar a IDE
ask-comment = comentário: { $comment }

# Histórico das interfaces (RFC 0007)
ui-history-panel = Histórico das interfaces
cmd-ui-history = Interfaces: histórico
cmd-ui-clear-history = Interfaces: apagar o histórico
ui-history-failed = Não deu para gravar no histórico: { $error }
ui-history-cleared = { $count ->
    [one] 1 entrada apagada do histórico.
   *[other] { $count } entradas apagadas do histórico.
}
ui-history-link = Histórico ({ $count })
ui-history-empty = O histórico está vazio.
ui-history-clear-title = Apagar o histórico?
ui-history-clear-detail = { $count ->
    [one] A entrada do histórico deste projeto (.idetea/history/) vai ser apagada.
   *[other] As { $count } entradas do histórico deste projeto (.idetea/history/) vão ser apagadas.
} As interfaces abertas no painel continuam. Não dá para desfazer.
ui-history-clear-button = Apagar o histórico
ui-history-search = Buscar no título, nas respostas e nos comentários
ui-history-open = Abertas ({ $count })
ui-history-waiting-you = esperando você
ui-history-answered-open = respondida, ainda aberta
ui-history-today = Hoje
ui-history-yesterday = Ontem
ui-history-none = Nenhuma pergunta respondida ainda. Cada resposta dada no painel Interfaces fica guardada aqui (em .idetea/history/, fora do git).
ui-history-no-match = Nada encontrado.
ui-history-back = ← Voltar
ui-history-record-decision = Registrar como decisão
ui-history-decision-recorded = Decisão registrada em { $path }: revise e complete as consequências.
ui-history-decision-failed = Não deu para registrar a decisão: { $error }
adr-context = Contexto e problema
adr-options = Opções consideradas
adr-decision = Decisão
adr-chosen = Escolhido
adr-consequences = Consequências
adr-consequences-todo = _A preencher: o que fica melhor, o que fica pior._
adr-origin = Registrada a partir da pergunta "{ $name }" do painel Interfaces da IDE TEA, respondida em { $date }.
ui-history-shown = Mostrada: { $time }
ui-history-answered = Respondida: { $time }
ui-history-closed = Fechada sem resposta: { $time }
ui-history-read = Lida pelo agente: { $time }
ui-history-button = Botão: { $action }
ui-history-not-read = O agente não leu esta resposta (o arquivo ainda está em .idetea/ui/responses/, ou foi apagado com a IDE fechada).
ui-history-not-read-short = não lida
ui-history-unanswered = sem resposta
ui-history-media-changed = { $path } mudou ou sumiu depois da resposta: o que aparece aqui é a versão de agora.
ui-history-no-audio = o histórico não toca sons
history-cli-comment = comentário
history-cli-not-read = o agente não leu esta resposta
history-cli-unanswered = sem resposta (o agente apagou a pergunta)
history-cli-bad = idetea history: "{ $arg }" não vale.
history-cli-usage = Uso: idetea history [--name nome] [--since AAAA-MM-DD] [--last N | --all] [--json]
history-cli-empty = idetea history: nada no histórico deste projeto.
