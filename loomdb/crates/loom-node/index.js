// LoomDB Node.js エントリポイント。
// ネイティブモジュール（napi-rs でビルドした .node）を読み込む。
// 公開時は @napi-rs/cli のプラットフォーム別パッケージ配布に置き換える予定。
const { LoomDB } = require("./loomdb.node");

module.exports = { LoomDB };
