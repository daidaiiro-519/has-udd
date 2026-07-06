# sd-reconciliation

---

## 概要

スペック自身が嘘をつかないよう、2種類のドリフト（スペック内部の参照整合性・スペックとテストシナリオの対応関係）を機械的に検知し続ける業務領域。spec 同士（bc/subdomain/usecase の宣言と実ファイル）の参照整合性、および spec の TestScenarios と対応するテストコードのシナリオ名の整合という、2つの異なるデータ領域（Spec と ソースコード）をまたいで一貫性を保つ。

---

## カテゴリー

- **カテゴリー**: core
- **根拠**: 「陳腐化しない仕様」は waffle の差別化原理そのもの。spec 同士・spec とテストコードという独立して変化しうる2つの世界を突き合わせ続ける能力は、既製の静的解析ツールでは代替できない waffle 独自の関心事。

---

## 所属ユースケース

---

## 実装ガイド

中核ゆえドメインモデルで厚く実装する。中身となる2つのusecase（spec内参照整合性検査・spec↔テストシナリオドリフト検査）は設計中（scripts/check_spec_referential_integrity.py・scripts/check_scenario_drift.py が参照実装。正式なusecase化にあたっては既存のhexagonalアーキテクチャ（port/adapter）に則って再設計する）。
