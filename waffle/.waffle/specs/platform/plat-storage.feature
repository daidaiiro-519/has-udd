Feature: plat-storage

  Given ストレージ資源が定義されている
  When そのIaC構成を検証する
  Then 暗号化が無効な状態では構成が拒否される
