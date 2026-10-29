//! Janela de ajuda: atalhos (gerados do mapa de teclas), guia curto e "Sobre".
//!
//! O guia cita comandos como `{file.save}`; na tela isso vira o atalho atual
//! do usuário (ou "paleta" se o comando não tiver atalho).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use iced::widget::{
    Id, button, column, container, rich_text, row, rule, scrollable, space, span, text, text_input,
};
use iced::{Element, Fill, Font, Theme};
use tea_core::icons::{Icon, icon_button};
use tea_core::keymap::Keymap;

use crate::palette::CommandInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Shortcuts,
    Guide,
    About,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Tab(Tab),
    Query(String),
    Close,
}

/// Dados que só o app conhece, para a aba "Sobre".
pub struct AboutInfo<'a> {
    pub root: &'a Path,
    pub global_config: Option<&'a Path>,
    pub project_config: PathBuf,
}

pub struct Help {
    tab: Tab,
    query: String,
}

pub fn input_id() -> Id {
    Id::new("help-search")
}

/// Atalhos do widget de edição: fixos, não passam pelo mapa de teclas.
const EDITOR_KEYS: &[(&str, &str)] = &[
    ("Ctrl+F", "Buscar no arquivo"),
    ("Ctrl+H", "Buscar e substituir"),
    ("Ctrl+G", "Ir para linha"),
    ("Ctrl+Z / Ctrl+Y", "Desfazer / refazer"),
    ("Ctrl+D", "Selecionar a próxima ocorrência (multi-cursor)"),
    ("Ctrl+K / Ctrl+J", "Recolher / expandir blocos"),
    ("Ctrl+.", "Ações rápidas do LSP (quando houver)"),
    ("Ctrl+Clique", "Ir para definição (LSP)"),
    ("Tab / Ctrl+Tab", "Inserir tabulação"),
];

