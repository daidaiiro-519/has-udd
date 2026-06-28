"""jsonschema による検証 adapter（Validator 実装）。

外部 library(jsonschema) はこの outbound adapter にのみ閉じ込める（lib-via-adapter 規約）。

@stack:schema-validation
"""
from __future__ import annotations

from jsonschema import Draft202012Validator

from has_udd.application.ports.validator import Validator


class JsonSchemaValidator(Validator):
    def validate(self, document: dict, schema: dict) -> list[str]:
        # has-udd:impl-start
        v = Draft202012Validator(schema)
        errors = sorted(v.iter_errors(document), key=lambda e: list(e.path))
        return [f"{list(e.path)}: {e.message}" for e in errors]
        # has-udd:impl-end
