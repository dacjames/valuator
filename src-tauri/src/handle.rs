use crate::constants::*;
use crate::tag::Tag;

#[allow(unused)]
pub enum Hdl<const CARD: usize> {
  PosHdl(PosHdl<CARD>),
  LblHdl(LblHdl<CARD>),
}
  
#[allow(unused)]
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
        (row * COL_MAX) + col
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