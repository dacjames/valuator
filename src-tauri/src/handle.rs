use crate::cell::{CellId, Cell};
use crate::constants::*;
use crate::tile::TileId;

#[allow(unused)]
pub enum Hdl<const CARD: usize> {
  Pos(PosHdl<CARD>),
  Label(LableHdl<CARD>),
}
  
#[allow(unused)]
pub struct LableHdl<const CARD: usize> {
  pub tag: TileId,
  pub pos: [String; CARD],
}

pub struct PosHdl<const CARD: usize> {
  pub tag: TileId,
  pub pos: [usize; CARD],
}

pub fn pos_to_index(col: usize, row: usize) -> usize {
  (row * COL_MAX) + col
}

pub fn index_to_pos(index: usize) -> (usize, usize) {
  let row = index / COL_MAX;
  let col = index % COL_MAX;
  (col, row)
}

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


pub trait Handle<const CARD: usize> {
  fn card(&self) -> usize {
    return CARD;
  }

  fn tag(&self) -> TileId;
  fn index(&self) -> usize;

  fn row(&self) -> usize;
  fn col(&self) -> usize;
  fn time(&self)  -> usize;
}

impl<const CARD: usize> Handle<CARD> for PosHdl<CARD> {
  fn tag(&self) -> TileId {
      return self.tag
  }

  fn col(&self) -> usize {
    if CARD < 1 {
      return 0;
    }
    self.pos[0]
  }

  fn row(&self) -> usize {
    if CARD < 2 {
      return 0;
    } 
    self.pos[1]
  }
  
  fn time(&self)  -> usize {
    if CARD < 3 {
      return 0
    }
    self.pos[2]
  }

  fn index(&self) -> usize {
    match CARD {
      1 => {let col = self.pos[0]; col},
      2 => {
        let col = self.pos[0];
        let row = self.pos[1];
        pos_to_index(col, row)
      }
      _ => panic!("bad CARD")
    }
  }
}

impl<const CARD: usize> PosHdl<CARD> {
  pub fn new(tag: TileId, pos: [usize; CARD]) -> PosHdl<CARD> {
    return PosHdl::<CARD> {
      tag: tag,
      pos: pos,
    }
  }
}