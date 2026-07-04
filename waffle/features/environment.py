"""behave 環境フック。scaffold シナリオが書いた一時 source document を後始末する。"""
import glob
import os


def after_scenario(context, scenario):
    # scaffold.feature が x-source-target に書く一時骨格を削除（source を汚さない）
    for p in glob.glob(".waffle/documents/skills/scaffold-demo*.json"):
        os.remove(p)
