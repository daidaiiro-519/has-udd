# has-udd リポジトリ

## loomdb/（独立プロダクト・別リポジトリ化予定）

`loomdb/` はゲートウェイ向け組込 NoSQL「LoomDB」の自己完結ディレクトリ。
**作業する場合は必ず `loomdb/CLAUDE.md` を先に読むこと**（確定済みの意思決定・
ユーザー要望・TDD 必須の開発プロセスが記録されている）。

## waffle/（独立プロダクト・GitHub上に分離済み）

`waffle/` はhas-uddのdocument.jsonスキーマ駆動エンジン「Waffle」の自己完結ディレクトリ
（`loomdb/`と同じ位置付け）。旧称`has_udd`パッケージから改名（経緯は
`docs/brainstorm/brainstorm-has-udd-oss-separation.md` 論点5を参照）。
**作業する場合は必ず `waffle/CLAUDE.md` を先に読むこと**。

`git subtree split --prefix=waffle` で https://github.com/daidaiiro-519/waffle
（public）へ履歴ごと切り出し済み。has-udd側の `waffle/` は通常のサブディレクトリとして
そのまま残っており、これまで通り直接編集してよい（submodule化はしていない）。
分離リポジトリ側への反映は、必要になった時点で改めて subtree split を実行して同期する。

has-udd自身のCLI/MCP呼び出しは、repo rootから`uv run --project waffle waffle <command>`
の形で行う（`.has-udd/documents/`・`.claude/skills/`はrepo root基準の相対パス）。
