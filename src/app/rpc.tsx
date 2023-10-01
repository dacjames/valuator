export enum TypeUi {
  Number,
  Boolean,
  Float,
  Int,
  String,
  List,
  Array,
  Record,
}

export interface ScalarValueUi {
  typ: TypeUi,
  value: string,
}

export interface ListValueUi {
  typ: TypeUi,
  value: Array<String>,
}

export interface ArrayValueUi {
  typ: TypeUi,
  value: Array<String>,
  dims: Array<number>,
}

export interface RecordValueUi {
  typ: TypeUi,
  value: Array<String>, // value = [k1, v1, k2, v2, ...]
  fields: number,
}

export type ValueUi = 
  | ScalarValueUi 
  | ListValueUi
  | ArrayValueUi
  | RecordValueUi


export interface CellUi {
  value: ValueUi,
  formula: string,
  style: string,
}

/**
* UI Data for a Tile.
* 
* @interface TileUi 
* @member rows is the number of rows in tile
* @member cells contains the cell contents in row-major order
*/
export interface TileUi {
  tag: number,
  rows: number,
  cells: Array<CellUi>,
  rowLabels: Array<String>,
  colLabels: Array<String>,
}


/**
 * UI Data for a Tile.
 * 
 * @interface BoardUi
 * @member tiles list of tiles in the board
 */
export interface BoardUi {
  tiles: Array<TileUi>,
}
