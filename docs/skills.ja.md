# エージェントのスキル

<p><a href="skills.md">Português</a> · <a href="skills.fr.md">Français</a> · <a href="skills.de.md">Deutsch</a> ·
<a href="skills.it.md">Italiano</a> · <b>日本語</b> · <a href="skills.zh-CN.md">中文（简体）</a></p>

スキルは、`SKILL.md` (`name` と `description` の見出しと、手順) と、補助ファイルを入れたフォルダーです。
参照、ひな型、ときにはスクリプトです。エージェントはすべての説明を読み、今の作業に役立つときにスキル全体を開きます。
検討と決定: [RFC 0005](rfc/0005-agentes-e-projetos.md)（ポルトガル語）、段階 2。

`idetea skills` は、git リポジトリのスキルを、現在のフォルダーのプロジェクトへインストールします。

```sh
idetea skills                               # このプロジェクトに入っているもの
idetea skills list                          # anthropics/skills のもの (既定のソース)
idetea skills list dono/repo                # 別のソース: GitHub の owner/repo、git の URL、またはフォルダー
idetea skills add anthropics/skills pdf frontend-design
idetea skills update                        # ソースで変わったもの。差分付き。まだ何も変わらない
idetea skills update --apply                # 変わったものをインストール
idetea skills remove pdf
```

- **場所:** `.agents/skills/<名前>/` に 1 つ (Codex、OpenCode など)、`.claude/skills/<名前>/` にもう 1 つ (Claude Code)。
  両方を git に入れます。`.idetea/skills.toml` も一緒で、各スキルのソースとコミットを記録します。
- **コミットに固定:** スキルは、`update --apply` まで、インストール時のままです。`--apply` なしの `update` は差分を表示し、
  何が変わったかを先に読めます。
- **何も実行しない:** IDE がするのはファイルのコピーだけです。ソースはキャッシュ (`~/.cache/idetea/skills/`、または
  `$XDG_CACHE_HOME/idetea/skills/`) にあり、git の履歴から読みます。スキルのシンボリック リンクとサブモジュールはコピーしません。
  一覧の「スクリプトあり」は、エージェントが実行しうるファイルを含むスキルです。使う前に読んでください。
- **手で変えたコピー:** `add`、`update --apply`、`remove` は拒否します。`--force` は、それでも置き換え (または削除) ます。
  IDE 由来でない同名フォルダーも、`--force` だけで置き換わります。

終了コード: 0 (完了)、1 (エラー、または未適用の更新)、2 (使い方が違う)。

## 新しいプロジェクト

「プロジェクト: 新規…」(コマンド パレット) は、インターフェースにフォームを出します。作成先、フォルダー名、テンプレート、
プロジェクトについての 1 文、git を作るか、どのエージェントを開くかです。コマンド ラインでは:

```sh
idetea new meu-app --modelo rust --descricao "um conversor de unidades" --agente claude
```

IDE はフォルダー、git、テンプレートのスキル、**最初の依頼** とパネルの手順が入った `AGENTS.md`、
`AGENTS.md` を取り込む `CLAUDE.md` を作ります。プロジェクトを作るのはエージェントです。

**IDE のターミナルの中のエージェント。** フォームは、`PATH` にあるエージェント (`claude`、`codex`、`gemini`、
`opencode`。最初に見つかったものが選択済み) と「なし」を出します。エージェントを選ぶと、IDE は新しいプロジェクトに
[ターミナル要求](terminais-de-agente.md)（現時点ではポルトガル語） `.idetea/agents/agente.json` を書き、コマンドは
`<エージェント> 'Start with the initial request in AGENTS.md.'` です。新しいウィンドウは、他の要求と同じく、
「ターミナル要求」タブに要求を出します。エージェントが開くのは「許可して実行」のあとだけです。「なし」のときは、
新しいフォルダーで開き、開始を頼んでください。

| テンプレート | スキル |
|---|---|
| `vazio` | `novo-projeto` (エージェントが、プロジェクトが何かを尋ねる) |
| `rust` | `novo-projeto`、`novo-projeto-rust` |
| `site` | `novo-projeto`、`novo-projeto-site`、`frontend-design` (`anthropics/skills` から) |
| `python` | `novo-projeto`、`novo-projeto-python` |
| `fastapi` | `novo-projeto`、`novo-projeto-python`、`novo-projeto-fastapi` |

スキルはこのリポジトリの [`skills/`](../skills/) にあり、テンプレートは
[`skills/modelos.toml`](../skills/modelos.toml) にあります。新しいテンプレートは、そこへの 1 項目です。
