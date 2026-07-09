# has-udd リポジトリ

## loomdb/（独立プロダクト・GitHub上に分離済み）

`loomdb/` はゲートウェイ向け組込 NoSQL「LoomDB」の自己完結ディレクトリ。
**作業する場合は必ず `loomdb/CLAUDE.md` を先に読むこと**（確定済みの意思決定・
ユーザー要望・TDD 必須の開発プロセスが記録されている）。

`git subtree split --prefix=loomdb` で https://github.com/daidaiiro-519/loomdb
（public）へ履歴ごと切り出し済み。has-udd側の `loomdb/` は通常のサブディレクトリとして
そのまま残っており、これまで通り直接編集してよい（submodule化はしていない）。
分離リポジトリ側への反映は、必要になった時点で改めて subtree split を実行して同期する。

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

<!-- BACKLOG.MD GUIDELINES START -->
<CRITICAL_INSTRUCTION>

## Backlog.md Workflow

This project uses Backlog.md for task and project management.

**For every user request in this project, run `backlog instructions overview` before answering or taking action.**

Use the overview to decide whether to search, read, create, or update Backlog tasks.

Use the detailed guides when needed:
- `backlog instructions task-creation` for creating or splitting tasks
- `backlog instructions task-execution` for planning and implementation workflow
- `backlog instructions task-finalization` for completion and handoff

Use `backlog <command> --help` before running unfamiliar commands. Help shows options, fields, and examples.

Do not edit Backlog task, draft, document, decision, or milestone markdown files directly. Use the `backlog` CLI so metadata, relationships, and history stay consistent.

</CRITICAL_INSTRUCTION>
<!-- BACKLOG.MD GUIDELINES END -->
