"""path_confinement — agg-document(Document集約)の不変条件「Document のパス解決は、
いかなる operation・command からも常にプロジェクトルート内に閉じ込められる」を守る
純粋なドメインサービス。ポート・実I/Oを一切必要としない（パス文字列だけを扱う）。
"""
from __future__ import annotations

from pathlib import Path

def is_confined(path: str) -> bool:
    """path がプロジェクトルート内に閉じ込められているか（パストラバーサルを含まないか）を判定する。"""
    return ".." not in Path(path).parts
