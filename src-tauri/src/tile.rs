
use std::fmt;

use serde::{Serialize, Deserialize};

use crate::{constants::*, cell};
use crate::handle::{Handle, PosHdl, pos_to_cellid};
use crate::cell::{CellOps, Val, Cell, CellId};
use crate::rpc::{TileUi, CellUi};


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct TileId(pub usize);

impl TileId {
  pub fn next(&self) -> TileId {
    TileId(self.0 + 1)
  }

  pub fn handle<const CARD: usize>(&self, pos: [usize; CARD]) -> impl Handle<CARD> {
    PosHdl::new(*self, pos)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CellRef<const CARD: usize> {
  Pos([usize; CARD]),
  Label([String; CARD]),
  // Id(CellId),
}

impl<const CARD: usize> From<[usize; CARD]> for CellRef<CARD> {
  fn from(value: [usize; CARD]) -> Self {
    CellRef::Pos(value)
  }
}

impl<const CARD: usize> From<[String; CARD]> for CellRef<CARD> {
  fn from(value: [String; CARD]) -> Self {
    CellRef::Label(value)
  }
}

// impl<const CARD: usize> From<CellId> for TileRef<CARD> {
//   fn from(value: [String; CARD]) -> Self {
//     TileRef::Label(value)
//   }
// }

pub trait TileContext {
  // fn get_pos<const CARD: usize>(&mut self, pos: [usize; CARD]) -> Cell;
  fn get_cell<const CARD: usize, TR: Into<CellRef<CARD>>+fmt::Debug>(&mut self, tileref: TR) -> (CellId, Cell);
}

#[derive(Debug)]
pub struct Tile<Cell: CellOps>{
  pub tag: TileId,
  pub rows: usize,
  pub cols: usize,
  cells: [Cell; ROW_MAX * COL_MAX],
  lbls: [String; ROW_MAX + COL_MAX],
}

pub struct TileIter<'a, Cell: CellOps>{
  tile: &'a Tile<Cell>,
  curr: usize,
}

impl<'a, Cell: CellOps> Iterator for TileIter<'a, Cell> {
  type Item = (CellId, &'a Cell);
  fn next(&mut self) -> Option<Self::Item> {
    // TODO remove empty cells from tile iteration
    if self.curr >= (ROW_MAX * COL_MAX) {
      return None
    }

    let id = CellId(self.curr as u32);
    let cell: &Cell = self.tile.cells.get(self.curr).unwrap();
    self.curr += 1;
    Some((id, cell))
  }
}

impl<Cell: CellOps> Tile<Cell> {
  pub fn iter<'a>(&'a self) -> TileIter<'a, Cell> {
    TileIter{
      tile: self,
      curr: 0
    }
  }
}

impl<Cell: CellOps>  Tile<Cell>{
  pub fn new(tag: TileId) -> Tile<Cell> {
    let mut lbls: [String; ROW_MAX + COL_MAX] = Default::default();

    ('a' ..= 'z').take(COL_MAX).enumerate().for_each( |(i, ch)| {
      lbls[i] = ch.to_string().to_ascii_uppercase();
    });

    (1 ..= ROW_MAX).take(ROW_MAX).enumerate().for_each( |(i, n)| {
      lbls[COL_MAX + i] = n.to_string();
    });

    let cells: [Cell; ROW_MAX * COL_MAX] = std::array::from_fn(|_| Cell::default());

    return Tile {
      tag: tag,
      rows: 0,
      cols: 0,
      cells: cells,
      lbls: lbls,
    }
  }

  pub fn len(&self) -> usize {
    return self.rows * self.cols;
  }

  pub fn get_hdl<const CARD: usize>(&self, handle: &impl Handle<CARD>) -> Cell {
    return self.cells[handle.index()].clone();
  }

  pub fn get_pos<const CARD: usize>(&self, pos: [usize; CARD]) -> Cell {
    return self.get_hdl(&self.tag.handle(pos));
  }

  pub fn get_lbl<const CARD: usize>(&self, lbls: [String; CARD]) -> Cell {
    let pos = self.pos_for(lbls);
    return self.get_pos(pos);
  }

  pub fn set_pos<const CARD: usize>(&mut self, pos: [usize; CARD], data: Cell) {
    return self.set_hdl(&self.tag.handle(pos), data)
  }

  pub fn set_lbl<const CARD: usize>(&mut self, lbls: [String; CARD], data: Cell) {
      let pos = self.pos_for(lbls);
      self.set_pos(pos, data);
  }

  pub fn set_hdl<const CARD: usize>(&mut self, handle: &impl Handle<CARD>, data: Cell) {
    if handle.row() >= self.rows {
      self.rows = handle.row() + 1;
    }
    if handle.col() >= self.cols {
      self.cols = handle.col() + 1;
    }

    self.cells[handle.index()] = data;
  }

  pub fn get_cell(&self, cell: CellId) -> Cell {
    return self.cells[cell.0 as usize].clone()
  }

