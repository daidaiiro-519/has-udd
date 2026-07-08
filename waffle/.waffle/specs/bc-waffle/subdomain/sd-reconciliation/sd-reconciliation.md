# sd-reconciliation

---

## 概要

スペック自身が嘘をつかないよう、3種類のドリフト（スペック内部の参照整合性・スペックとテストシナリオの対応関係・Document集約とSchema版の対応関係）を機械的に検知し続ける業務領域。spec 同士（bc/subdomain/usecase の宣言と実ファイル）の参照整合性、spec の TestScenarios と対応するテストコードのシナリオ名の整合、Document が参照する Schema の版が実在し最新であるかという、複数の異なるデータ領域（Spec・ソースコード・Schema）をまたいで一貫性を保つ。

---

## カテゴリー

- **カテゴリー**: core
- **根拠**: 「陳腐化しない仕様」は waffle の差別化原理そのもの。spec 同士・spec とテストコードという独立して変化しうる2つの世界を突き合わせ続ける能力は、既製の静的解析ツールでは代替できない waffle 独自の関心事。

---

## 所属ユースケース

- uc-check-spec-integrity
- uc-check-scenario-drift
- uc-check-schema-version-drift

---

## 実装ガイド

中核ゆえドメインモデルで厚く実装する。宣言と実態の突き合わせ、シナリオ名とテスト関数名の突き合わせは自前の決定的コードで書き、AIに判定を委ねない。
