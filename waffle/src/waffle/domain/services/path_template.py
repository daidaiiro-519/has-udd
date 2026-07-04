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
    """具体パスとテンプレートを突き合わせ、{変数名} の実際の値を辞書で返す。一致しなければ None。"""
    pattern = re.escape(template)
    var_names = re.findall(r"\\\{(\w+)\\\}", pattern)
    for name in var_names:
        pattern = pattern.replace(re.escape("{" + name + "}"), f"(?P<{name}>[^/]+)")
    m = re.fullmatch(pattern, concrete_path)
    return m.groupdict() if m else None
