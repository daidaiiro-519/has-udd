// loom-node の end-to-end テスト（node:test・実ファイル redb を使用）。
// 重い意味論は Rust 側（loom-bridge / loom-core）で網羅済み。
// ここでは「JS からライブラリとして自然に使えること」を検証する。
import test from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const require = createRequire(import.meta.url);
const { LoomDB } = require("../index.js");

function freshPath() {
  return join(mkdtempSync(join(tmpdir(), "loomdb-")), "test.loom");
}

function ordersDb() {
  const db = new LoomDB(freshPath());
  db.createTable({
    name: "orders",
    key: { pk: "userId", sk: "orderId" },
    indexes: [{ name: "byStatus", key: { pk: "status", sk: "amount" } }],
  });
  for (const [oid, status, amount] of [
    ["o1", "open", 30],
    ["o2", "open", 10],
    ["o3", "shipped", 99],
  ]) {
    db.put("orders", { userId: "u1", orderId: oid, status, amount });
  }
  return db;
}

test("素の JS オブジェクトで put/get が round-trip する", () => {
  const db = ordersDb();
  const item = {
    userId: "u1",
    orderId: "x1",
    amount: 1200,
    ratio: 0.5,
    active: true,
    note: null,
    tags: ["red", "blue"],
    addr: { city: "tokyo" },
  };
  db.put("orders", item);
  const got = db.get("orders", { userId: "u1", orderId: "x1" });
  assert.deepEqual(got, item);
  // 見つからないキーは null
  assert.equal(db.get("orders", { userId: "u1", orderId: "nope" }), null);
});

test("条件付き put の失敗はエラーコード付きで throw される", () => {
  const db = ordersDb();
  const item = { userId: "u1", orderId: "c1" };
  db.put("orders", item, { condition: "attribute_not_exists(userId)" });
  assert.throws(
    () => db.put("orders", item, { condition: "attribute_not_exists(userId)" }),
    (err) => err.message.includes("ConditionalCheckFailed"),
  );
});

test("update は ALL_NEW を返し、ADD は原子カウンタになる", () => {
  const db = ordersDb();
  db.update("orders", { userId: "u1", orderId: "page" }, {
    update: "ADD hits :one",
    values: { ":one": 1 },
  });
  const after = db.update("orders", { userId: "u1", orderId: "page" }, {
    update: "ADD hits :one",
    values: { ":one": 1 },
  });
  assert.equal(after.hits, 2);
});

test("query: filter/values 共有・降順・ページング", () => {
  const db = ordersDb();
  const page = db.query("orders", {
    keyCondition: "userId = :u",
    filter: "amount >= :min",
    values: { ":u": "u1", ":min": 20 },
  });
  assert.deepEqual(page.items.map((i) => i.orderId), ["o1", "o3"]);

  // limit=1 の降順ページングを最後まで回す
  const collected = [];
  let startKey = undefined;
  for (let guard = 0; guard < 5; guard++) {
    const p = db.query("orders", {
      keyCondition: "userId = :u",
      values: { ":u": "u1" },
      scanForward: false,
      limit: 1,
      ...(startKey ? { startKey } : {}),
    });
    collected.push(...p.items.map((i) => i.orderId));
    if (!p.lastEvaluatedKey) break;
    startKey = p.lastEvaluatedKey;
  }
  assert.deepEqual(collected, ["o3", "o2", "o1"]);
});

test("index 指定 query は isk(N) の数値順で返る", () => {
  const db = ordersDb();
  const page = db.query("orders", {
    index: "byStatus",
    keyCondition: "#s = :s",
    names: { "#s": "status" },
    values: { ":s": "open" },
  });
  assert.deepEqual(page.items.map((i) => i.orderId), ["o2", "o1"]); // 10, 30
});

test("JOIN が JS からそのまま使える（LoomDB の差別化）", () => {
  const db = ordersDb();
  db.createTable({ name: "users", key: { pk: "id" } });
  db.put("users", { id: "u1", name: "Alice" });

  const result = db.join({
    root: { table: "orders", alias: "o" },
    steps: [
      { table: "users", alias: "u", kind: "inner",
        on: [{ left: "o.userId", right: "u.id" }] },
    ],
    filter: "o.amount >= :min",
    values: { ":min": 20 },
    select: ["o.orderId", "u.name"],
  });
  assert.equal(result.rows.length, 2); // o1(30), o3(99)
  for (const row of result.rows) {
    assert.equal(row["u.name"], "Alice");
  }
  assert.deepEqual(result.warnings, []);
});

test("テーブル管理: listTables / updateTable(後付け索引バックフィル)", () => {
  const db = ordersDb();
  assert.deepEqual(db.listTables(), ["orders"]);
  db.updateTable("orders", { add: [{ name: "byAmount", key: { pk: "amount" } }] });
  const page = db.query("orders", {
    index: "byAmount",
    keyCondition: "amount = :a",
    values: { ":a": 30 },
  });
  assert.equal(page.items.length, 1);
});

test("永続化: close して開き直してもデータが残っている", () => {
  const path = freshPath();
  const db = new LoomDB(path);
  db.createTable({ name: "kvstore", key: { pk: "id" } });
  db.put("kvstore", { id: "a", v: 42 });
  db.close(); // redb のファイルロックを明示的に解放（better-sqlite3 と同じ流儀）

  const db2 = new LoomDB(path);
  assert.deepEqual(db2.get("kvstore", { id: "a" }), { id: "a", v: 42 });
  // close 後の操作はエラー
  assert.throws(() => db.get("kvstore", { id: "a" }), /closed/);
});
