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
