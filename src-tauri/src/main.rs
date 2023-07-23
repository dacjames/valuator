// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use std::cmp;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
struct Tag(usize);

impl Tag {
  fn next(&self) -> Tag {
    Tag(self.0 + 1)
  }

  fn handle<const CARD: usize>(&self, pos: [usize; CARD]) -> Handle<CARD> {
    Handle::new(*self, pos)
  }
}

const X_MAX: usize = 6;
const Y_MAX: usize = 4;
// const Z_MAX: usize = 32;

#[derive(Debug)]
struct TileData<Cell: Default + Copy + std::fmt::Debug, const N: usize> {
  cells: [Cell; N],
}
 
#[derive(Debug)]
struct Tile<Cell: Default + Copy + std::fmt::Debug>{
  tag: Tag,
  rows: usize,
  cols: usize,
  data: TileData<Cell, {X_MAX * Y_MAX}>,
}

impl<Cell: Default + Copy + ToString + std::fmt::Debug> Tile<Cell> {
  fn new(tag: Tag) -> Tile<Cell> {
    return Tile {
      tag: tag,
      rows: 0,
      cols: 0,
      data: TileData {  cells: [Cell::default(); {X_MAX * Y_MAX}] },
    }
  }

  fn len(&self) -> usize {
    return self.rows * self.cols;
  }

  fn get_handle<const CARD: usize>(&self, handle: &Handle<CARD>) -> Cell {
    return self.data.cells[handle.index()];
  }

  fn get<const CARD: usize>(&self, pos: [usize; CARD]) -> Cell {
    return self.get_handle(&self.tag.handle(pos));
  }

  fn set<const CARD: usize>(&mut self, handle: &Handle<CARD>, data: Cell) {
    if handle.row() >= self.rows {
      self.rows = handle.row() + 1;
    }
    if handle.col() >= self.cols {
      self.cols = handle.col() + 1;
    }

    self.data.cells[handle.index()] = data;
    // println!("after_set {:?}", self)
  }

  fn render(&self) -> TileUi {
    let r = self.rows;
    let c = self.cols;
    let l = cmp::max(r, c);
    let mut cells = vec!["".to_string(); l*l];

    for ir in 0..r {
      for ic in 0..c {
        cells[ir * r + ic] = self.get([ir, ic]).to_string().to_owned();
      }
    }

    return TileUi { 
      rows: r as u32, 
      cells: cells,
    }
  }
}

struct Handle<const CARD: usize> {
  tag: Tag,
  pos: [usize; CARD],
}

impl<const CARD: usize> Handle<CARD> {
  fn new(tag: Tag, pos: [usize; CARD]) -> Handle<CARD> {
    return Handle::<CARD> {
      tag: tag,
      pos: pos,
    }
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

type CubeMap<Cell> = BTreeMap<Tag, Tile<Cell>>;
struct Board<Cell: Default + Copy + ToString + std::fmt::Debug> {
  next_tag: Tag, 
  tiles: CubeMap<Cell>,
}

impl<Cell: Default + Copy + ToString + std::fmt::Debug> Default for Board<Cell> {
  fn default() -> Board<Cell> {
    Board {
      next_tag: Tag::default(),
      tiles: CubeMap::new(),
    }
  }
}

impl<Cell: Default + Copy + ToString + std::fmt::Debug> Board<Cell> {
  fn add_tile(&mut self) -> Tag{
    let tile_tag = self.next_tag;
    self.tiles.insert(
      tile_tag, 
      Tile::<Cell>::new(tile_tag),
    );
    self.next_tag = self.next_tag.next();
    return tile_tag;
  }

  fn tile(&self, tag: Tag) -> &Tile<Cell> {
    return self.tiles.get(&tag).unwrap()
  }

  fn get_handle<const CARD: usize>(&self, handle: &Handle<CARD>) -> Cell {
    match self.tiles.get(&handle.tag) {
      Some(tile) => tile.get_handle(handle),
      None => Cell::default(),
    }
  }

  fn get<const CARD: usize>(&self, tag: Tag, pos: [usize; CARD]) -> Cell {
    return self.get_handle(&tag.handle(pos));
  }

  fn set_handle<const CARD: usize>(&mut self, handle: &Handle<CARD>, data: Cell) {
    match self.tiles.get_mut(&handle.tag) {
      Some(tile) => tile.set(handle, data),
      None => (),
    };
  }

  fn set<const CARD: usize>(&mut self, tag: Tag, pos: [usize; CARD], data: Cell) {
    self.set_handle(&tag.handle(pos), data);
  }

  fn len(&self) -> usize {
    self.tiles.len()
  }
  
}

#[derive(Serialize, Deserialize, Debug)]
struct TileUi {
  rows: u32,
  cells: Vec<String>,
}


#[tauri::command]
fn tile() -> TileUi {
  let mut board: Board<f64> = Board::default();
  let tag = board.add_tile();

  board.set(tag, [0], 2.0);
  board.set(tag, [1], 17.5);
  board.set(tag, [2], 37.8);
  board.set(tag, [0, 1], 3.0);

  return board.tile(tag).render();
}

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}", name)
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet])
    .invoke_handler(tauri::generate_handler![tile])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}