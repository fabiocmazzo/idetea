## Erros do documento (aparecem no painel, para o usuário e para o agente corrigir)
ui-too-big = O documento tem { $size } bytes; o máximo é { $max }.
ui-not-json = O arquivo não é um JSON válido: { $error }
ui-not-object = O documento precisa ser um objeto JSON.
ui-wrong-format = "format" precisa ser "idetea-ui" (veio { $found }).
ui-unsupported-version = Versão { $version } do formato não suportada (esta IDE entende a { $supported }).
ui-missing = Falta o campo "{ $field }".
ui-wrong-type = "{ $field }" precisa ser do tipo { $expected }.
ui-too-many = "{ $field }" tem mais de { $max } itens.
ui-too-long = "{ $field }" passa de { $max } bytes.
ui-bad-id = "{ $field }": id inválido ("{ $id }"). Use letras, números, "_" e "-" (até 64).
ui-duplicate-id = Id repetido: "{ $id }".
ui-bad-path = "{ $field }": "{ $path }" não é um caminho válido (ex.: "/nome").
ui-bad-file-path = "{ $field }": "{ $path }" tem que ser um caminho relativo à raiz do projeto, sem "..".
ui-data-too-big = "data" passa de { $max } bytes.
ui-data-too-deep = "data" tem mais de { $max } níveis.

## Avisos (o documento abre mesmo assim)
ui-unknown-field = Campo desconhecido ignorado: "{ $field }".
ui-unknown-value = "{ $field }": valor desconhecido "{ $value }", usando o padrão.
ui-unknown-component = Componente "{ $component }" não suportado nesta versão.
ui-link-error = Não deu para abrir { $url }: { $error }
ui-link-ignored = Link ignorado (só web, arquivos do projeto e idetea:config): { $url }
ui-comment-label = Comentário para o agente (opcional)
ui-comment-placeholder = vai junto com o botão que você clicar
ui-copy = Copiar
ui-copied = Copiado.
ui-steps-missing = { $count ->
    [one] Falta marcar 1 passo.
   *[other] Faltam marcar { $count } passos.
}
ui-step-optional = (opcional)
ui-requires-unbound = "{ $field }": "{ $path }" não é o "value" de nenhum componente; ignorado.
ui-fields-missing = Falta preencher: { $fields }
ui-too-many-images = O documento tem mais de { $max } imagens.
ui-image-loading = { $path }: carregando…
ui-image-error = não deu para ler: { $error }
ui-image-not-file = não é um arquivo (symlinks não são seguidos)
ui-image-too-big = passa de { $max } MB
ui-image-too-large = { $width }×{ $height } px; o máximo é { $max } de lado
ui-image-total = as imagens do documento passam de { $max } milhões de pixels
ui-too-many-sounds = O documento tem mais de { $max } sons.
ui-audio-play = Tocar
ui-audio-stop = Parar
ui-audio-info = { $seconds } s · { $rate } Hz · { $channels ->
        [one] mono
        [2] estéreo
       *[other] { $channels } canais
    } · { $bits } bits · { $peak }
ui-audio-peak = pico { $db } dBFS
ui-audio-silent = silêncio
ui-audio-loop = em repetição
ui-audio-clips = Chega a 0 dBFS: pode estar estourando.
ui-audio-extension = Não é um som (aceitos: { $list }).
ui-audio-too-big = Maior que { $max } MB: toca, sem duração nem forma de onda.
ui-audio-unreadable = Sem duração nem forma de onda ({ $error }); toca mesmo assim.
ui-audio-no-player = Nenhum tocador de som encontrado (pw-play, paplay, aplay ou afplay).
ui-audio-plugin = Os painéis de plugin não tocam som.
ui-sprite-pause = Pausar
ui-sprite-play = Animar
ui-sprite-step = Próximo quadro
ui-sprite-info = quadro { $frame } de { $count } · { $fps } fps
ui-sprite-frame = { $field }: frame_width e frame_height vão juntos.
ui-sprite-too-small = A folha ({ $width }×{ $height }) é menor que um quadro.
ui-open-blocked = { $path }: o link idetea:open só abre { $list }.
ui-open-not-file = { $path } não é um arquivo comum (falta, é pasta ou symlink).
