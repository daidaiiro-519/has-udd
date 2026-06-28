"""schema 解決 adapter（SchemaRepository 実装）。

schema はパッケージ内 `has_udd/domain/model/` に閉じる（ユーザープロジェクトに配布しない）。
importlib.resources でパッケージから解決する（インストール後も動く）。

@stack:filesystem
"""
from __future__ import annotations

import json
from importlib import resources

from has_udd.application.ports.schema_repository import SchemaRepository

_MODEL_PACKAGE = "has_udd.domain.model"


class PackageSchemaRepository(SchemaRepository):
    def load(self, schema_ref: str) -> dict:
        # schema_ref 例: "SkillSchema/v1" -> has_udd/domain/model/SkillSchema/v1.json
        # has-udd:impl-start
        ref = resources.files(_MODEL_PACKAGE)
        *dirs, name = schema_ref.split("/")
        for d in dirs:
            ref = ref / d
        text = (ref / f"{name}.json").read_text(encoding="utf-8")
        return json.loads(text)
        # has-udd:impl-end
