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
