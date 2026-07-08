# advisorエコシステム ロードマップ

このドキュメントは、`docs/brainstorm/brainstorm-platform-engineering-application.md`（AI時代のDDD×プラットフォームエンジニアリングのブレスト）を踏まえた、advisorエコシステム（ddd-advisor/tech-lead-advisor/platform-advisor/ux-advisor）構築の実行計画。

**現在地（2026-07-09）**：ddd-advisor（19 knowledge・OSS配布可能な独自解釈へ刷新済み）とtech-lead-advisor（11 knowledge）が、KnowledgeSchema/v1・SkillSchema/v1に正式準拠する形で完成し、コミット済み。**Phase 1（効果測定）・Phase 2（ブレスト全11論点）とも完了**。次はPhase 3（advisorエコシステムの拡張）から着手する。

---

## Phase 1：効果測定 ✅完了

tech-lead-advisorの動く最小サンプル検証。CodingSchema刷新時（[[project-coding-schema-stage-k]] MS-1〜5）に実証済みの手法を流用した。

- **手法**：複数の独立した実装エージェントを、互いに隔離された環境（worktree等）で起動し、has-udd/Waffleの内部ソース・ドキュメントには一切アクセスさせない。**各エージェントには`ddd-advisor`と`tech-lead-advisor`の両方をSkillとして使える状態で渡す**（tech-lead-advisorだけに限定すると、そのStep1が前提とするサブドメイン分類の入力元が無くなり、DDD的判断を経ずに「安全側（中核相当）」へ機械的に倒れてしまうため。実際のAIオーサーの利用シーン——DDD判断はddd-advisorへ・配置判断はtech-lead-advisorへ、という使い分け——をそのまま再現する）。同じお題（新規実装タスク。DDD判断とアーキテクチャ配置判断の両方が必要になる規模のものを選ぶ）を独立に解かせる。
- **検証結果**：ラウンド1（設計判断のみ）は2エージェントとも「中核」・ドメイン層への不変条件配置・DTO越境設計で完全収束。ラウンド2（実装版）は2エージェントとも「補完」に判定を修正（実装に踏み込んで初めて気づいた精緻化）した点まで含めて再収束。
- **見つかった知識欠落**：不変条件のカプセル化について、メソッドを用意するだけでなく状態フィールドへの直接代入経路を言語機能（private化・読み取り専用プロパティ等）で塞ぐ必要がある、という点でエージェント間に実装徹底度の差があった。`architecture-dependency-direction`のPrinciples/AntiPatternsに追記し補強済み。

## Phase 2：ブレスト全11論点を解消 ✅完了

`docs/brainstorm/brainstorm-platform-engineering-application.md`の論点1〜11、全て合意決定まで到達し正式クローズ済み。主な決定:

- 論点4：Thinnest Viable Platform原則を`tech-lead-advisor`のknowledge（`architecture-evidence-based-scope`）として採用
- 論点5：advisorエコシステムの二層構造・命名・担当領域（ddd-advisor/tech-lead-advisor/platform-advisor/ux-advisor）を確定
- 論点8：「backbone/運用ルール分離」の構造パターンは全advisor共通だが、「単一書籍を厳密抽出する」重いプロセスはDDD固有（根幹だから）の特別対応であり、他advisorは公開知見の総合で足りるとした
- 論点9：Conformance Scorecardの適合基準は`tech-lead-advisor`のknowledgeを出典とし、判定ロジックは決定的スクリプト側に実装する

## Phase 3：advisorエコシステムの拡張（次はここ）

- `platform-advisor`（SRE・セキュリティ・クラウドアーキテクチャ）
- `ux-advisor`（画面設計・体験設計・PresentationSpecSchema）
- 今回確立した型（KnowledgeSchema準拠・原典知識/運用ルール分離・テキストベース疎結合インターフェース）をそのまま複製できるため、tech-lead-advisorより速く着手できる見込み。
- 規模：中〜大（advisor2つ分）
- **依存**：Phase 1の効果測定で型に欠陥が見つかった場合、先に直してから複製すべき

## Phase 4：Stage B実装（Conformance Scorecard本体）

- 3本のドリフト検知スクリプト（参照整合性・シナリオドリフト・スキーマ版ドリフト）の結果形式を統一し、render機構でspec×checkのマトリクスとして可視化する。
- 適合基準の実体はtech-lead-advisorの運用ルールとして持たせる（論点9の方向性を実装に落とす）。
- 規模：中
- **依存**：Phase 2（論点9の確定）✅解消済み・着手可能

## Phase 5：Stage C実装（OKF/カタログ）

- Waffleの`DomainSpecSchema`が持つ`specKind`階層（bounded-context→subdomain→aggregate→usecase）＋コンテキスト間連係パターン（`context-integration.md`）を核とした、最小限の可視化ビュー。
- 「仕様が人間にとって発見・可読可能であることは、AI時代に人間が仕様へ注力するというWaffleの根本前提そのものである」（論点3）という位置づけで、優先度を引き上げ済み。
- 規模：大（新規UI/可視化層が必要・`design-engine-render-okf`等の既存ブレストの続きから着手）
- 他のPhaseと並行しても支障のない独立ワークストリーム。

## バックログ（優先度未定・急ぎではない）

- ddd-advisorのKnowledgeSchemaへのレトロフィット（現在は手書きmarkdownのまま。tech-lead-advisorと構成が異なる状態が続いている）
- Waffle版DevEx指標の設計（「scaffoldからvalidate緑になるまでの往復回数」等、論点1のアクション項目）

---

## 参照

- `docs/brainstorm/brainstorm-platform-engineering-application.md`（本ロードマップの根拠となる全論点・実施記録）
- `docs/planning/roadmap.md`（has-udd/Waffle全体のロードマップ。本ドキュメントはStage C周辺の詳細計画に位置づく）
