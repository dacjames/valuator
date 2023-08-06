use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use std::cmp;

use crate::tag::Tag;
use crate::constants::*;
use crate::handle::Handle;

#[derive(Debug)]
struct CellData<Cell: Default + Copy + std::fmt::Debug, const N: usize> {
  cells: [Cell; N],
}
 
#[derive(Debug)]
pub struct Tile<Cell: Default + Copy + std::fmt::Debug>{
  pub tag: Tag,
  pub rows: usize,
  pub cols: usize,
  data: CellData<Cell, {ROW_MAX * COL_MAX}>,
  lbls: [String; ROW_MAX + COL_MAX], 
}

pub trait TileTrait<Cell: Default + Copy + ToString + std::fmt::Debug> {
  fn new(tag: Tag) -> Tile<Cell>;
  fn len(&self) -> usize;

  fn get_hdl<const CARD: usize>(&self, handle: &impl Handle<CARD>) -> Cell;
  fn get_pos<const CARD: usize>(&self, pos: [usize; CARD]) -> Cell;
  fn get_lbl<const CARD: usize>(&self, lbls: [String; CARD]) -> Cell;

  fn set_hdl<const CARD: usize>(&mut self, handle: &impl Handle<CARD>, data: Cell); 
  fn set_pos<const CARD: usize>(&mut self, pos: [usize; CARD], data: Cell);
  fn set_lbl<const CARD: usize>(&mut self, lbls: [String; CARD], data: Cell);
}


impl<Cell: Default + Copy + ToString + std::fmt::Debug> TileTrait<Cell> for Tile<Cell>{
  fn new(tag: Tag) -> Tile<Cell> {
    let mut lbls: [String; ROW_MAX + COL_MAX] = Default::default();

    ('a' ..= 'z').take(COL_MAX).enumerate().for_each( |(i, ch)| {
      lbls[i] = ch.to_string().to_ascii_uppercase();
    });

    (1 ..= ROW_MAX).take(ROW_MAX).enumerate().for_each( |(i, n)| {
      lbls[COL_MAX + i] = n.to_string();
    });
    
    return Tile {
      tag: tag,
      rows: 0,
      cols: 0,
      data: CellData {  cells: [Cell::default(); {ROW_MAX * COL_MAX}] },
      lbls: lbls,
    }
  }

  fn len(&self) -> usize {
    return self.rows * self.cols;
  }

  fn get_hdl<const CARD: usize>(&self, handle: &impl Handle<CARD>) -> Cell {
    return self.data.cells[handle.index()];
  }

  fn get_pos<const CARD: usize>(&self, pos: [usize; CARD]) -> Cell {
    return self.get_hdl(&self.tag.handle(pos));
  }

  fn get_lbl<const CARD: usize>(&self, lbls: [String; CARD]) -> Cell {
    let pos = self.pos_for(lbls);
    return self.get_pos(pos);
  }

  fn set_pos<const CARD: usize>(&mut self, pos: [usize; CARD], data: Cell) {
    return self.set_hdl(&self.tag.handle(pos), data)
  }

  fn set_lbl<const CARD: usize>(&mut self, lbls: [String; CARD], data: Cell) {
      let pos = self.pos_for(lbls);
      self.set_pos(pos, data);
  }

  fn set_hdl<const CARD: usize>(&mut self, handle: &impl Handle<CARD>, data: Cell) {
    if handle.row() >= self.rows {
      self.rows = handle.row() + 1;
    }
    if handle.col() >= self.cols {
      self.cols = handle.col() + 1;
    }

    self.data.cells[handle.index()] = data;
  }
}

impl<Cell: Default + Copy + ToString + std::fmt::Debug> Tile<Cell> {

  fn pos_for<const CARD: usize>(&self, lbls: [String; CARD]) -> [usize; CARD] {
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
  
  pub fn render(&self) -> TileUi {
    let c = self.cols;
    let r = self.rows;
    let l = cmp::max(c, r);
    let mut cells = vec!["".to_string(); c*r];

    // 0, 0 => 0
    // 0, 1 => 2
    // 0, 2 => 4
    // 1, 0 => 1
    // 1, 1 => 3
    // 1, 2 => 5
    for ic in 0..c {
      for ir in 0..r {
        // cells[ir * c + ic] = self.get_pos([ir, ic]).to_string();
        cells[ir * c + ic] = self.get_pos([ic, ir]).to_string();
      }
    }

    return TileUi { 
      rows: r as u32, 
      cells: cells,
      colLabels: self.lbls.iter().take(c).cloned().collect(),
      rowLabels: self.lbls.iter().skip(COL_MAX).take(r).cloned().collect(),
    }
  }
}




#[derive(Serialize, Deserialize, Debug)]
pub struct TileUi {
  rows: u32,
  cells: Vec<String>,
  colLabels: Vec<String>,
  rowLabels: Vec<String>,
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_tile_labels() {
      let mut t = Tile::<isize>::new(Tag(0));
      
      let pos1 = t.pos_for(["A".to_owned()]);
      assert_eq!([0], pos1);

      let pos2 = t.pos_for(["B".to_owned(), "2".to_owned()]);
      assert_eq!([1, 1], pos2);

      assert_eq!([0, 0], t.pos_for(["A".to_owned(), "1".to_owned()]));
      assert_eq!([0, 1], t.pos_for(["A".to_owned(), "2".to_owned()]));

      t.set_pos([0, 0],  1);
      t.set_pos([0, 1], 2);
      t.set_pos([1, 0], 3);
      t.set_pos([1, 1], 4);

      assert_eq!(t.get_lbl(["A".to_owned()]), 1);
      assert_eq!(t.get_lbl(["A".to_owned(), "1".to_owned()]), 1);
      assert_eq!(t.get_lbl(["A".to_owned(), "2".to_owned()]), 2);
    }

    #[test]
    fn test_tile_basics() {
      let mut t = Tile::<isize>::new(Tag(0));
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

    #[test]
    fn test_tile_render() {
      let mut t = Tile::<isize>::new(Tag(0));
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

      let expected_cells: Vec<String> = vec![
        // COL: A          COL: B
        "1".to_owned(), "4".to_owned(),  // ROW: 1
        "2".to_owned(), "5".to_owned(),  // ROW: 2
        "3".to_owned(), "6".to_owned(),  // ROW: 3  
       ];

      assert_eq!(ui.cells, expected_cells);
    }
}