  pub fn pos_for<const CARD: usize>(&self, lbls: [String; CARD]) -> [usize; CARD] {
    let mut pos: [usize; CARD] = [0; CARD];

    for (i, lbl) in lbls.iter().enumerate() {
      pos[i] =
        match self.lbls.iter().position(
          |hay| { hay.eq(lbl) }
        ) {
          Some(n) => if n < COL_MAX { n } else { n - COL_MAX },
          None => 0,
        };
    };

    return pos
  }

  pub fn resolve<const CARD: usize, R: Into<CellRef<CARD>>+fmt::Debug>(&self, cellref: R) -> CellId {
    let cellref: CellRef<CARD> = cellref.into();
    match cellref {
      CellRef::Pos(pos) => pos_to_cellid(pos),
      CellRef::Label(labels) => pos_to_cellid(self.pos_for(labels)),
    }
  }

  pub fn render(&self) -> TileUi {
    let c = self.cols;
    let r = self.rows;
    let mut cells: Vec<CellUi> = vec![Default::default(); c*r];

    // 0, 0 => 0
    // 0, 1 => 2
    // 0, 2 => 4
    // 1, 0 => 1
    // 1, 1 => 3
    // 1, 2 => 5
    for ic in 0..c {
      for ir in 0..r {
        cells[ir * c + ic] = self.get_pos([ic, ir]).render();
      }
    }

    return TileUi {
      tag: self.tag,
      rows: r as u32,
      cells: cells,
      colLabels: self.lbls.iter().take(c).cloned().collect(),
      rowLabels: self.lbls.iter().skip(COL_MAX).take(r).cloned().collect(),
    }
  }
}






#[cfg(test)]
mod tests {
    use crate::rpc::ScalarValueUi;
    use crate::rpc::{ValueUi, TypeUi};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_tile_labels() {
      let mut t = Tile::<isize>::new(TileId(0));

      let pos1 = t.pos_for(["A".to_owned()]);
      assert_eq!([0], pos1);

      let pos2 = t.pos_for(["B".to_owned(), "2".to_owned()]);
      assert_eq!([1, 1], pos2);

      assert_eq!([0, 0], t.pos_for(["A".to_owned(), "1".to_owned()]));
      assert_eq!([0, 1], t.pos_for(["A".to_owned(), "2".to_owned()]));

      t.set_pos([0, 0], 1);
      t.set_pos([0, 1], 2);
      t.set_pos([1, 0], 3);
      t.set_pos([1, 1], 4);

      assert_eq!(t.get_lbl(["A".to_owned()]), 1);
      assert_eq!(t.get_lbl(["A".to_owned(), "1".to_owned()]), 1);
      assert_eq!(t.get_lbl(["A".to_owned(), "2".to_owned()]), 2);
    }

    #[test]
    fn test_tile_basics() {
      let mut t = Tile::<isize>::new(TileId(0));
      t.set_pos([0, 0],  1);
      t.set_pos([0, 1], 2);
      t.set_pos([1, 0], 3);
      t.set_pos([1, 1], 4);


      assert_eq!(t.get_pos([0]), 1);
      assert_eq!(t.get_pos([0, 0]), 1);
      assert_eq!(t.get_pos([0, 1]), 2);
      assert_eq!(t.get_pos([1]), 3);
      assert_eq!(t.get_pos([1, 0]), 3);
      assert_eq!(t.get_pos([1, 1]), 4);
    }

    // todo fix test_tile_render
    #[ignore = "test is broken, not the unit"]
    #[test]
    fn test_tile_render() {
      let mut t = Tile::<isize>::new(TileId(0));
      t.set_pos([0, 0], 1);
      t.set_pos([0, 1], 2);
      t.set_pos([0, 2], 3);
      t.set_pos([1, 0], 4);
      t.set_pos([1, 1], 5);
      t.set_pos([1, 2], 6);

      let ui = t.render();

      assert_eq!(ui.cells.len() as u32 / ui.rows, 2);
      assert_eq!(ui.rows, 3);

      assert_eq!(ui.colLabels.len(), 2);
      assert_eq!(ui.rowLabels.len(), 3);

      assert_eq!(ui.colLabels, vec![
        "A".to_owned(),
        "B".to_owned(),
      ]);

      fn string_cell(value: &str) -> CellUi {
        return CellUi {
          value: ValueUi::V(ScalarValueUi{typ: TypeUi::String, value: value.to_owned()}),
          formula: value.to_owned(),
          style: String::new(),
        }
      }

      let expected_cells: Vec<CellUi> = vec![
        // COL: A          COL: B
        string_cell("1"), string_cell("4"),  // ROW: 1
        string_cell("2"), string_cell("5"),  // ROW: 2
        string_cell("3"), string_cell("6"),  // ROW: 3
       ];

      assert_eq!(ui.cells, expected_cells);
    }
}
