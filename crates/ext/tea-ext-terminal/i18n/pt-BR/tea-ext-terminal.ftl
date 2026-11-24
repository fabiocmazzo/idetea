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

## Estado das abas (RFC 0024)
terminal-state-running = Rodando
terminal-state-unseen = Saída nova desde a última vez que você viu
terminal-state-attention = Pediu sua atenção (permissão ou resposta)
terminal-state-exited = Terminou com o código { $code }
terminal-state-bell = Tocou o sino
terminal-state-turn-done = Terminou a vez e espera você
cmd-terminal-next = Terminal: próxima aba
cmd-terminal-prev = Terminal: aba anterior
cmd-terminal-goto = Terminal: ir para…
cmd-terminal-next-attention = Terminal: próxima aba esperando você
cmd-terminal-rename = Terminal: renomear a aba
terminal-no-attention = Nenhuma aba de terminal pede atenção.
terminal-goto-placeholder = Ir para a aba de terminal…
terminal-rename-placeholder = Nome da aba
terminal-rename-reset = Voltar ao nome padrão
terminal-rename-to = Renomear para
terminal-close-running-title = Fechar a aba?
terminal-close-running = { $name } ainda está rodando. Fechar a aba encerra o processo.
terminal-close-running-yes = Fechar e encerrar
terminal-close-running-no = Cancelar
cmd-terminal-find = Terminal: buscar
terminal-find-placeholder = Buscar no terminal (Enter: mais antiga)
terminal-find-older = Mais antiga (acima)
terminal-find-newer = Mais nova (abaixo)
terminal-find-none = Nada encontrado
terminal-find-close = Fechar a busca
cmd-terminal-split = Terminal: dividir (ou juntar)
cmd-terminal-resume = Terminal: retomar agentes da sessão anterior
terminal-resume-nothing = Nenhum agente da sessão anterior esperando.
terminal-resume-title = Retomar os agentes?
terminal-resume-message = { $count ->
    [one] Um agente estava aberto quando a IDE fechou: { $names }.
   *[other] { $count } agentes estavam abertos quando a IDE fechou: { $names }.
}
terminal-resume-all = Retomar todos
terminal-resume-pick = Escolher…
terminal-resume-none = Não
terminal-resume-pick-placeholder = Agente para retomar…
terminal-resume-same = mesma conversa
terminal-resume-new = conversa nova
terminal-resume-failed = Não deu para retomar a conversa de { $title } (código { $code }). Abra uma conversa nova em "Agentes: iniciar…" (Ctrl+Shift+A).
terminal-previous-session = — sessão anterior, { $when } —
agent-request-ignored = Campos que esta versão não conhece (ignorados): { $fields }
