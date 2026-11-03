# O manifesto da IDE TEA

<p><a href="MANIFESTO.md">English</a> · <b>Português</b> · <a href="MANIFESTO.es.md">Español</a></p>

O editor é onde você passa o dia. Ele deveria parecer uma sala silenciosa, não um shopping.

## 1. Rapidez é funcionalidade

Esperar quebra o foco. A IDE TEA abre num piscar de olhos, abre arquivos grandes sem travar e faz o trabalho lento
(busca, git, servidores de linguagem) longe da tela onde você digita. Quando algo fica lento, é bug.

## 2. Silêncio por padrão

Sem conta. Sem telemetria. Sem pop-ups sobre funcionalidades que você não pediu. A rede só é usada quando você
pede (push, pull, fetch). Nada muda debaixo dos seus pés: as configurações ficam num arquivo TOML simples que você
lê, versiona e compartilha.

## 3. Previsível antes de esperta

Todos os atalhos estão listados num lugar só (`F1`) e podem ser trocados. Um painel fica onde você o colocou. Fechar
algo com alterações não salvas sempre pergunta. Apagar um arquivo manda para a lixeira. Arquivos mudados fora da IDE
são percebidos, e o seu trabalho não salvo nunca é jogado fora para "ajudar".

## 4. Pequena por dentro, completa por fora

O núcleo só edita arquivos. A árvore, o terminal, o git, o depurador e todo o resto são extensões que você pode
desligar. Um núcleo pequeno é mais fácil de confiar, mais fácil de ler e mais difícil de quebrar.

## 5. Agentes são bem-vindos, e você continua no comando

Agentes de código fazem parte de como trabalhamos hoje. A IDE TEA dá a eles um jeito simples, aberto e feito de
arquivos de falar com você: perguntas como formulários de verdade, terminais que eles podem pedir, ajudantes no
próprio worktree. As regras são sempre as mesmas:

- **Você vê antes de rodar.** Todo comando que um agente pede aparece inteiro e espera a sua aprovação, a não ser
  que você o ponha numa lista no *seu* config. Um projeto nunca consegue se dar permissão.
- **A sua resposta é a resposta.** Aprovar é uma decisão, não um clique para se livrar de um diálogo. Os botões que
  importam esperam você fazer os passos.
- **Arquivos simples, sem amarras.** O protocolo é JSON numa pasta. Qualquer agente que lê e escreve arquivos pode
  usar, e você pode ler cada mensagem.

## 6. Gratuita de verdade

A IDE TEA é Apache 2.0 e gratuita, e vai continuar assim. Nada de plano pago escondendo as partes boas.

## 7. Feita para mentes diferentes

O nome tem um segundo sentido: em português, TEA é a sigla do Transtorno do Espectro Autista. A IDE TEA é feita por
um desenvolvedor autista, e isso aparece no que ela valoriza: pouco barulho, texto claro, coisas que se comportam
sempre do mesmo jeito, e espaço para se concentrar. Se a sua mente funciona assim também, seja bem-vindo. Se não,
provavelmente você vai gostar mesmo assim.