/// Guia: (título, parágrafos). `{comando}` vira o atalho atual.
const GUIDE: &[(&str, &[&str])] = &[
    (
        "Primeiros passos",
        &[
            "Abra a IDE com `idetea` numa pasta (ela vira a raiz da árvore) ou com `idetea arquivo...` \
             (os arquivos abrem em abas; a raiz continua sendo a pasta atual).",
            "Todos os comandos estão na paleta: {palette.open}. Digite parte do nome; o atalho aparece ao lado.",
            "Esta ajuda abre com {help.open}.",
            "A barra de ícones na borda esquerda mostra e oculta os painéis com um clique (o aberto fica destacado). \
             Arraste a borda entre um painel e o editor para mudar o tamanho; duplo clique volta ao padrão.",
        ],
    ),
    (
        "Arquivos e abas",
        &[
            "Abrir pelo nome: {file.quick_open} (digite partes do nome; `main.rs:42` vai para a linha; \
             `>` troca para os comandos).",
            "Novo: {file.new} · Abrir: {file.open} · Salvar: {file.save} · Salvar como: {file.save_as} · \
             Fechar aba: {file.close}.",
            "Abrir e Salvar como usam a janela de arquivos do sistema. Para digitar o caminho: \
             {file.save_as_path}. Abrir outra pasta ({file.open_folder}) abre uma nova janela da IDE.",
            "Trocar de aba: {tab.next} e {tab.prev} (Ctrl+Tab insere tabulação no editor).",
            "Fechar uma aba ou sair com alterações pergunta antes: Salvar, Descartar ou Cancelar.",
            "Arquivo mudado por fora (terminal, outro editor): a aba recarrega sozinha se não tiver alterações; \
             se tiver, fica como está e aparece um aviso.",
            "Botão direito numa aba: fechar outras, à direita, salvas ou todas, copiar caminho, revelar na \
             árvore e abrir no gerenciador de arquivos. Clique do meio fecha a aba.",
        ],
    ),
    (
        "Árvore de arquivos",
        &[
            "Mostrar/ocultar: {filetree.toggle} · Revelar o arquivo atual: {filetree.reveal}.",
            "Botão direito num item: novo arquivo, nova pasta, renomear, apagar, copiar caminho e abrir no \
             gerenciador de arquivos. Apagar manda para a lixeira do sistema.",
            "Cores do git: amarelo = modificado, verde = novo, vermelho = conflito, apagado = ignorado. \
             Uma pasta com mudanças dentro mostra um ponto.",
        ],
    ),
    (
        "Painel inferior",
        &[
            "Busca no projeto ({search.focus}), Problemas ({problems.focus}), Terminal ({terminal.focus}) e Git \
             ({git.focus}) ficam em abas no painel de baixo. {view.toggle_bottom} mostra/esconde o painel.",
            "Próximo problema: {problems.next}.",
        ],
    ),
    (
        "Terminal",
        &[
            "Com o foco no terminal, Ctrl+letra, F-teclas e setas vão para o shell (Ctrl+C, Ctrl+W...). \
             Os atalhos da IDE com Ctrl+Shift continuam valendo, então a paleta sempre abre.",
            "Copiar e colar no terminal: Ctrl+Shift+C e Ctrl+Shift+V. Digitar `exit` fecha o painel.",
        ],
    ),
    (
        "Debug",
        &[
            "Breakpoint: clique no número da linha ou {debug.toggle_breakpoint}. Iniciar/continuar: {debug.continue}. \
             Em projetos Rust compila com cargo build e acha o executável sozinho; nos outros, use [debug] no \
             config (program, args, cwd, build) ou escolha o executável. Trocar o programa: {debug.choose_program}.",
            "Parado: {debug.next} próxima linha, {debug.step_in} entra na função, {debug.step_out} sai, \
             {debug.pause} pausa, {debug.stop} para. A seta amarela mostra onde o programa está. Usa o gdb 14+.",
            "Painel Debug ({debug.focus}): botões, pilha de chamadas (clique vai para o quadro e mostra as variáveis \
             dele), variáveis (clique abre structs e vetores) e a saída do programa e da compilação.",
        ],
    ),
    (
        "Git",
        &[
            "A branch aparece na barra de status. O painel Git ({git.focus}) mostra o que mudou: + e − colocam e \
             tiram do stage, clicar num arquivo mostra o diff, Enter na mensagem faz o commit.",
            "Botão direito num arquivo do Git: abrir, ver diff, stage, descartar alterações (pede confirmação) \
             e copiar caminho.",
            "Fetch ({git.fetch}), pull ({git.pull}) e push ({git.push}) ficam nos botões do topo do painel. \
             A senha não é pedida aqui: use o agente ssh (ssh-add) ou um credential helper. \
             Uma branch nova pergunta antes de ser publicada no remoto.",
            "O painel se atualiza ao salvar e quando o repositório muda (commit ou checkout feitos no terminal). \
             Push e pull: use o terminal.",
        ],
    ),
    (
        "Linguagens (LSP)",
        &[
            "Servidores de linguagem iniciam sozinhos ao abrir um arquivo: rust-analyzer, clangd, pyright, \
             typescript-language-server, gopls e lua-language-server (precisam estar instalados).",
            "Ir para definição: {lsp.definition} ou Ctrl+Clique · Documentação: {lsp.hover} ou mouse parado \
             sobre o código · Formatar: {lsp.format}.",
            "Se um servidor falhar, a barra de status mostra o motivo; \"LSP: reiniciar servidores\" tenta de novo.",
        ],
    ),
    (
        "Configuração",
        &[
            "A tela de configurações ({settings.open}) muda tema, fonte, extensões, atalhos e LSP, gravando no config \
             global ou no do projeto. As mudanças valem na hora.",
            "Há um config global e um por projeto (`.idetea/config.toml`); o do projeto vence. Para editar o arquivo \
             direto: \"Config: abrir config global\" ou \"...do projeto\". Ao salvar, tudo recarrega.",
            "Atalhos ficam em [keys]: \"ctrl+alt+t\" = \"terminal.focus\". Texto vazio desliga um atalho padrão. \
             Os ids dos comandos aparecem na aba Atalhos.",
        ],
    ),
];

impl Help {
    pub fn new(tab: Tab) -> Self {
        Self { tab, query: String::new() }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Tab(tab) => self.tab = tab,
            Msg::Query(q) => self.query = q,
            Msg::Close => {}
        }
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn view<'a>(&'a self, commands: &'a [CommandInfo], keymap: &'a Keymap, about: AboutInfo<'_>) -> Element<'a, Msg> {
        let tab_button = |label: &'static str, tab: Tab| {
            let style = if self.tab == tab { button::secondary } else { button::text };
            button(text(label).size(13)).on_press(Msg::Tab(tab)).style(style).padding([4, 12])
        };
        let header = row![
            text("Ajuda").size(18),
            space::horizontal().width(16),
            tab_button("Atalhos", Tab::Shortcuts),
            tab_button("Guia", Tab::Guide),
            tab_button("Sobre", Tab::About),
            space::horizontal(),
            icon_button(Icon::Close, "Fechar (Esc)", Some(Msg::Close)),
        ]
        .spacing(4)
        .align_y(iced::Center);

