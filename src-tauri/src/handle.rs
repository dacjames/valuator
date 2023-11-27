use crate::cell::CellId;
use crate::constants::*;

pub fn pos_to_index(col: usize, row: usize) -> usize {
  (row * COL_MAX) + col
}

pub fn index_to_pos(index: usize) -> (usize, usize) {
  let row = index / COL_MAX;
  let col = index % COL_MAX;
  (col, row)
}

// TODO Tile should own cellid calculations
pub fn pos_to_cellid<const CARD: usize>(pos: [usize; CARD]) -> CellId {
  let mut col = 0;
  let mut row = 0;
  if CARD == 1 {
    col = pos[0];
  } else if CARD == 2 {
    col = pos[0];
    row = pos[1];
  } else if CARD >= 3 {
    panic!("bad cardinality")
  }

  CellId(pos_to_index(col, row) as u32)
}
