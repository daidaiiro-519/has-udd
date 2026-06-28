# 保守ループ / 変更時シーケンス 設計ブレスト（エンドユーザの陳腐化対策）

## 目的

has-udd を導入したエンドユーザが **長期運用で陳腐化させずに保守できる**ための変更ループを設計する。
初回作成（happy path）ではなく、**Spec 変更・スタック進化・ドリフトという変更フロー**が対象。
出典: `docs/design/coding-schema-sequence.md`（図2: 変更フロー）で炙り出した未解決リスク ④⑤ ＋ ループ A/B/C。

この論点群は **Hooks（`design-hooks.md`）・engine 帰属（`design-engine-set.md` ES-3）・has-udd ツール本体**にまたがる。
ここを umbrella とし、機構の詳細は各シートへ振る。

---

## 背景（変更ループの3種・図2より）

- **ループA: Spec 変更**（What が変わる）→ @spec 逆引きで影響コード特定 → .feature 再 render（赤）→ 修正 → 緑
- **ループB: スタック/規約 変更**（library 更新・アーキ変更）→ サンプル更新 → 全コード再 validate → 移行
- **ループC: ドリフト検知**（随時/CI）→ ripgrep 走査 → Spec と reconcile → 未実装/orphan を surface

---

## 論点一覧

| # | 論点 | 状態 | 振り先 |
|---|---|---|---|
| ML-1 | スタック進化（ループB）の伝播保証。既存コードの再 validate→移行の規模・自動化粒度。大規模時に現実的か | 未 | Hooks / engine |
| ML-2 | **固定サンプルの鮮度（④）**。トークン効率で固定サンプルを手本にしたが規約進化で腐る。更新トリガと保証 | 未 | Hooks（ループB に同期） |
| ML-3 | **②規約 instance と ③サンプルの drift（⑤）**。サンプルが「自分の検証ルールを通る」ことを CI で保証する機構 | 未 | Hooks / CI |
| ML-4 | Spec 変更（ループA）の影響特定→赤テスト誘導の自動化度。どこまで機械・どこから AI | 未 | Hooks / engine |
| ML-5 | ドリフト検知（ループC reconcile）の発火（commit/merge/定期/CI）と責務の置き場 | 未 | Hooks / engine ES-3 |
| ML-6 | これらが UDD の「陳腐化しない」約束を運用で本当に守れるか（保守ループ全体の健全性） | 未 | umbrella |

---

## AI の初期見解（たたき台・各論点）

- **ML-2/ML-3**: サンプルと code-template doc は同一スタックの2表現。**「サンプルが自分の code-template の検証ルールを通る」ことを CI で常時保証**すれば drift も鮮度も同時に守れる（サンプル=実行可能な規約の単体テスト、と捉える）。スタック変更（ループB）時はサンプル更新を必須ステップにする
- **ML-1**: 全コード再 validate は reconcile/走査（PoC 済）で機械的に違反列挙までは可能。移行（修正）は AI。規模は「違反箇所だけ」に絞れるので現実的見込みだが、大規模一括移行の安全性は要検証
- **ML-4/ML-5**: ループA/C は **@spec 逆引き（ripgrep→schema JSON）＋ reconcile** が技術核。発火を Hooks が担う。engine 帰属は ES-3 で決める
- **ML-6**: 変更ループが Hooks で機械強制されて初めて「陳腐化しない」が運用で成立する。Hooks 無し（手動運用）だと約束は努力目標に劣化する

---

## ユーザー見解

---

## 合意事項

（論点解決後に記録）

---

## 次のアクション

ML 論点は Hooks（design-hooks.md）と engine 帰属（design-engine-set.md ES-3）の解決と連動。
Phase 5（Hooks / FeedbackReport）で本格化。それまで論点を保持。
