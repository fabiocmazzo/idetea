## Terminal
terminal-start-error = Terminal: não foi possível iniciar o shell: { $error }
cmd-terminal-focus = Terminal: abrir e focar
cmd-terminal-new = Terminal: novo terminal
cmd-terminal-close = Terminal: fechar o terminal
cmd-terminal-restart = Terminal: reiniciar o shell
terminal-ended-n = Terminal { $number } encerrado
terminal-not-running = O shell não está rodando.
terminal-start = Iniciar terminal

## Tarefas
terminal-task-error = Tarefa: não foi possível iniciar: { $error }
terminal-tab-n = Terminal { $number }
terminal-close = Fechar o terminal
terminal-tab-task = Tarefa: { $name }
terminal-close-task = Fechar tarefa

## Terminais pedidos por agentes
terminal-tab-agent = Agente: { $title }
terminal-tab-request = { $count ->
    [one] Pedido de terminal
   *[other] { $count } pedidos de terminal
}
agent-request-notify = Um agente pede um terminal: { $title }. Veja no painel do terminal.
agent-request-auto = Pedido de terminal "{ $title }" rodando sem confirmar: está no [agents] allow do seu config ("{ $entry }").
agent-request-someone = Um agente
agent-request-title = { $who } pede um terminal: { $title }
agent-request-command = Comando (roda exatamente isto):
agent-request-cwd = Na pasta: { $cwd }
agent-request-worktree = Roda num worktree próprio: branch { $branch }, em { $path } (criado a partir do HEAD, se o branch não existir)
agent-worktree-kept = O worktree { $path } ficou: { $error }
agent-request-warning = Roda com as suas permissões, como se você digitasse. Permita só o que reconhece.
agent-request-allow = Permitir e rodar
agent-request-refuse = Recusar
agent-request-more = { $count ->
    [one] E mais um pedido depois deste.
   *[other] E mais { $count } pedidos depois deste.
}
agent-done = Terminou com código { $code }. Enter fecha a aba.
run-bad-arg = idetea run: "{ $arg }" não vale.
run-usage = Uso: idetea run [--title T] [--name nome] [--cwd pasta] [--worktree branch] [--keep] [--no-wait] [--timeout segundos] -- <comando>; idetea run --wait <nome>; idetea run --stop <nome>
run-busy = idetea run: já existe um pedido "{ $name }" (ou o status dele); escolha outro --name ou espere com idetea run --wait { $name }
run-asked = idetea run: pedido "{ $name }" no painel do terminal; esperando o usuário permitir e o comando terminar
run-exited = idetea run: "{ $name }" terminou com código { $code }
run-refused = idetea run: o usuário recusou "{ $name }"
run-invalid = idetea run: pedido "{ $name }" inválido: { $error }
run-ended = idetea run: "{ $name }" acabou sem código ({ $state })
run-branch = idetea run: o trabalho ficou no branch { $branch }
run-timeout = idetea run: o prazo acabou e "{ $name }" segue no terminal (idetea run --wait { $name } volta a esperar; --stop encerra)
