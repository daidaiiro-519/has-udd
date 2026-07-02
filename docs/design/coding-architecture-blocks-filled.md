<!-- architecture codingKind の各ブロックを「中身入り」で提示。必要十分かをフィールド単位で判断するための材料。題材=python-hexagonal（has-udd 自身の backend スタック）。 -->

# architecture ブロック 内部構造（中身入り）— python-hexagonal

各ブロックの**内部フィールドを実データで埋めた**もの。これで「このブロック/フィールドは要る？足りる？」を判断する。
凡例: 🟩=決定ルール（規約が持つ）／🟦=正確な"形"は examples/ が担う。

---

## Block 1: Summary  〔paragraph〕

```yaml
summary: "backend スタックの技術方式と、DDD 概念をこのスタックでどう実装するかの規約。"
```

---

## Block 2: Layers  〔table〕 🟩

内部: `items[] = { layer, responsibility, mayDependOn[] }`

```yaml
layers:
  - layer: domain
    responsibility: ドメインモデル・不変条件・値
    mayDependOn: []                     # 最内
  - layer: application
    responsibility: usecase の調整・トランザクション境界
    mayDependOn: [domain, ports]
  - layer: ports
    responsibility: 外部との抽象境界（trait/interface）
    mayDependOn: [domain]
  - layer: inbound-adapter
    responsibility: API・CLI・MCP 等の入口
    mayDependOn: [application]
  - layer: outbound-adapter
    responsibility: DB・外部サービスの具体実装
    mayDependOn: [ports]
```

- `mayDependOn` が**依存方向の SSOT**（許可された辺の列挙）。これを超える依存は Rules の error で弾く。
- ポートの居場所もここで表現（1レイヤーとして）。

---

## Block 3: Layout  〔code(tree)〕 🟩

内部: `tree`（正典ディレクトリ）＋ 合成ルートの指定

```yaml
layout: |
  src/has_udd/
    domain/
      model/          # value-object, entity, 集約状態（schema）
      services/       # domain-service, status 遷移 guard
    application/
      usecases/       # 1 usecase = 1 module
      ports/          # trait 定義（driven port）
    adapters/
      inbound/        # cli, mcp
      outbound/       # fs, schema 等
    shared/           # Result, tags, errors
  compositionRoot: src/has_udd/adapters/inbound/*/main  # 結線はここだけ
```

- 「置き場が曖昧」を潰す＝概念の物理位置が一意に。
- **合成ルート（wiring）の場所**を明示（前回の欠けを回収）。

---

## Block 4: ConceptPlacement  〔section×concept ＋ keyvalue〕 🟩＋🟦

内部: `items[] = { concept(固定enum), layer, placement, pattern }`

```yaml
conceptPlacement:
  - concept: usecase
    layer: application
    placement: src/has_udd/application/usecases/{name}.py
    pattern: エントリ関数1つ・txn 境界をここで張る・docstring に @spec   # 🟦 run() の正確な形は sample
  - concept: aggregate
    layer: domain
    placement: （集約クラスは作らない）
    pattern: 不変条件=schema 宣言／status 遷移=guard(domain/services)
  - concept: value-object
    layer: domain
    placement: src/has_udd/domain/model/
    pattern: 不変（frozen）                                            # 🟦 具体形は sample
  - concept: entity
    layer: domain
    placement: src/has_udd/domain/model/
    pattern: 同一性=id・可変属性は最小
  - concept: domain-service
    layer: domain
    placement: src/has_udd/domain/services/
    pattern: ステートレス・複数集約を跨る計算
  - concept: inbound-adapter
    layer: inbound-adapter
    placement: src/has_udd/adapters/inbound/{tool}/
    pattern: 変換のみ・application を呼ぶ・ロジックを持たない
  - concept: outbound-adapter
    layer: outbound-adapter
    placement: src/has_udd/adapters/outbound/
    pattern: ports の trait を実装・外部ライブラリをここに閉じ込め
```

- **concept は固定 enum**（DDD building block）。アーキで変わるのは `layer`/`placement`/`pattern` だけ。
- `pattern` は**決定レベルの指針**（🟩）。**正確なコード形（シグネチャ・port trait の書き方）は examples/**（🟦）。

---

## Block 5: Rules  〔list〕 🟩

内部: `items[] = { rule, severity }`

```yaml
rules:
  - rule: 依存は内向きのみ（Layers.mayDependOn を超える依存禁止）
    severity: error
  - rule: 外部ライブラリは outbound-adapter 経由（domain/application から直接呼ばない）
    severity: error
  - rule: 外部能力（@stack capability）は port 経由で使う
    severity: error
  - rule: 公開 API は Result を返す・ライブラリで unwrap/panic 禁止
    severity: error
  - rule: 合成ルート（結線）は compositionRoot にのみ置く
    severity: error
  - rule: 標準出力への print をライブラリに書かない（logging capability）
    severity: warn
```

- 前回洗い出した欠け（ポート経由・エラー方針・合成ルート・no-print）を**ここに構造化**。散文で厚くしない。

---

## Block 6: ThicknessBySubdomain  〔table〕 🟩

内部: `items[] = { category, thickness }`（値は subdomain spec の Category を参照＝複製しない）

```yaml
thicknessBySubdomain:
  - category: 中核
    thickness: 厚い設計（明示的なドメインモデル）
  - category: 一般
    thickness: ライブラリを adapter で薄く包む
  - category: 補完
    thickness: 最小のトランザクションスクリプト
```

---

## 必要十分の自己点検（判断材料）

| 問い | 現状 |
|---|---|
| コードの**置き場**が一意に決まるか | ✅ Layout＋ConceptPlacement.placement |
| **依存方向**が機械判定できるか | ✅ Layers.mayDependOn＋Rules |
| **ポート/合成ルート/エラー方針/能力使用**の決定があるか | ✅ Rules（前回の欠け回収） |
| 各概念の**実装方針**があるか | 🟩 ConceptPlacement.pattern（決定レベル） |
| 各概念の**正確なコード形** | 🟦 examples/ に委譲（規約は持たない・意図的） |
| subdomain 厚みの**二重管理**回避 | ✅ Category を参照 |

→ 「規約＝決定ルール（🟩）」としては**必要十分に近い**。🟦（正確な形）は sample が担う分担。ここで「pattern が緩い／このフィールドが足りない／このブロックは要らない」を判断してほしい。