        let body: Element<'a, Msg> = match self.tab {
            Tab::Shortcuts => self.shortcuts(commands, keymap),
            Tab::Guide => scrollable(guide(keymap)).height(Fill).into(),
            Tab::About => scrollable(about_view(about)).height(Fill).into(),
        };

        crate::overlay::modal(column![header, rule::horizontal(1), body].spacing(10), 860.0, Msg::Close)
    }

    fn shortcuts<'a>(&'a self, commands: &'a [CommandInfo], keymap: &'a Keymap) -> Element<'a, Msg> {
        let search = text_input("Filtrar por nome, comando ou tecla…", &self.query)
            .id(input_id())
            .on_input(Msg::Query)
            .size(14)
            .padding([6, 10]);

        let q = self.query.trim().to_lowercase();
        let matches = |fields: &[&str]| q.is_empty() || fields.iter().any(|f| f.to_lowercase().contains(&q));

        // Agrupa pelo prefixo do título ("Arquivo: salvar" → "Arquivo").
        let mut groups: BTreeMap<&str, Vec<(&CommandInfo, String)>> = BTreeMap::new();
        let mut seen = std::collections::HashSet::new();
        for c in commands {
            if !seen.insert(c.id.as_str()) {
                continue;
            }
            let keys = keymap.chords_for(&c.id).iter().map(|k| k.to_string()).collect::<Vec<_>>().join("  ");
            if !matches(&[&c.title, &c.id, &keys]) {
                continue;
            }
            let group = c.title.split_once(':').map_or("Geral", |(g, _)| g.trim());
            groups.entry(group).or_default().push((c, keys));
        }

        let mut list = column![].spacing(2);
        for (group, mut items) in groups {
            items.sort_by(|a, b| a.0.title.cmp(&b.0.title));
            list = list.push(section_title(group));
            for (c, keys) in items {
                let title = c.title.split_once(':').map_or(c.title.as_str(), |(_, t)| t.trim());
                list = list.push(shortcut_row(title, Some(&c.id), keys));
            }
        }
        let editor: Vec<_> = EDITOR_KEYS.iter().filter(|(k, d)| matches(&[k, d])).collect();
        if !editor.is_empty() {
            list = list.push(section_title("Editor (fixos)"));
            for (keys, desc) in editor {
                list = list.push(shortcut_row(desc, None, keys.to_string()));
            }
        }
        column![search, scrollable(list.padding([0, 12])).height(Fill)].spacing(8).into()
    }
}

fn section_title(title: &str) -> Element<'_, Msg> {
    container(text(title.to_uppercase()).size(11).style(text::secondary)).padding([10, 0]).into()
}

fn shortcut_row<'a>(title: &'a str, id: Option<&'a str>, keys: String) -> Element<'a, Msg> {
    let keys: Element<'a, Msg> = if keys.is_empty() {
        text("—").size(12).style(text::secondary).into()
    } else {
        row(keys.split("  ").map(|k| key_cap(k.to_string()))).spacing(6).into()
    };
    row![
        text(title).size(13).width(Fill),
        text(id.unwrap_or_default()).size(11).font(Font::MONOSPACE).style(text::secondary),
        container(keys).align_right(170),
    ]
    .spacing(12)
    .align_y(iced::Center)
    .padding([3, 0])
    .into()
}

fn key_cap<'a>(key: String) -> Element<'a, Msg> {
    container(text(key).size(12).font(Font::MONOSPACE))
        .padding([1, 6])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.strong.color.into()),
                border: iced::Border { color: p.background.strongest.color, width: 1.0, radius: 4.0.into() },
                ..container::Style::default()
            }
        })
        .into()
}

