# engine / コード → 将来 spec id 対応表（移行レディ ⑥）

bootstrap（手作り）で書くコードに先置きする `@spec:` placeholder と、
後で起こす UsecaseSpec の id を対応づける。dogfood（Sprint 4）で retroactive に Spec 化する際の地図。

| コード成果物 | @spec placeholder | 将来の Spec（usecase） | Skill 文書 | 状態 |
|---|---|---|---|---|
| render engine | `@spec:uc-render-document` | document.json → 成果物を描画する | `.has-udd/skills/harness-render-engine.json` | 未着手 |
| validate engine | `@spec:uc-validate-document` | document.json を schema 検証する | （要 Skill 文書） | 未着手 |
| scaffold engine | `@spec:uc-scaffold-document` | schema から空 document を生成し充填する | `.has-udd/skills/harness-scaffold-engine.json` | 未着手 |
| query engine | `@spec:uc-query-document` | document を読み _index を動的計算する | `.has-udd/skills/harness-query-engine.json` | 未着手 |

> 更新ルール: コードに `@spec:` を先置きしたら必ずこの表に1行足す。Sprint 4 で「将来の Spec」を実際に作成したら状態を更新。
