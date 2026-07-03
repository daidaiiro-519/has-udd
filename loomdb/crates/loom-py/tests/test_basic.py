# loom-py の end-to-end テスト（unittest・実ファイル redb を使用）。
# 重い意味論は Rust 側（loom-bridge / loom-core）で網羅済み。
# ここでは「Python からライブラリとして自然に使えること」を検証する。
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from loomdb import LoomDB  # noqa: E402


def fresh_path():
    return os.path.join(tempfile.mkdtemp(prefix="loomdb-py-"), "test.loom")


def orders_db():
    db = LoomDB(fresh_path())
    db.create_table({
        "name": "orders",
        "key": {"pk": "userId", "sk": "orderId"},
        "indexes": [{"name": "byStatus", "key": {"pk": "status", "sk": "amount"}}],
    })
    for oid, status, amount in [("o1", "open", 30), ("o2", "open", 10), ("o3", "shipped", 99)]:
        db.put("orders", {"userId": "u1", "orderId": oid, "status": status, "amount": amount})
    return db


class TestLoomDB(unittest.TestCase):
    def test_plain_dict_round_trips(self):
        db = orders_db()
        item = {
            "userId": "u1", "orderId": "x1",
            "amount": 1200, "ratio": 0.5, "active": True, "note": None,
            "tags": ["red", "blue"], "addr": {"city": "tokyo"},
            "blob": b"\x01\x02",  # bytes は B 型として round-trip
        }
        db.put("orders", item)
        got = db.get("orders", {"userId": "u1", "orderId": "x1"})
        self.assertEqual(got, item)
        self.assertIsNone(db.get("orders", {"userId": "u1", "orderId": "nope"}))

    def test_python_big_ints_are_exact(self):
        db = orders_db()
        big = 9007199254740993  # 2^53+1: JS の number では壊れるが Python int は正確
        db.put("orders", {"userId": "u1", "orderId": "big", "n": big})
        self.assertEqual(db.get("orders", {"userId": "u1", "orderId": "big"})["n"], big)

    def test_conditional_put_raises_with_code(self):
        db = orders_db()
        item = {"userId": "u1", "orderId": "c1"}
        db.put("orders", item, {"condition": "attribute_not_exists(userId)"})
        with self.assertRaises(RuntimeError) as cm:
            db.put("orders", item, {"condition": "attribute_not_exists(userId)"})
        self.assertIn("ConditionalCheckFailed", str(cm.exception))

    def test_update_returns_all_new_and_counts(self):
        db = orders_db()
        db.update("orders", {"userId": "u1", "orderId": "page"},
                  {"update": "ADD hits :one", "values": {":one": 1}})
        after = db.update("orders", {"userId": "u1", "orderId": "page"},
                          {"update": "ADD hits :one", "values": {":one": 1}})
        self.assertEqual(after["hits"], 2)

    def test_query_filter_and_pagination(self):
        db = orders_db()
        page = db.query("orders", {
            "keyCondition": "userId = :u",
            "filter": "amount >= :min",
            "values": {":u": "u1", ":min": 20},
        })
        self.assertEqual([i["orderId"] for i in page["items"]], ["o1", "o3"])

        collected, start = [], None
        for _ in range(5):
            params = {"keyCondition": "userId = :u", "values": {":u": "u1"},
                      "scanForward": False, "limit": 1}
            if start:
                params["startKey"] = start
            p = db.query("orders", params)
            collected += [i["orderId"] for i in p["items"]]
            start = p.get("lastEvaluatedKey")
            if not start:
                break
        self.assertEqual(collected, ["o3", "o2", "o1"])

    def test_join(self):
        db = orders_db()
        db.create_table({"name": "users", "key": {"pk": "id"}})
        db.put("users", {"id": "u1", "name": "Alice"})
        result = db.join({
            "root": {"table": "orders", "alias": "o"},
            "steps": [{"table": "users", "alias": "u", "kind": "inner",
                       "on": [{"left": "o.userId", "right": "u.id"}]}],
            "filter": "o.amount >= :min",
            "values": {":min": 20},
            "select": ["o.orderId", "u.name"],
        })
        self.assertEqual(len(result["rows"]), 2)  # o1(30), o3(99)
        for row in result["rows"]:
            self.assertEqual(row["u.name"], "Alice")
        self.assertEqual(result["warnings"], [])

    def test_table_management(self):
        db = orders_db()
        self.assertEqual(db.list_tables(), ["orders"])
        db.update_table("orders", {"add": [{"name": "byAmount", "key": {"pk": "amount"}}]})
        page = db.query("orders", {"index": "byAmount",
                                   "keyCondition": "amount = :a", "values": {":a": 30}})
        self.assertEqual(len(page["items"]), 1)

    def test_transact_write_and_get(self):
        db = orders_db()
        db.transact_write([
            {"put": {"table": "orders",
                     "item": {"userId": "u1", "orderId": "t1", "amount": 1}}},
            {"update": {"table": "orders", "key": {"userId": "u1", "orderId": "o1"},
                        "update": "ADD amount :d", "values": {":d": 5}}},
            {"delete": {"table": "orders", "key": {"userId": "u1", "orderId": "o2"}}},
            {"conditionCheck": {"table": "orders", "key": {"userId": "u1", "orderId": "o3"},
                                "condition": "amount = :a", "values": {":a": 99}}},
        ])
        self.assertEqual(db.get("orders", {"userId": "u1", "orderId": "o1"})["amount"], 35)
        self.assertIsNone(db.get("orders", {"userId": "u1", "orderId": "o2"}))

        # 条件不成立 → TransactionCanceled で put もロールバック
        with self.assertRaises(RuntimeError) as cm:
            db.transact_write([
                {"put": {"table": "orders", "item": {"userId": "u1", "orderId": "t2"}}},
                {"conditionCheck": {"table": "orders",
                                    "key": {"userId": "u1", "orderId": "o3"},
                                    "condition": "amount = :a", "values": {":a": -1}}},
            ])
        self.assertIn("TransactionCanceled", str(cm.exception))
        self.assertIsNone(db.get("orders", {"userId": "u1", "orderId": "t2"}))

        # transact_get / batch_get は同順で item | None
        keys = [{"table": "orders", "key": {"userId": "u1", "orderId": "o3"}},
                {"table": "orders", "key": {"userId": "u1", "orderId": "ghost"}}]
        got = db.transact_get(keys)
        self.assertEqual(got[0]["amount"], 99)
        self.assertIsNone(got[1])
        self.assertEqual(db.batch_get(keys), got)

    def test_batch_write_and_sweep_expired(self):
        db = orders_db()
        db.batch_write({
            "puts": [{"table": "orders",
                      "item": {"userId": "u2", "orderId": "b1", "amount": 7}}],
            "deletes": [{"table": "orders", "key": {"userId": "u1", "orderId": "o2"}}],
        })
        self.assertEqual(db.get("orders", {"userId": "u2", "orderId": "b1"})["amount"], 7)
        self.assertIsNone(db.get("orders", {"userId": "u1", "orderId": "o2"}))

        # TTL: 失効項目は読取で隠れ、sweep_expired が物理削除数を返す
        import time
        db.create_table({"name": "sessions", "key": {"pk": "id"}, "ttlAttr": "expiresAt"})
        db.put("sessions", {"id": "old", "expiresAt": 1})  # とうに失効
        db.put("sessions", {"id": "live", "expiresAt": int(time.time()) + 3600})
        self.assertIsNone(db.get("sessions", {"id": "old"}))
        self.assertEqual(db.sweep_expired("sessions", 10), 1)
        self.assertEqual(db.get("sessions", {"id": "live"})["id"], "live")

    def test_sets_round_trip_as_python_sets(self):
        db = orders_db()
        item = {
            "userId": "u1", "orderId": "s1",
            "tags": {"red", "blue"},          # str の set → SS
            "scores": {1, 2.5},               # int/float の set → NS
            "blobs": {b"\x01", b"\x00\xff"},  # bytes の set → BS
        }
        db.put("orders", item)
        got = db.get("orders", {"userId": "u1", "orderId": "s1"})
        self.assertEqual(got, item)

        # 巨大 int も set の中で正確（NS は 10 進文字列で往復する）
        big = 2 ** 100
        db.put("orders", {"userId": "u1", "orderId": "s2", "ns": {big, 1}})
        self.assertEqual(db.get("orders", {"userId": "u1", "orderId": "s2"})["ns"], {big, 1})

        # ADD = 集合和 / DELETE = 集合差（空になったら属性ごと削除）
        after = db.update("orders", {"userId": "u1", "orderId": "s1"},
                          {"update": "ADD tags :t DELETE scores :s",
                           "values": {":t": {"green"}, ":s": {1, 2.5}}})
        self.assertEqual(after["tags"], {"red", "blue", "green"})
        self.assertNotIn("scores", after)

        # 型が混ざった set は TypeError
        with self.assertRaises(TypeError):
            db.put("orders", {"userId": "u1", "orderId": "s3", "bad": {"a", 1}})

    def test_join_pagination(self):
        db = orders_db()
        db.create_table({"name": "users", "key": {"pk": "id"}})
        db.put("users", {"id": "u1", "name": "Alice"})

        collected, start = [], None
        for _ in range(10):
            params = {
                "root": {"table": "orders", "alias": "o"},
                "steps": [{"table": "users", "alias": "u", "kind": "inner",
                           "on": [{"left": "o.userId", "right": "u.id"}]}],
                "select": ["o.orderId"],
                "limit": 1,
            }
            if start:
                params["startKey"] = start
            page = db.join(params)
            self.assertLessEqual(len(page["rows"]), 1)
            collected += [r["o.orderId"] for r in page["rows"]]
            start = page.get("lastEvaluatedKey")
            if not start:
                break
        self.assertEqual(sorted(collected), ["o1", "o2", "o3"])

    def test_projection(self):
        db = orders_db()
        db.put("orders", {"userId": "u1", "orderId": "p1", "amount": 5,
                          "addr": {"city": "tokyo", "zip": "100"}})
        got = db.get("orders", {"userId": "u1", "orderId": "p1"},
                     {"projection": "addr.city, #a", "names": {"#a": "amount"}})
        self.assertEqual(got, {"addr": {"city": "tokyo"}, "amount": 5})

        page = db.query("orders", {"keyCondition": "userId = :u",
                                   "projection": "orderId", "values": {":u": "u1"}})
        for item in page["items"]:
            self.assertEqual(list(item.keys()), ["orderId"])

    def test_persistence_with_close(self):
        path = fresh_path()
        db = LoomDB(path)
        db.create_table({"name": "kvstore", "key": {"pk": "id"}})
        db.put("kvstore", {"id": "a", "v": 42})
        db.close()  # redb のファイルロックを明示的に解放

        db2 = LoomDB(path)
        self.assertEqual(db2.get("kvstore", {"id": "a"}), {"id": "a", "v": 42})
        with self.assertRaises(RuntimeError):
            db.get("kvstore", {"id": "a"})  # close 後の操作はエラー


if __name__ == "__main__":
    unittest.main()
