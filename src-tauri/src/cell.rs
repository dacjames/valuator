use std::fmt::Debug;

use rust_decimal::{Decimal, prelude::{FromPrimitive, ToPrimitive}};
use rust_decimal_macros::dec;

use crate::rpc::*;

pub trait RenderCell {
  fn render(&self) -> CellUi;
}

pub trait RenderValue {
  fn render(&self) -> ValueUi;
}

impl RenderCell for f64 {
  fn render(&self) -> CellUi {
    let s = self.to_string();
    return CellUi {
      value: ValueUi::V(ScalarValueUi{
        typ: TypeUi::Float,
        value: s.clone(),
      }),
      formula: s,
      style: String::new(),
    }
  }
}

impl RenderCell for isize {
  fn render(&self) -> CellUi {
    let s = self.to_string();
    return CellUi {
      value: ValueUi::V(ScalarValueUi{
        typ: TypeUi::Int,
        value: s.clone(),
      }),
      formula: s,
      style: String::new(),
    }
  }
}

pub trait ValueOps:
  Default + Clone + ToString + Debug
  where Self: std::marker::Sized {}

impl<T> ValueOps for T where T:
  Default + Clone + ToString + Debug {}

pub trait CellOps:
  ValueOps + RenderCell
  where Self: std::marker::Sized {
  // This block left intentionally empty
}

impl<T> CellOps for T where T:
  ValueOps + RenderCell {
  // This block left intentionally empty
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[allow(unused)]
pub enum Val {
  Num(Decimal),
  Bool(bool),
  Float(f64),
  Int(i64),
  Str(String),
  List(Vec<Val>),
  Array{value: Vec<Val>, dims: Vec<u32>},
  Record{value: Vec<Val>, fields: u32},
}

impl From<&Val> for Decimal {
  fn from(value: &Val) -> Self {
    use Val::*;
    match value {
      &Num(d) => d,
      &Bool(b) => if b {Decimal::new(1, 0)} else {Decimal::new(0, 0)},
      &Float(f) => Decimal::from_f64(f).unwrap_or_default(),
      &Int(i) => Decimal::from_i64(i).unwrap_or_default(),
      Str(s) => Decimal::from_str_radix(s, 10).unwrap_or_default(),
      List(_) => Decimal::default(),
      Array{value: _, dims: _} => Decimal::default(),
      Record{value: _, fields: _} => Decimal::default(),
    }
  }
}

impl From<Val> for i64 {
  fn from(value: Val) -> Self {
    use Val::*;
    match value {
      Num(d) => d.to_i64().unwrap_or(0),
      Bool(b) => if b {1} else {0},
      Float(f) => f as i64,
      Int(i) => i,
      Str(s)=> s.parse().unwrap(),
      _ => Default::default(),
    }
  }
}

impl From<Val> for String {
  fn from(value: Val) -> Self {
    use Val::*;
    match value {
      Num(d) => d.to_string(),
      Bool(b) => (if b {"true"} else {"false"}).to_owned(),
      Float(f) => f.to_string(),
      Int(i) => i.to_string(),
      Str(s) => s,
      List(elems) => {
        let strs: Vec<String> = elems.iter().map(|e|e.to_string()).collect();
        strs.join(",")
      }
      _ => panic!("to_string not impl"),
    }
  }
}

#[allow(unused)]
impl Val {
  fn is_scalar(self) -> bool {
    use Val::*;
    match self {
      Num(_) => true,
      Bool(_) => true,
      Float(_) => true,
      Int(_) => true,
      Str(_) => true,
      _ => false
    }
  }
}

impl Default for Val {
  fn default() -> Self {
    Val::Num(dec!(0))
  }
}


impl From<usize> for Val {
  fn from(value: usize) -> Self {
      Val::Int(value as i64)
  }
}

impl From<f64> for Val {
  fn from(value: f64) -> Self {
      Val::Float(value as f64)
  }
}

impl From<i64> for Val {
  fn from(value: i64) -> Self {
      Val::Int(value as i64)
  }
}

impl From<bool> for Val {
  fn from(value: bool) -> Self {
      Val::Bool(value)
  }
}
impl From<Decimal> for Val {
  fn from(value: Decimal) -> Self {
      Val::Num(value)
  }
}

fn join_cell_values<'a>(iter: core::slice::Iter<'_, Val>, sep: &str) -> String {
  iter.map(ToString::to_string)
      .collect::<Vec<String>>()
      .join(sep)
}

