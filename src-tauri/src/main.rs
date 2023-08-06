// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

mod tile;
mod tag;
mod handle;
mod constants;

use tag::Tag;
use tile::TileTrait;
use handle::Handle;


type TileMap<Cell> = BTreeMap<Tag, tile::Tile<Cell>>;
struct Board<Cell: Default + Copy + ToString + std::fmt::Debug> {
  next_tag: Tag, 
  tiles: TileMap<Cell>,
}

impl<Cell: Default + Copy + ToString + std::fmt::Debug> Default for Board<Cell> {
  fn default() -> Board<Cell> {
    Board {
      next_tag: Tag::default(),
      tiles: TileMap::new(),
    }
  }
}

impl<Cell: Default + Copy + ToString + std::fmt::Debug> Board<Cell> {
  fn add_tile(&mut self) -> Tag{
    let tile_tag = self.next_tag;
    self.tiles.insert(
      tile_tag, 
      tile::Tile::<Cell>::new(tile_tag),
    );
    self.next_tag = self.next_tag.next();
    return tile_tag;
  }

  fn get_tile(&self, tag: Tag) -> Option<&tile::Tile<Cell>> {
    self.tiles.get(&tag)
  }

  fn tile(&self, tag: Tag) -> &tile::Tile<Cell> {
    return self.tiles.get(&tag).unwrap()
  }

  fn get_hdl<const CARD: usize>(&self, handle: &impl Handle<CARD>) -> Cell {
    match self.tiles.get(&handle.tag()) {
      Some(tile) => tile.get_hdl(handle),
      None => Cell::default(),
    }
  }

  fn get_pos<const CARD: usize>(&self, tag: Tag, pos: [usize; CARD]) -> Cell {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_pos(pos),
      None => Cell::default(),
    }
  }

  fn get_lbl<const CARD: usize>(&self, tag: Tag, lblbs: [String; CARD]) -> Cell {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_lbl(lblbs),
      None => Cell::default(),
    }
  }

  fn set_hdl<const CARD: usize>(&mut self, handle: &impl Handle<CARD>, data: Cell) {
    match self.tiles.get_mut(&handle.tag()) {
      Some(tile) => tile.set_hdl(handle, data),
      None => (),
    };
  }

  fn set_pos<const CARD: usize>(&mut self, tag: Tag, pos: [usize; CARD], data: Cell) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_pos(pos, data),
      None => (),
    };
  }

  fn set_lbl<const CARD: usize>(&mut self, tag: Tag, lbls: [String; CARD], data: Cell) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_lbl(lbls, data),
      None => (),
    };
  }

  fn len(&self) -> usize {
    self.tiles.len()
  }

}



#[tauri::command]
fn tile() -> tile::TileUi {
  let mut board: Board<f64> = Board::default();
  let tag = board.add_tile();

  board.set_pos(tag, [0, 0], 2.0);
  board.set_pos(tag, [0, 1], 17.5);
  board.set_pos(tag, [0, 2], 37.8);
  board.set_pos(tag, [1, 0], 3.0);

  return board.tile(tag).render();
}


#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}", name)
}

fn main() {
  tauri::Builder::default()
    .manage::<Board<f64>>(Board::default())
    .invoke_handler(tauri::generate_handler![greet])
    .invoke_handler(tauri::generate_handler![tile])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}