use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::tile::{Tile, TileTrait};
use crate::tag::Tag;
use crate::handle::Handle;

type TileMap<Cell> = BTreeMap<Tag, Tile<Cell>>;

pub struct Board<Cell: Default + Copy + ToString + Debug> {
  next_tag: Tag, 
  tiles: TileMap<Cell>,
}

impl<Cell: Default + Copy + ToString + Debug> Default for Board<Cell> {
  fn default() -> Board<Cell> {
    Board {
      next_tag: Tag::default(),
      tiles: TileMap::new(),
    }
  }
}

impl<Cell: Default + Copy + ToString + Debug> Board<Cell> {
  pub fn add_tile(&mut self) -> Tag{
    let tile_tag = self.next_tag;
    self.tiles.insert(
      tile_tag, 
      Tile::<Cell>::new(tile_tag),
    );
    self.next_tag = self.next_tag.next();
    return tile_tag;
  }

  pub fn get_tile(&self, tag: Tag) -> Option<&Tile<Cell>> {
    self.tiles.get(&tag)
  }

  pub fn tile(&self, tag: Tag) -> &Tile<Cell> {
    return self.tiles.get(&tag).unwrap()
  }

  pub fn get_hdl<const CARD: usize>(&self, handle: &impl Handle<CARD>) -> Cell {
    match self.tiles.get(&handle.tag()) {
      Some(tile) => tile.get_hdl(handle),
      None => Cell::default(),
    }
  }

  pub fn get_pos<const CARD: usize>(&self, tag: Tag, pos: [usize; CARD]) -> Cell {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_pos(pos),
      None => Cell::default(),
    }
  }

  pub fn get_lbl<const CARD: usize>(&self, tag: Tag, lblbs: [String; CARD]) -> Cell {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_lbl(lblbs),
      None => Cell::default(),
    }
  }

  pub fn set_hdl<const CARD: usize>(&mut self, handle: &impl Handle<CARD>, data: Cell) {
    match self.tiles.get_mut(&handle.tag()) {
      Some(tile) => tile.set_hdl(handle, data),
      None => (),
    };
  }

  pub fn set_pos<const CARD: usize>(&mut self, tag: Tag, pos: [usize; CARD], data: Cell) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_pos(pos, data),
      None => (),
    };
  }

  pub fn set_lbl<const CARD: usize>(&mut self, tag: Tag, lbls: [String; CARD], data: Cell) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_lbl(lbls, data),
      None => (),
    };
  }

  pub fn len(&self) -> usize {
    self.tiles.len()
  }

}