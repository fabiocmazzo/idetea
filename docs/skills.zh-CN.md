# 智能体技能

[Português](skills.md) · [Français](skills.fr.md) · [Deutsch](skills.de.md) · [Italiano](skills.it.md) · [日本語](skills.ja.md) · **中文（简体）**

一项技能是一个文件夹，里面有一份`SKILL.md`（带`name`和`description`的头，加上说明）和配套文件：参考、模板，有时还有脚本。智能体读每一项的描述，当它用得上当前这件事时，再打开整项技能。研究与决策见[RFC 0005](rfc/0005-agentes-e-projetos.md)第2阶段（原文为葡萄牙语）。

`idetea skills`把一个git仓库里的技能安装到当前文件夹的项目里：

```sh
idetea skills                               # 这个项目里已安装的
idetea skills list                          # anthropics/skills的（默认来源）
idetea skills list dono/repo                # 另一个来源：GitHub的dono/repo、一个git URL，或一个文件夹
idetea skills add anthropics/skills pdf frontend-design
idetea skills update                        # 来源上变了什么，带差异；此时什么都还不改
idetea skills update --apply                # 安装变了的部分
idetea skills remove pdf
```

- **放在哪里：**一份副本在`.agents/skills/<名称>/`（Codex、OpenCode和其他），另一份在`.claude/skills/<名称>/`（Claude Code）。两份都进git，连同`.idetea/skills.toml`，里面记下每一项来自哪个来源、哪个提交。
- **固定在一次提交上：**技能保持安装时的样子，直到一次`update --apply`。不带`--apply`的`update`只显示差异，让你先读改了什么。
- **什么都不运行：**IDE只复制文件。来源放在缓存里（`~/.cache/idetea/skills/`，或`$XDG_CACHE_HOME/idetea/skills/`），并从git历史读取。一项技能里的符号链接和子模块不会被复制。列表里，“有脚本”标出那些带了智能体可以运行的文件的技能：先读再装。
- **手工改过的副本：**`add`、`update --apply`和`remove`会拒绝；`--force`仍然替换（或删除）。一个同名文件夹如果不是IDE装进来的，也只有`--force`才会替换。

退出码：0（完成），1（出错，或有更新还没应用），2（用法错误）。

## 新建项目

“项目: 新建…”（命令面板）在界面里显示一张表单：创建到哪里、文件夹名、模板、一句关于项目的话、是否创建git，以及打开哪个智能体。从命令行：

```sh
idetea new meu-app --modelo rust --descricao "一个单位换算器" --agente claude
```

IDE创建文件夹、git、模板的技能、带**初始请求**和面板说明的`AGENTS.md`，以及一个导入`AGENTS.md`的`CLAUDE.md`。真正把项目做出来的是智能体。

**智能体在IDE的一个终端里。**表单列出`PATH`上的智能体（`claude`、`codex`、`gemini`、`opencode`；第一个找到的默认选中）以及“无（我自己打开）”。选了一个智能体时，IDE在新项目里写下一份[终端请求](terminais-de-agente.md)（暂时为葡萄牙语），文件是`.idetea/agents/agente.json`，命令是`<agente> 'Start with the initial request in AGENTS.md.'`。新窗口把这份请求显示在“终端请求”选项卡里，和其他请求一样：只有在“允许并运行”之后，智能体才会打开。选“无（我自己打开）”时，你自己到新文件夹里打开它，并让它开始。

| 模板 | 技能 |
|---|---|
| `vazio` | `novo-projeto`（智能体询问这个项目是什么） |
| `rust` | `novo-projeto`、`novo-projeto-rust` |
| `site` | `novo-projeto`、`novo-projeto-site`、`frontend-design`（来自`anthropics/skills`） |
| `python` | `novo-projeto`、`novo-projeto-python` |
| `fastapi` | `novo-projeto`、`novo-projeto-python`、`novo-projeto-fastapi` |

技能在本仓库的[`skills/`](../skills/)里，模板在[`skills/modelos.toml`](../skills/modelos.toml)里：一个新模板只是那里的一条记录。
