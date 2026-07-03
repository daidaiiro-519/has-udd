/**
 * LoomDB — DynamoDB のデータモデル＋JOIN を持つ組込ローカル NoSQL。
 * サーバ不要・DB はファイル1個・同期 API（better-sqlite3 と同じ方針）。
 *
 * 数値の注意: JS の number は f64。整数は ±2^53 まで正確。
 * f64 で正確に表せない大きな N 値は文字列として返る（精度を黙って壊さない）。
 */

/**
 * 集合型の明示表現（JSON に集合が無いため）。要素は一意に正規化されて保存される。
 * NS の要素は number のほか、f64 で精度が壊れる値のための数値文字列も受け付ける。
 * BS の要素は hex 文字列。
 */
export type StringSet = { $ss: string[] };
export type NumberSet = { $ns: (number | string)[] };
export type BinarySet = { $bs: string[] };

/** 素の JS 値がそのまま item の属性値になる（DynamoDB の型記法は不要）。 */
export type Attr =
  | string
  | number
  | boolean
  | null
  | Attr[]
  | StringSet
  | NumberSet
  | BinarySet
  | { [key: string]: Attr };
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

/** ProjectionExpression（例: "name, addr.city, tags[0]"）。取得属性を絞る。 */
export interface GetOptions {
  projection?: string;
  names?: { [placeholder: string]: string };
}

export interface QueryParams {
  /** KeyConditionExpression（例: "userId = :u AND begins_with(orderId, :p)"） */
  keyCondition: string;
  filter?: string;
  /** ProjectionExpression（Filter の後に適用） */
  projection?: string;
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
  /** ProjectionExpression（Filter の後に適用） */
  projection?: string;
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
  /** 1 ページの最大行数（filter 適用後の出力行で数える） */
  limit?: number;
  /** 前ページの lastEvaluatedKey（不透明トークン） */
  startKey?: string;
}

export interface JoinResult {
  rows: { [aliasDotAttr: string]: Attr }[];
  /** scan フォールバック等の実行時警告（結合キーへの索引追加を促す） */
  warnings: string[];
  /** limit で途中終了した場合の再開トークン。次回の startKey に渡す */
  lastEvaluatedKey?: string;
}

/** transactWrite の 1 操作（put / update / delete / conditionCheck のいずれか1つ）。 */
export type TransactWriteOp =
  | { put: { table: string; item: Item } & ConditionOptions }
  | { update: { table: string; key: Item } & UpdateParams }
  | { delete: { table: string; key: Item } & ConditionOptions }
  | { conditionCheck: { table: string; key: Item; condition: string } & ConditionOptions };

/** transactGet / batchGet / batchWrite の deletes で使うキー参照。 */
export interface KeyRef {
  table: string;
  key: Item;
}

export interface BatchWriteParams {
  puts?: { table: string; item: Item }[];
  deletes?: KeyRef[];
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
  get(table: string, key: Item, options?: GetOptions): Item | null;
  /** 旧 item（無ければ null）を返す。 */
  delete(table: string, key: Item, options?: ConditionOptions): Item | null;
  /** 適用後の item 全体（ALL_NEW）を返す。 */
  update(table: string, key: Item, params: UpdateParams): Item;

  query(table: string, params: QueryParams): Page;
  scan(table: string, params?: ScanParams): Page;
  /** LoomDB の差別化: N テーブル JOIN（inner / left・多段）。 */
  join(params: JoinParams): JoinResult;

  /**
   * 複数操作を 1 トランザクションで all-or-nothing 適用（件数無制限）。
   * 条件不成立時は TransactionCanceled（理由コード配列付き）を throw。
   */
  transactWrite(ops: TransactWriteOp[]): void;
  /** 単一スナップショットで複数キーを読む。結果は同順の item | null。 */
  transactGet(keys: KeyRef[]): (Item | null)[];
  /** ローカルでは transactGet と同一意味論（UnprocessedKeys は常に空）。 */
  batchGet(keys: KeyRef[]): (Item | null)[];
  /** puts / deletes の冪等ループ（件数無制限・UnprocessedItems は常に空）。 */
  batchWrite(params: BatchWriteParams): void;
  /** TTL 失効項目を budget 件まで物理削除し、削除数を返す。 */
  sweepExpired(table: string, budget: number): number;

  /** itemCount は O(1)（書込パスで維持されるカウンタ）・storageBytes はファイルサイズ。 */
  stats(table: string): { itemCount: number; storageBytes: number };
  /** 空き領域の回収（redb の compact）。回収を実行したら true。 */
  compact(): boolean;
}
