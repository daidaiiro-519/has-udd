"""schema 解決 adapter（SchemaRepository 実装）。

schema はパッケージ内3箇所に閉じる（ユーザープロジェクトに配布しない）:
- `waffle/domain/model/`         Documentのschemaが指しうる型＝集約（identity持ち）
- `waffle/domain/value_objects/` 他schemaに埋め込まれる値オブジェクトの型定義（集約ではない）
- `waffle/application/dto/`      usecaseの出力データの形状定義（業務ロジックではなくusecaseの
  入出力契約なのでapplication層。集約でも値オブジェクトでもない）
importlib.resources でパッケージから解決する（インストール後も動く）。
"""
from __future__ import annotations

import json
from importlib import resources

from waffle.application.ports.schema_repository import SchemaRepository

_PACKAGES = ["waffle.domain.model", "waffle.domain.value_objects", "waffle.application.dto"]

class PackageSchemaRepository(SchemaRepository):
    def load(self, schema_ref: str) -> dict:
        # schema_ref 例: "SkillSchema/v1" -> waffle/domain/model/SkillSchema/v1.json
        *dirs, name = schema_ref.split("/")
        last_error: FileNotFoundError | None = None
        for package in _PACKAGES:
            ref = resources.files(package)
            for d in dirs:
                ref = ref / d
            try:
                text = (ref / f"{name}.json").read_text(encoding="utf-8")
            except FileNotFoundError as e:
                last_error = e
                continue
            return json.loads(text)
        raise last_error
