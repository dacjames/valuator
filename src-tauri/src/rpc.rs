use serde::{Serialize, Deserialize};
use serde_repr::Serialize_repr;

use crate::tile::TileId;


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct TileUi {
  pub tag: TileId,
  pub rows: u32,
  pub cells: Vec<CellUi>,
  pub colLabels: Vec<String>,
  pub rowLabels: Vec<String>,
}

#[derive(Serialize_repr, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeUi {
  Number,
  Boolean,
  Float,
  Int,
  String,
  List,
  Array,
  Record,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ScalarValueUi {
  pub typ: TypeUi,
  pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ListValueUi {
  pub typ: TypeUi,
  pub value: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ArrayValueUi {
  pub typ: TypeUi,
  pub value: Vec<String>,
  pub dims: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordValueUi {
  pub typ: TypeUi,
  pub value: Vec<String>,
  pub fields: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "typ")]
pub enum ValueUi {
  V(ScalarValueUi),
  L(ListValueUi),
  A(ArrayValueUi),
  R(RecordValueUi),
}

impl Default for ValueUi {
  fn default() -> Self {
      return ValueUi::V(ScalarValueUi { 
        typ: TypeUi::String, 
        value: String::new(),
      })
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CellUi {
  pub value: ValueUi,
  pub formula: String,
  pub style: String,
}

impl Default for CellUi {
  fn default() -> Self {
      return CellUi {
        value: Default::default(),
        formula: String::new(),
        style: String::new(),
      }
  }
}
