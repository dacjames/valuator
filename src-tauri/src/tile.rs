use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use std::cmp;

const X_MAX: usize = 6;
const Y_MAX: usize = 6;
// const Z_MAX: usize = 32;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Tag(usize);

impl Tag {
  pub fn next(&self) -> Tag {
    Tag(self.0 + 1)
  }

  pub fn handle<const CARD: usize>(&self, pos: [usize; CARD]) -> impl Handle<CARD> {
    PosHdl::new(*self, pos)
  }
}

#[derive(Debug)]
struct CellData<Cell: Default + Copy + std::fmt::Debug, const N: usize> {
  cells: [Cell; N],
}
 
#[derive(Debug)]
pub struct Tile<Cell: Default + Copy + std::fmt::Debug>{
  tag: Tag,
  rows: usize,
  cols: usize,
  data: CellData<Cell, {X_MAX * Y_MAX}>,
  lbls: [String; {X_MAX + Y_MAX}], 
}

pub trait TileTrait<Cell: Default + Copy + ToString + std::fmt::Debug> {
  fn new(tag: Tag) -> Tile<Cell>;
  fn len(&self) -> usize;

  fn get_hdl<const CARD: usize>(&self, handle: &impl Handle<CARD>) -> Cell;
  fn get_pos<const CARD: usize>(&self, pos: [usize; CARD]) -> Cell;
  fn get_lbl<const CARD: usize, S>(&self, pos: [S; CARD]) -> Cell where S: Into<String>;

  fn set_hdl<const CARD: usize>(&mut self, handle: &impl Handle<CARD>, data: Cell); 
  fn set_pos<const CARD: usize>(&mut self, tag: Tag, pos: [usize; CARD], data: Cell);
  fn set_lbl<const CARD: usize, S>(&mut self, tag: Tag, pos: [S; CARD], data: Cell) where S: Into<String>;
}


impl<Cell: Default + Copy + ToString + std::fmt::Debug> TileTrait<Cell> for Tile<Cell>{
  fn new(tag: Tag) -> Tile<Cell> {
    let mut lbls: [String; {X_MAX + Y_MAX}] = Default::default();

    (1 ..= X_MAX).take(X_MAX).enumerate().for_each( |(i, n)| {
      lbls[i] = n.to_string();
    });
    ('a' ..= 'z').take(Y_MAX).enumerate().for_each( |(i, ch)| {
      lbls[X_MAX + i] = ch.to_string();
    });

    return Tile {
      tag: tag,
      rows: 0,
      cols: 0,
      data: CellData {  cells: [Cell::default(); {X_MAX * Y_MAX}] },
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

  fn get_lbl<const CARD: usize, S>(&self, pos: [S; CARD]) -> Cell where S: Into<String> {
      panic!()
  }

  fn set_pos<const CARD: usize>(&mut self, tag: Tag, pos: [usize; CARD], data: Cell) {
    return self.set_hdl(&tag.handle(pos), data)
  }

  fn set_lbl<const CARD: usize, S>(&mut self, tag: Tag, pos: [S; CARD], data: Cell) where S: Into<String> {
      panic!()
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
  pub fn render(&self) -> TileUi {
    let r = self.rows;
    let c = self.cols;
    let l = cmp::max(r, c);
    let mut cells = vec!["".to_string(); c*r];


    for ir in 0..r {
      for ic in 0..c {
        cells[ir * c + ic] = self.get_pos([ir, ic]).to_string();
      }
    }

    return TileUi { 
      rows: r as u32, 
      cells: cells,
      rowLabels: self.lbls.iter().take(r).cloned().collect(),
      colLabels: self.lbls.iter().skip(X_MAX).take(c).cloned().collect(),
    }
  }
}


pub enum Hdl<const CARD: usize> {
  PosHdl(PosHdl<CARD>),
  LblHdl(LblHdl<CARD>),
}


pub struct LblHdl<const CARD: usize> {
  pub tag: Tag,
  pub pos: [String; CARD],
}

pub struct PosHdl<const CARD: usize> {
  pub tag: Tag,
  pub pos: [usize; CARD],
}


pub trait Handle<const CARD: usize> {
  fn card(&self) -> usize {
    return CARD;
  }

  fn tag(&self) -> Tag;
  fn index(&self) -> usize;

  fn row(&self) -> usize;
  fn col(&self) -> usize;
  fn time(&self)  -> usize;
}

impl<const CARD: usize> Handle<CARD> for PosHdl<CARD> {
  fn tag(&self) -> Tag {
      return self.tag
  }

  fn row(&self) -> usize {
    if CARD < 1 {
      return 0;
    } 
    self.pos[0]
  }

  fn col(&self) -> usize {
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
      1 => {let x = self.pos[0]; x},
      2 => {
        let x= self.pos[0];
        let y = self.pos[1];
        (y * X_MAX) + x
      }
      _ => panic!("bad CARD")
    }
  }
}

impl<const CARD: usize> PosHdl<CARD> {
  pub fn new(tag: Tag, pos: [usize; CARD]) -> PosHdl<CARD> {
    return PosHdl::<CARD> {
      tag: tag,
      pos: pos,
    }
  }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct TileUi {
  rows: u32,
  cells: Vec<String>,
  rowLabels: Vec<String>,
  colLabels: Vec<String>,
}