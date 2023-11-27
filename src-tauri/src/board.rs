use std::collections::BTreeMap;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};

use crate::rpc::TileUi;
use crate::tile::Tile;
use crate::tile::TileId;
use crate::cell::{CellOps, Cell};

type TileMap<V> = BTreeMap<TileId, Tile<V>>;


#[derive(Debug)]
pub struct Board<V: CellOps = Cell> {
  next_tag: TileId,
  tiles: TileMap<V>,
}

impl Board {
  pub fn example() -> (Self, TileId) {
    let mut board = Self::default();
    let tag = board.add_tile();

    board.set_pos(tag, [0, 0], 2.0);
    board.set_pos(tag, [0, 1], 17.5);
    board.set_pos(tag, [0, 2], 37.8);
    board.set_pos(tag, [1, 0], 3.0);

    board.set_pos(tag, [1, 1], vec![1.0, 2.0]);
    board.set_pos(tag, [1, 2], true);
    (board, tag)
  }
}
impl<V: CellOps> Default for Board<V> {
  fn default() -> Board<V> {
    Board {
      next_tag: TileId::default(),
      tiles: TileMap::new(),
    }
  }
}

#[allow(unused)]
impl<V: CellOps> Board<V> {
  pub fn add_tile(&mut self) -> TileId{
    let tile_tag = self.next_tag;
    self.tiles.insert(
      tile_tag,
      Tile::<V>::new(tile_tag),
    );
    self.next_tag = self.next_tag.next();
    return tile_tag;
  }

  pub fn get_tile(&self, tag: TileId) -> Option<&Tile<V>> {
    self.tiles.get(&tag)
  }

  pub fn tile(&self, tag: TileId) -> &Tile<V> {
    return self.tiles.get(&tag).unwrap()
  }

  pub fn render_tile(&self, tag: TileId) -> TileUi {
    return self.tiles.get(&tag).unwrap().render()
  }

  pub fn get_pos<const CARD: usize>(&self, tag: TileId, pos: [usize; CARD]) -> V {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_cell(pos),
      None => V::default(),
    }
  }

  pub fn get_lbl<const CARD: usize>(&self, tag: TileId, lblbs: [String; CARD]) -> V {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_cell(lblbs),
      None => V::default(),
    }
  }

  pub fn set_pos<const CARD: usize>(&mut self, tag: TileId, pos: [usize; CARD], data: impl Into<V>) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_cell(pos, data.into()),
      None => (),
    };
  }

  pub fn set_lbl<const CARD: usize>(&mut self, tag: TileId, lbls: [String; CARD], data: impl Into<V>) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_cell(lbls, data.into()),
      None => (),
    };
  }

  pub fn len(&self) -> usize {
    self.tiles.len()
  }

  pub fn render(&self) -> BoardUi {
    return BoardUi {
      tiles: self.tiles.values().map(|t| { t.render() } ).collect(),
    }
  }

}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct BoardUi {
  tiles: Vec<TileUi>,
}
