/**
 * LoomDB — DynamoDB のデータモデル＋JOIN を持つ組込ローカル NoSQL。
 * サーバ不要・DB はファイル1個・同期 API（better-sqlite3 と同じ方針）。
 *
 * 数値の注意: JS の number は f64。整数は ±2^53 まで正確。
 * f64 で正確に表せない大きな N 値は文字列として返る（精度を黙って壊さない）。
 */

/** 素の JS 値がそのまま item の属性値になる（DynamoDB の型記法は不要）。 */
export type Attr = string | number | boolean | null | Attr[] | { [key: string]: Attr };
export type Item = { [key: string]: Attr };

export interface KeySchema {
  pk: string;
  sk?: string;
}

export interface IndexDef {
  name: string;
  key: KeySchema;
}

export interface TableDef {
  name: string;
  key: KeySchema;
  indexes?: IndexDef[];
  ttlAttr?: string;
}

/** ConditionExpression 一式（values/names は式の間で共有される）。 */
export interface ConditionOptions {
  condition?: string;
  values?: { [placeholder: string]: Attr };
  names?: { [placeholder: string]: string };
}

export interface UpdateParams extends ConditionOptions {
  /** UpdateExpression（例: "SET title = :t ADD hits :one"） */
  update: string;
}

export interface QueryParams {
  /** KeyConditionExpression（例: "userId = :u AND begins_with(orderId, :p)"） */
  keyCondition: string;
  filter?: string;
  values?: { [placeholder: string]: Attr };
  names?: { [placeholder: string]: string };
  index?: string;
  limit?: number;
  /** true = 昇順（既定） / false = 降順 */
  scanForward?: boolean;
  /** 前ページの lastEvaluatedKey（不透明トークン） */
  startKey?: string;
}

export interface ScanParams {
  filter?: string;
  values?: { [placeholder: string]: Attr };
  names?: { [placeholder: string]: string };
  limit?: number;
  startKey?: string;
}

export interface Page {
  items: Item[];
  lastEvaluatedKey?: string;
}

export interface JoinInput {
  table: string;
  alias: string;
  index?: string;
}

export interface JoinStep extends JoinInput {
  kind?: "inner" | "left";
  on: { left: string; right: string }[];
}

export interface JoinParams {
  root: JoinInput;
  steps: JoinStep[];
  /** 結合後フィルタ。属性は "alias.attr" で参照（例: "o.amount >= :min"） */
  filter?: string;
  values?: { [placeholder: string]: Attr };
  names?: { [placeholder: string]: string };
  /** 射影（"alias.attr" の配列）。省略時は全属性 */
  select?: string[];
}

export interface JoinResult {
  rows: { [aliasDotAttr: string]: Attr }[];
  /** scan フォールバック等の実行時警告（結合キーへの索引追加を促す） */
  warnings: string[];
}

export class LoomDB {
  /** ファイルを開く（無ければ作成）。 */
  constructor(path: string);

  /** DB を閉じてファイルロックを解放する。以後の操作はエラー。 */
  close(): void;

  createTable(def: TableDef): void;
  deleteTable(name: string): void;
  listTables(): string[];
  /** GSI の後付け追加（既存データをバックフィル）・削除。 */
  updateTable(name: string, changes: { add?: IndexDef[]; remove?: string[] }): void;

  put(table: string, item: Item, options?: ConditionOptions): void;
  get(table: string, key: Item): Item | null;
  /** 旧 item（無ければ null）を返す。 */
  delete(table: string, key: Item, options?: ConditionOptions): Item | null;
  /** 適用後の item 全体（ALL_NEW）を返す。 */
  update(table: string, key: Item, params: UpdateParams): Item;

  query(table: string, params: QueryParams): Page;
  scan(table: string, params?: ScanParams): Page;
  /** LoomDB の差別化: N テーブル JOIN（inner / left・多段）。 */
  join(params: JoinParams): JoinResult;
}
