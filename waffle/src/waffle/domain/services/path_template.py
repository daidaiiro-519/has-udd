"""パステンプレートの解決（順方向）・逆解析（逆方向）を行う純ロジック。

schema の x-source-target/x-render-target が specKind 等でネストしたパスを持つ場合、
create 時は既知の変数（documentId・contextRef 等）からパスを組み立て（resolve）、
render/validate 時はドキュメント自体には保存していない変数（contextRef 等）を、
実際に渡されたパスとテンプレートを突き合わせて逆算する（reverse_parse）。
"""
from __future__ import annotations

import re


def resolve(template: str, **variables) -> str:
    """テンプレート文字列に既知の変数を当てはめて具体パスを作る。"""
    return template.format(**variables)


def reverse_parse(template: str, concrete_path: str) -> dict | None:
    """具体パスとテンプレートを突き合わせ、{変数名} の実際の値を辞書で返す。一致しなければ None。

    同じ変数名がテンプレート内に複数回登場する場合（例: subdomain の自己格納パターン
    ".../{documentId}/{documentId}.json"）は、2回目以降はバックリファレンスにして
    「同じ値が繰り返されている」ことを要求する（Python の正規表現は同名グループを
    重複定義できないため）。
    """
    pattern = re.escape(template)
    var_names = re.findall(r"\\\{(\w+)\\\}", pattern)
    seen: set[str] = set()
    for name in var_names:
        token = re.escape("{" + name + "}")
        replacement = f"(?P<{name}>[^/]+)" if name not in seen else f"(?P={name})"
        pattern = pattern.replace(token, replacement, 1)
        seen.add(name)
    m = re.fullmatch(pattern, concrete_path)
    return m.groupdict() if m else None
