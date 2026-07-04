"""waffle 契約定数。

CodingSchema/v1 の `x-coding-contract` と一致させる（SSOT は schema 側）。
コードのアンカー（@spec / @stack）と gen-gap マーカーはこの定数で表現する。
"""

SPEC_TAG = "@spec:{spec_id}"
STACK_TAG = "@stack:{capability}"
GENGAP_START = "waffle:impl-start"
GENGAP_END = "waffle:impl-end"
