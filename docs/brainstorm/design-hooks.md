# Hooks 設計ブレスト（Phase 5）

## 目的

has-udd の変更ループ・検証ゲートを **機械強制**する Hooks を設計する。
保守ループ（`design-maintenance-loop.md`）の機構面を担う。Hooks が無いと「陳腐化しない・規約を守る」は努力目標に劣化する。

関連: `design-maintenance-loop.md`（保守ループ）/ `design-engine-set.md` ES-3（reconcile engine 帰属）/ `project-coding-schema.md`（検証ゲート・逆引き）。

---

## 論点一覧

| # | 論点 | 状態 |
|---|---|---|
| H-1 | Hooks の発火点（pre-commit / pre-merge / post-spec-change / post-stack-change / CI 定期）と各で何を実行するか | 未 |
| H-2 | 検証ゲート: validate（依存方向/命名/ban/アンカー/テスト緑）を Hooks で BLOCK | 未 |
| H-3 | ドリフト検知（ループC reconcile・@spec/@stack 走査→Spec 突合→未実装/orphan surface）の Hooks 化と発火頻度 | 未 |
| H-4 | スタック変更（ループB）時に全コード再 validate→違反列挙をトリガする Hook | 未 |
| H-5 | Spec 変更（ループA）時に影響コードを @spec 逆引き→赤テスト誘導する Hook | 未 |
| H-6 | サンプル鮮度・drift（ML-2/ML-3）: 「サンプルが自分の検証ルールを通る」CI Hook | 未 |
| H-7 | マルチツール互換: Claude Code / Kiro 等で hook 機構が異なる。共通抽象をどう持つか（Phase 6 と連動） | 未 |

---

## AI の初期見解（たたき台）

- **H-1**: 最低限 `pre-commit`（自分の変更分を validate）＋ `pre-merge/CI`（全体 reconcile）の2段。重い reconcile は毎コミットでなく merge/CI に寄せる
- **H-2**: 検証ゲートが Hooks の主目的。違反 severity=error は BLOCK・warn は通すが記録
- **H-3/H-5**: ループ A/C の技術核は @spec 逆引き（ripgrep→schema JSON・PoC 済）＋reconcile。Hook はこれを発火するだけ。engine 帰属は ES-3
- **H-4/H-6**: スタック変更とサンプル鮮度はセット。スタック変更 Hook で「サンプル更新＋全コード再 validate」を促す
- **H-7**: has-udd は Hook の**意図**（いつ何を検証）を schema/engine 側に持ち、ツール固有の hook 設定はそこから render/deploy する（render-engine の deploy と同思想）。Phase 6 互換と直結

---

## ユーザー見解

---

## 合意事項

（論点解決後に記録）

---

## 次のアクション

Phase 5 で本格化。Phase 4（HarnessAgent）・Phase 6（マルチツール互換）と連動。
それまで論点を保持。reconcile の engine 帰属は design-engine-set.md ES-3 で先に決まる可能性あり。
