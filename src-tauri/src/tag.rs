use serde::{Serialize, Deserialize};
use crate::handle::{PosHdl, Handle};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct Tag(pub usize);

impl Tag {
  pub fn next(&self) -> Tag {
    Tag(self.0 + 1)
  }

  pub fn handle<const CARD: usize>(&self, pos: [usize; CARD]) -> impl Handle<CARD> {
    PosHdl::new(*self, pos)
  }
}
