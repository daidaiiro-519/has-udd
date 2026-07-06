"""schema_versioning — schemaRef(例: 'Foo/v2')からバージョン番号を取り出す純粋な
ドメインサービス。check_schema_version_drift.pyがドキュメントの参照バージョンと
最新バージョンを数値として正しく比較するために使う（文字列ソートでは 'v10' が
'v2' より前に来てしまう誤りを避ける）。
"""
from __future__ import annotations

def version_number(ref: str) -> int | None:
    """'Foo/v2' や 'v2' からバージョン番号を取り出す。解釈できなければNone。"""
    version = ref.rpartition("/")[2]
    if not version.startswith("v") or not version[1:].isdigit():
        return None
    return int(version[1:])
