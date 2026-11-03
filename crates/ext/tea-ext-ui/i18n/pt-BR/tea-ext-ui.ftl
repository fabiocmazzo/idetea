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
