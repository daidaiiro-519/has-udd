# Waffle

> スキーマという型で文書を焼き上げる、**構造検証＋意味ガイダンス内蔵**のドキュメントエンジン。

**waffle＝ワッフル。** ワッフルアイロンの格子に生地を流し込んで焼き上げるように、schemaという
型に沿って構造化された`document.json`を検証・整形するこのエンジンを、そのまま名前にしている。

## これは何か

- JSON Schemaに適合する構造化ドキュメント（`document.json`）を、**検証・クエリ・レンダリング・
  雛形生成**する軽量エンジン。
- documentは`blockKey`で住所付けされたコンテンツブロックの集合として構成され、各ブロックの
  `blockType`はschema側の`x-prompt-query`／`x-prompt-write`という**静的に埋め込まれた
  LLM向けガイダンス**を持つ。AIはファイルを直接解釈せず、Waffleが返す`{ prompt, value }`を
  手がかりに動く。
- 対象・呼び出し元を一切限定しない。CLIとMCPの2つのinbound adapterを持ち、**テキストベースの
  入出力**だけで完結する（内部実装への結合を作らない）。

## Waffleの差別化ポイント

| | 一般的なJSON Schemaバリデータ | Waffle |
|---|---|---|
| 構造検証 | あり | あり |
| ブロック単位のクエリ・射影 | 無い | **あり**（`blockKey`・配列フィルタ・再帰検索など16操作） |
| 意味ガイダンスの埋め込み | 無い | **あり**（`x-prompt-query`/`x-prompt-write`をschemaに静的定義） |
| Markdown/HTMLレンダリング | 無い | **あり**（`x-render`テンプレートからJinja2で生成） |
| 雛形生成（scaffold） | 無い | **あり**（discriminatorからskeleton＋fillTemplateを生成） |
| ドキュメントのライフサイクル管理 | 無い | **あり**（`status`: CREATED→VALIDATED→RENDERED→SUPERSEDED） |

構造検証だけでなく、「このブロックはどう解釈・執筆されるべきか」という**意味の契約**をschema
自体に持たせている点が、素のJSON Schemaバリデータとの違い。

## 設計の要点

- **アーキテクチャ**: ポートとアダプター（ヘキサゴナル）。`domain`（documentモデル・schema）・
  `application`（validate/query/render/scaffoldのusecase）・`adapters`（CLI/MCPのinbound、
  ファイルシステム/JSON Schemaのoutbound）。
- **schemaはWaffle自身の資産**: `SkillSchema`/`SpecSchema`/`CodingSchema`/`RenderMetaSchema`を
  パッケージ内（`src/waffle/domain/model/`）に同梱する。外部プロジェクトが差し替える口は
  現状なく、これらのschema語彙自体がWaffleの提供する価値の一部。
- **依存は厳選**: `jsonschema`（構造検証）・`typer`（CLI）・`fastmcp`（MCP）のみ。

## ワークスペース構成

```
waffle/
├─ pyproject.toml
├─ src/waffle/
│   ├─ domain/          # documentモデル・schema（SkillSchema/SpecSchema/CodingSchema/RenderMetaSchema）
│   ├─ application/     # usecases: validate_engine / query_engine / render_engine / scaffold_engine
│   ├─ adapters/
│   │   ├─ inbound/     # CLI（typer）・MCP（fastmcp）
│   │   └─ outbound/    # ファイルシステム・JSON Schema validator・schema resolver
│   └─ shared/          # Result型・タグ定数
├─ tests/               # pytest（単体）
├─ features/            # behave（受け入れシナリオ・Waffle自身のspec/skill documentで検証）
└─ .has-udd/documents/  # Waffle自身を説明するspec/skill document（自己完結テスト用）
```

## 動かす

```bash
cd waffle
uv sync
uv run waffle validate --path .has-udd/documents/skills/harness-query-engine.json
uv run waffle query --operation get_block --path .has-udd/documents/skills/harness-query-engine.json --blockKey interface
uv run waffle render --path .has-udd/documents/skills/harness-query-engine.json --no-deploy
```

他プロジェクトのルートから使う場合（has-uddでの実例）:

```bash
uv run --project waffle waffle validate --path .has-udd/documents/skills/harness-query-engine.json
```

## ステータス

TDDで実装中。

| 操作 | 状態 |
|---|---|
| `validate`（JSON Schema構造検証・ConditionExpression等は対象外） | ✅ |
| `query`（16オペレーション: blockKey取得・配列フィルタ・再帰検索など） | ✅ |
| `render`（`x-render`テンプレートからMarkdown/HTML生成・deploy） | ✅ |
| `scaffold`（discriminatorからskeleton＋fillTemplate生成） | ✅ |
| CLI（typer）・MCP（fastmcp）の両inbound adapter | ✅ |
| pytest（単体）15件・behave（受け入れ）65シナリオ | ✅ green |
| PyPI公開（パッケージング・配布） | ⏳ 未着手 |
| 外部プロジェクトからのschema差し替え（現状はバンドル固定） | ⏳ 未着手（設計判断としてバンドル維持を選択済み） |

## ライセンス

MIT License。[LICENSE](LICENSE)を参照。

## 背景

`has-udd`（Harness Agentic Scrum Usecase-Driven-Development）というエージェントシステムの
document.json処理部分として生まれ、2026年にengine部分を独立OSS「Waffle」として切り出した。
経緯は`docs/brainstorm/brainstorm-has-udd-oss-separation.md`（has-uddリポジトリ側）を参照。
