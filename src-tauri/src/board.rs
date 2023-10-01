use std::collections::BTreeMap;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};

use crate::rpc::TileUi;
use crate::tile::{Tile, TileTrait};
use crate::tag::Tag;
use crate::handle::Handle;
use crate::cell::CellOps;

type TileMap<V> = BTreeMap<Tag, Tile<V>>;

pub struct Board<V: CellOps> {
  next_tag: Tag, 
  tiles: TileMap<V>,
}

impl<V: CellOps> Default for Board<V> {
  fn default() -> Board<V> {
    Board {
      next_tag: Tag::default(),
      tiles: TileMap::new(),
    }
  }
}

impl<V: CellOps> Board<V> {
  pub fn add_tile(&mut self) -> Tag{
    let tile_tag = self.next_tag;
    self.tiles.insert(
      tile_tag, 
      Tile::<V>::new(tile_tag),
    );
    self.next_tag = self.next_tag.next();
    return tile_tag;
  }

  pub fn get_tile(&self, tag: Tag) -> Option<&Tile<V>> {
    self.tiles.get(&tag)
  }

  pub fn tile(&self, tag: Tag) -> &Tile<V> {
    return self.tiles.get(&tag).unwrap()
  }

  pub fn render_tile(&self, tag: Tag) -> TileUi {
    return self.tiles.get(&tag).unwrap().render()
  }

  pub fn get_hdl<const CARD: usize>(&self, handle: &impl Handle<CARD>) -> V {
    match self.tiles.get(&handle.tag()) {
      Some(tile) => tile.get_hdl(handle),
      None => V::default(),
    }
  }

  pub fn get_pos<const CARD: usize>(&self, tag: Tag, pos: [usize; CARD]) -> V {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_pos(pos),
      None => V::default(),
    }
  }

  pub fn get_lbl<const CARD: usize>(&self, tag: Tag, lblbs: [String; CARD]) -> V {
    return match self.get_tile(tag) {
      Some(tile) => tile.get_lbl(lblbs),
      None => V::default(),
    }
  }

  pub fn set_hdl<const CARD: usize>(&mut self, handle: &impl Handle<CARD>, data: impl Into<V>) {
    match self.tiles.get_mut(&handle.tag()) {
      Some(tile) => tile.set_hdl(handle, data.into()),
      None => (),
    };
  }

  pub fn set_pos<const CARD: usize>(&mut self, tag: Tag, pos: [usize; CARD], data: impl Into<V>) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_pos(pos, data.into()),
      None => (),
    };
  }

  pub fn set_lbl<const CARD: usize>(&mut self, tag: Tag, lbls: [String; CARD], data: impl Into<V>) {
    match self.tiles.get_mut(&tag) {
      Some(tile) => tile.set_lbl(lbls, data.into()),
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