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
ui-write-failed = Não deu para gravar a resposta: { $error }
ui-prepare-failed = Não deu para preparar a pasta .idetea/ui: { $error }
ui-not-a-file = não é um arquivo comum
ui-surface-mismatch = o campo surface é "{ $found }", mas o arquivo se chama { $expected }.json
