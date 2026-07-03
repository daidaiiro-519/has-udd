// TypeScript からの利用テスト。
// - `tsc --noEmit` で index.d.ts に対する型チェックが通ること（誤用は @ts-expect-error で検証）
// - `node --experimental-strip-types --test` で実行も通ること
import test from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { Item, JoinResult, Page, TransactWriteOp } from "../index.js";

const require = createRequire(import.meta.url);
// ネイティブモジュールは CJS なので require で読み、型は index.d.ts から付ける
const { LoomDB } = require("../index.js") as typeof import("../index.js");

function freshPath(): string {
  return join(mkdtempSync(join(tmpdir(), "loomdb-ts-")), "test.loom");
}

test("TypeScript: 型付きで一通りの操作が書ける", () => {
  const db = new LoomDB(freshPath());
  db.createTable({
    name: "orders",
    key: { pk: "userId", sk: "orderId" },
    indexes: [{ name: "byStatus", key: { pk: "status", sk: "amount" } }],
  });

  const item: Item = { userId: "u1", orderId: "o1", status: "open", amount: 30 };
  db.put("orders", item, { condition: "attribute_not_exists(userId)" });

  const got: Item | null = db.get("orders", { userId: "u1", orderId: "o1" });
  assert.deepEqual(got, item);

  const page: Page = db.query("orders", {
    keyCondition: "userId = :u",
    values: { ":u": "u1" },
    scanForward: false,
    limit: 10,
  });
  assert.equal(page.items.length, 1);

  const updated: Item = db.update(
    "orders",
    { userId: "u1", orderId: "o1" },
    { update: "ADD hits :one", values: { ":one": 1 } },
  );
  assert.equal(updated.hits, 1);

  db.createTable({ name: "users", key: { pk: "id" } });
  db.put("users", { id: "u1", name: "Alice" });
  const joined: JoinResult = db.join({
    root: { table: "orders", alias: "o" },
    steps: [
      { table: "users", alias: "u", kind: "inner",
        on: [{ left: "o.userId", right: "u.id" }] },
    ],
    select: ["o.orderId", "u.name"],
  });
  assert.equal(joined.rows.length, 1);
  assert.equal(joined.rows[0]["u.name"], "Alice");

  const ops: TransactWriteOp[] = [
    { put: { table: "orders", item: { userId: "u1", orderId: "o2", amount: 5 } } },
    { conditionCheck: { table: "users", key: { id: "u1" },
                        condition: "attribute_exists(id)" } },
  ];
  db.transactWrite(ops);
  const fetched: (Item | null)[] = db.transactGet([
    { table: "orders", key: { userId: "u1", orderId: "o2" } },
  ]);
  assert.equal(fetched[0]?.amount, 5);

  db.batchWrite({ deletes: [{ table: "orders", key: { userId: "u1", orderId: "o2" } }] });
  const swept: number = db.sweepExpired("orders", 100);
  assert.equal(swept, 0); // TTL 未設定テーブルは常に 0

  db.close();
});

test("TypeScript: 誤用はコンパイル時に弾かれる", () => {
  const db = new LoomDB(freshPath());
  db.createTable({ name: "docs", key: { pk: "id" } });

  // @ts-expect-error — keyCondition は必須
  const bad1 = () => db.query("docs", { values: { ":u": "x" } });
  // @ts-expect-error — item に関数は入れられない（Attr 型違反）
  const bad2 = () => db.put("docs", { id: "a", fn: () => 1 });
  // @ts-expect-error — scanForward は boolean
  const bad3 = () => db.query("docs", { keyCondition: "id = :i", scanForward: "yes" });
  // @ts-expect-error — createTable に key は必須
  const bad4 = () => db.createTable({ name: "xxx" });
  // @ts-expect-error — transactWrite の op は put/update/delete/conditionCheck のみ
  const bad5 = () => db.transactWrite([{ teleport: { table: "docs" } }]);

  // 実行はしない（型チェック専用）。未使用警告を避けるためだけに触れておく
  assert.equal(typeof bad1, "function");
  assert.equal(typeof bad2, "function");
  assert.equal(typeof bad3, "function");
  assert.equal(typeof bad4, "function");
  assert.equal(typeof bad5, "function");
  db.close();
});
