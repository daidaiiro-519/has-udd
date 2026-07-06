"""schema_versioning — agg-schema(Schema集約)の不変条件「移行は版を上げる方向にのみ行う」を守る
純粋なドメインサービス。ポート・実I/Oを一切必要としない（schemaRef文字列だけを扱う）。
"""
from __future__ import annotations

def version_number(schema_ref: str) -> int | None:
    """schemaRef(例: 'Foo/v2')からバージョン番号を取り出す。解釈できなければNone。"""
    _, _, version = schema_ref.partition("/")
    if not version.startswith("v") or not version[1:].isdigit():
        return None
    return int(version[1:])

def is_forward_migration(from_ref: str, to_ref: str) -> bool:
    """fromSchemaRef から toSchemaRef への移行が、版を上げる方向かどうかを判定する。
    バージョン番号を解釈できない場合は判定しない（True を返し、既存の挙動を維持する）。
    """
    from_n, to_n = version_number(from_ref), version_number(to_ref)
    if from_n is None or to_n is None:
        return True
    return to_n > from_n
