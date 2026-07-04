# has-udd リポジトリ

## loomdb/（独立プロダクト・別リポジトリ化予定）

`loomdb/` はゲートウェイ向け組込 NoSQL「LoomDB」の自己完結ディレクトリ。
**作業する場合は必ず `loomdb/CLAUDE.md` を先に読むこと**（確定済みの意思決定・
ユーザー要望・TDD 必須の開発プロセスが記録されている）。

## waffle/（独立プロダクト・別リポジトリ化予定）

`waffle/` はhas-uddのdocument.jsonスキーマ駆動エンジン「Waffle」の自己完結ディレクトリ
（`loomdb/`と同じ位置付け・`git subtree split --prefix=waffle`で将来切り出し可能）。
旧称`has_udd`パッケージから改名（経緯は`docs/brainstorm/brainstorm-has-udd-oss-separation.md`
論点5を参照）。**作業する場合は必ず `waffle/CLAUDE.md` を先に読むこと**。

has-udd自身のCLI/MCP呼び出しは、repo rootから`uv run --project waffle waffle <command>`
の形で行う（`.has-udd/documents/`・`.claude/skills/`はrepo root基準の相対パス）。
