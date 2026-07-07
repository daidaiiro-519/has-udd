# advisorエコシステム ロードマップ

このドキュメントは、`docs/brainstorm/brainstorm-platform-engineering-application.md`（AI時代のDDD×プラットフォームエンジニアリングのブレスト）を踏まえた、advisorエコシステム（ddd-advisor/tech-lead-advisor/platform-advisor/ux-advisor）構築の実行計画。

**現在地（2026-07-08）**：ddd-advisor（19 knowledge・OSS配布可能な独自解釈へ刷新済み）とtech-lead-advisor（10 knowledge・新設）が、KnowledgeSchema/v1・SkillSchema/v1に正式準拠する形で完成し、コミット済み（`b45c8f5`）。効果測定（動く最小サンプル）は未実施。

---

## Phase 1：効果測定（今ここ）

tech-lead-advisorの動く最小サンプル検証。CodingSchema刷新時（[[project-coding-schema-stage-k]] MS-1〜5）に実証済みの手法を流用する。

- **手法**：複数の独立した実装エージェントを、互いに隔離された環境（worktree等）で起動し、has-udd/Waffleの内部ソース・ドキュメントには一切アクセスさせない。**各エージェントには`ddd-advisor`と`tech-lead-advisor`の両方をSkillとして使える状態で渡す**（tech-lead-advisorだけに限定すると、そのStep1が前提とするサブドメイン分類の入力元が無くなり、DDD的判断を経ずに「安全側（中核相当）」へ機械的に倒れてしまうため。実際のAIオーサーの利用シーン——DDD判断はddd-advisorへ・配置判断はtech-lead-advisorへ、という使い分け——をそのまま再現する）。同じお題（新規実装タスク。DDD判断とアーキテクチャ配置判断の両方が必要になる規模のものを選ぶ）を独立に解かせる。
- **検証すること**：独立したエージェントどうしが同じ正しい構造に収束するか。収束すれば、知識が「読むだけで正しい判断に導ける」ことの実証になる。収束しなければ、その分岐点が知識の欠落・曖昧さとして特定できる（tech-lead-advisor側の欠落か、ddd-advisor側の欠落か、両者の連携部分の欠落かも切り分ける）。
- **見つかった欠落があれば知識を補強し、再検証する。**
- 規模：中（複数エージェントの起動・比較が必要）

## Phase 2：ブレスト論点の残り2つを解消

- **論点4**：Thinnest Viable Platform原則をWaffle自身の設計原則として明文化する（言語化のみ・コストほぼゼロ）
- **論点9**：Stage B Conformance Scorecardの担当割り振りを確定する（tech-lead-advisorが適合基準を運用ルールとして持ち、既存3スクリプトは薄い実行体にする、という方向性は既に出ている）
- 規模：小（ユーザー見解を伺うだけ）

## Phase 3：advisorエコシステムの拡張

- `platform-advisor`（SRE・セキュリティ・クラウドアーキテクチャ）
- `ux-advisor`（画面設計・体験設計・PresentationSpecSchema）
- 今回確立した型（KnowledgeSchema準拠・原典知識/運用ルール分離・テキストベース疎結合インターフェース）をそのまま複製できるため、tech-lead-advisorより速く着手できる見込み。
- 規模：中〜大（advisor2つ分）
- **依存**：Phase 1の効果測定で型に欠陥が見つかった場合、先に直してから複製すべき

## Phase 4：Stage B実装（Conformance Scorecard本体）

- 3本のドリフト検知スクリプト（参照整合性・シナリオドリフト・スキーマ版ドリフト）の結果形式を統一し、render機構でspec×checkのマトリクスとして可視化する。
- 適合基準の実体はtech-lead-advisorの運用ルールとして持たせる（論点9の方向性を実装に落とす）。
- 規模：中
- **依存**：Phase 2（論点9の確定）

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
