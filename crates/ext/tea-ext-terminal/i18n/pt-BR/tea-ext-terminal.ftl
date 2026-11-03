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
agent-request-someone = Um agente
agent-request-title = { $who } pede um terminal: { $title }
agent-request-command = Comando (roda exatamente isto):
agent-request-cwd = Na pasta: { $cwd }
agent-request-warning = Roda com as suas permissões, como se você digitasse. Permita só o que reconhece.
agent-request-allow = Permitir e rodar
agent-request-refuse = Recusar
agent-request-more = { $count ->
    [one] E mais um pedido depois deste.
   *[other] E mais { $count } pedidos depois deste.
}
agent-done = Terminou com código { $code }. Enter fecha a aba.
