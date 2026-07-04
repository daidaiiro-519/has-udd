"""schema の allOf if/then から discriminator キー（specKind/codingKind/skillKind 等）を機械的に取り出す。"""
from __future__ import annotations


def discriminator_key(schema: dict) -> str | None:
    if "if" in schema:
        return next(iter(schema["if"].get("properties", {})), None)
    for entry in schema.get("allOf", []):
        if "if" in entry:
            key = next(iter(entry["if"].get("properties", {})), None)
            if key:
                return key
    return None