impl ToString for Val {
  fn to_string(&self) -> String {
    use Val::*;

    match &self {
      Num(value) => value.to_string(),
      Bool(value) => value.to_string(),
      Float(value) => value.to_string(),
      Int(value) => value.to_string(),
      Str(value) => value.clone(),
      List(value) =>
        value.into_iter()
             .map(ToString::to_string)
             .collect::<Vec<String>>().join(","),
      Array{value, dims: _} =>
        value.iter()
             .map(ToString::to_string)
             .collect::<Vec<String>>().join(","),
      Record{value, fields: _} => {
        let kvs: Vec<String> =
          value.chunks(2)
               .map(|p| join_cell_values(p.into_iter(), ":"))
               .collect();
        kvs.join(",")
      }
    }
  }
}

impl RenderValue for Val {
  fn render(&self) -> ValueUi {
    use Val::*;
    match &self {
      Num(value) =>
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Number,
          value: value.to_string(),
        }),
      Bool(value) =>
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Boolean,
          value: value.to_string(),
        }),
      Float(value) =>
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Number,
          value: value.to_string(),
        }),
      Int(value) =>
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Number,
          value: value.to_string(),
        }),
      Str(value) =>
        ValueUi::V(ScalarValueUi {
          typ: TypeUi::String,
          value: value.clone(),
        }),
      List(value) =>
        ValueUi::L(ListValueUi {
          typ: TypeUi::List,
          value: value.into_iter().map(|cell| cell.to_string()).collect(),
        }),
      Array{value, dims} =>
        ValueUi::A(ArrayValueUi {
          typ: TypeUi::Array,
          value: value.into_iter().map(|cell| cell.to_string()).collect(),
          dims: dims.clone(),
        }),
      Record{value, fields} =>
        ValueUi::R(RecordValueUi {
          typ: TypeUi::Record,
          value: value.into_iter().map(|cell| cell.to_string()).collect(),
          fields: *fields,
        }),
    }
  }
}

#[derive(Default, Debug, Clone)]
pub struct Cell {
  pub value: Val,
  pub formula: String,
  pub style: String,
}

impl From<usize> for Cell {
  fn from(value: usize) -> Self {
    Cell{
      value: value.into(),
      formula: value.to_string(),
      style: String::new(),
    }
  }
}

impl From<f64> for Cell {
  fn from(value: f64) -> Self {
    Cell{
      value: value.into(),
      formula: value.to_string(),
      style: String::new(),
    }
  }
}

impl From<bool> for Cell {
    fn from(value: bool) -> Self {
        Cell{
          value: value.into(),
          formula: value.to_string().to_lowercase(),
          style: String::new(),
        }
    }
}


impl<T: Into<Val>> From<Vec<T>> for Cell {
  fn from(value: Vec<T>) -> Self {
    let values: Vec<Val> = value.into_iter().map(Into::into).collect();
    let formula = join_cell_values((&values).iter(), ",");
    Cell {
      value: Val::List(values),
      formula: formula,
      style: String::new(),
    }
  }
}

// impl<T> From<Vec<T>> for Cell {
//   fn from(value: Vec<T>) -> Self {
//       Cell {
//         value: CellValue::L
//       }
//   }
// }

impl ToString for Cell {
    fn to_string(&self) -> String {
        return self.value.to_string()
    }
}

impl RenderCell for Cell {
  fn render(&self) -> CellUi {
    return CellUi{
      value: RenderValue::render(&self.value),
      formula: String::new(),
      style: String::new(),
    }
  }
}

// impl ToString for Cell {
//   fn to_string(&self) -> String {
//       return self.value.to_string()
//   }
// }

// impl<T: ToString> From<T> for CellUi {
//   fn from(value: T) -> Self {
//     let s = value.to_string();
//     return CellUi {
//       value: ValueUi::V(ScalarValueUi{
//         typ: TypeUi::String,
//         value: s.clone(),
//       }),
//       formula: s,
//       style: String::new(),
//     }
//   }
// }