/// Troca `{comando}` pelo atalho e `` `código` `` por fonte monoespaçada.
pub fn render_paragraph(paragraph: &str, keymap: &Keymap) -> Vec<(String, Chunk)> {
    let mut out = Vec::new();
    let mut rest = paragraph;
    while !rest.is_empty() {
        let next = rest.find(['{', '`']);
        let Some(start) = next else {
            out.push((rest.to_string(), Chunk::Text));
            break;
        };
        if start > 0 {
            out.push((rest[..start].to_string(), Chunk::Text));
        }
        let close = if rest[start..].starts_with('{') { '}' } else { '`' };
        let Some(len) = rest[start + 1..].find(close) else {
            out.push((rest[start..].to_string(), Chunk::Text));
            break;
        };
        let inner = &rest[start + 1..start + 1 + len];
        if close == '}' {
            match keymap.chord_for(inner) {
                Some(k) => out.push((k.to_string(), Chunk::Key)),
                None => out.push(("(paleta)".to_string(), Chunk::Code)),
            }
        } else {
            out.push((inner.to_string(), Chunk::Code));
        }
        rest = &rest[start + 1 + len + 1..];
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chunk {
    Text,
    Key,
    Code,
}

fn guide<'a>(keymap: &Keymap) -> Element<'a, Msg> {
    let mut col = column![].spacing(6).padding([0, 12]);
    for (title, paragraphs) in GUIDE {
        col = col.push(container(text(*title).size(15)).padding([10, 0]));
        for p in *paragraphs {
            let spans: Vec<iced::widget::text::Span<'a, ()>> = render_paragraph(p, keymap)
                .into_iter()
                .map(|(t, kind)| match kind {
                    Chunk::Text => span(t),
                    Chunk::Key => span(t).font(Font { weight: iced::font::Weight::Bold, ..Font::MONOSPACE }),
                    Chunk::Code => span(t).font(Font::MONOSPACE),
                })
                .collect();
            col = col.push(rich_text(spans).size(13));
        }
    }
    col.into()
}

fn about_view<'a>(about: AboutInfo<'_>) -> Element<'a, Msg> {
    let line = |label: &str, value: String| {
        row![
            text(label.to_string()).size(13).width(160).style(text::secondary),
            text(value).size(13).font(Font::MONOSPACE)
        ]
        .spacing(8)
    };
    let global = about.global_config.map_or("(HOME não definido)".to_string(), |p| p.display().to_string());
    column![
        text("IDE TEA").size(24),
        text("Just code. Add ice.").size(14).style(text::secondary),
        space::vertical().height(12),
        line("Versão", env!("CARGO_PKG_VERSION").to_string()),
        line("Licença", format!("{} (livre e gratuita, sempre)", env!("CARGO_PKG_LICENSE"))),
        line("Projeto", about.root.display().to_string()),
        line("Config global", global),
        line("Config do projeto", about.project_config.display().to_string()),
        space::vertical().height(12),
        text("Feita em Rust com iced, iced-code-editor, alacritty_terminal (via iced_term) e syntect.").size(13),
        text("Ícones: Codicons © Microsoft, licença CC BY 4.0.").size(13),
    ]
    .spacing(6)
    .padding([0, 12])
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_placeholders() {
        let (km, _) = Keymap::from_pairs([("ctrl+s", "file.save")]);
        let out = render_paragraph("Salvar: {file.save}, outro: {x.y} e `idetea`.", &km);
        assert_eq!(
            out,
            vec![
                ("Salvar: ".into(), Chunk::Text),
                ("Ctrl+S".into(), Chunk::Key),
                (", outro: ".into(), Chunk::Text),
                ("(paleta)".into(), Chunk::Code),
                (" e ".into(), Chunk::Text),
                ("idetea".into(), Chunk::Code),
                (".".into(), Chunk::Text),
            ]
        );
        assert_eq!(render_paragraph("sem fim {abc", &km).last().unwrap().0, "{abc");
    }

    #[test]
    fn guide_commands_exist() {
        // Todo `{comando}` do guia precisa existir (núcleo ou extensões embutidas).
        let known: Vec<&str> = crate::CORE_COMMANDS.iter().map(|(id, _)| *id).collect();
        let ext = [
            "filetree.toggle",
            "filetree.reveal",
            "search.focus",
            "problems.focus",
            "problems.next",
            "terminal.focus",
            "git.focus",
            "git.fetch",
            "git.pull",
            "git.push",
            "debug.focus",
        ];
        for (_, paragraphs) in GUIDE {
            for p in *paragraphs {
                let mut rest = *p;
                while let Some(i) = rest.find('{') {
                    let end = rest[i..].find('}').unwrap() + i;
                    let id = &rest[i + 1..end];
                    assert!(known.contains(&id) || ext.contains(&id), "comando desconhecido no guia: {id}");
                    rest = &rest[end + 1..];
                }
            }
        }
    }
}
