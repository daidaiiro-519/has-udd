# docs/ ガイド

has-udd のドキュメント。種類ごとにフォルダを分けている。

| フォルダ | 用途 | 主な中身 |
|---|---|---|
| `brainstorm/` | 設計の議論（論点→見解→合意）。未確定〜確定の思考過程 | `design-*.md`（各 schema/engine の設計）、`brainstorm-*.md`（コンセプト） |
| `design/` | 確定した設計の図・成果物（議論ではなく結論の図示） | `coding-schema-sequence.md`（時系列シーケンス） |
| `planning/` | 実装の計画・スプリント・進行管理 | `implementation-plan.md`、`sprint-plan.md`、`spec-id-map.md` |
| `research/` | 外部技術の調査メモ | ライブラリ・ツール調査 |
| `reference/` | 参考資料（書籍など） | DDD 書籍 PDF |

## 読む順（実装に入る人向け）

1. `brainstorm/` … なぜこの設計かの背景（特に `design-spec-schema.md` / `design-coding-schema.md` / `design-engine-set.md`）
2. `planning/implementation-plan.md` … フェーズと依存関係
3. `planning/sprint-plan.md` … スプリント単位のバックログ（ここから着手）
4. `design/coding-schema-sequence.md` … Spec とコードの時系列関係

## 主要な確定事項の所在

- スキーマ実体: `src/has_udd/domain/model/`（`SkillSchema/v1.json`, `CodingSchema/v2.json`, `SpecSchema/v2.json`）
- engine の Skill 文書: `.has-udd/skills/`
- 横断的な確定事項の要約: ユーザーのメモリ（`MEMORY.md`）
