use std::fmt::Debug;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{rpc::*, handle::Handle, tag::Tag};

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
pub enum Value {
  N(Decimal),
  B(bool),
  F(f64),
  I(i64),
  S(String),
  L(Vec<Value>),
  A{value: Vec<Value>, dims: Vec<u32>},
  R{value: Vec<Value>, fields: u32},
}

impl Default for Value {
  fn default() -> Self {
    Value::N(dec!(0))
  }
}


impl From<usize> for Value {
  fn from(value: usize) -> Self {
      Value::I(value as i64)
  }
}

impl From<f64> for Value {
  fn from(value: f64) -> Self {
      Value::F(value as f64)
  }
}

impl From<i64> for Value {
  fn from(value: i64) -> Self {
      Value::I(value as i64)
  }
}

impl From<bool> for Value {
  fn from(value: bool) -> Self {
      Value::B(value)
  }
}
impl From<Decimal> for Value {
  fn from(value: Decimal) -> Self {
      Value::N(value)
  }
}

fn join_cell_values<'a>(iter: core::slice::Iter<'_, Value>, sep: &str) -> String {
  iter.map(ToString::to_string)
      .collect::<Vec<String>>()
      .join(sep)
}

impl ToString for Value {
  fn to_string(&self) -> String {
    use Value::*;

    match &self {
      N(value) => value.to_string(),
      B(value) => value.to_string(),
      F(value) => value.to_string(),
      I(value) => value.to_string(),
      S(value) => value.clone(),
      L(value) => 
        value.into_iter().map(ToString::to_string).collect::<Vec<String>>().join(","),
      A{value, dims} =>
        value.iter().map(ToString::to_string).collect::<Vec<String>>().join(","),
      R{value, fields} => {
        let kvs: Vec<String> = 
          value.chunks(2)
               .map(|p| join_cell_values(p.into_iter(), ":"))
               .collect();
        kvs.join(",")
      }
    }
  }
}

impl RenderValue for Value {
  fn render(&self) -> ValueUi {
    use Value::*;
    match &self {
      N(value) => 
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Number,
          value: value.to_string(),
        }),
      B(value) => 
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Boolean,
          value: value.to_string(),
        }),
      F(value) => 
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Number,
          value: value.to_string(),
        }),
      I(value) => 
        ValueUi::V(ScalarValueUi{
          typ: TypeUi::Number,
          value: value.to_string(),
        }),
      S(value) =>
        ValueUi::V(ScalarValueUi { 
          typ: TypeUi::String, 
          value: value.clone(),
        }),
      L(value) =>
        ValueUi::L(ListValueUi {
          typ: TypeUi::List,
          value: value.into_iter().map(|cell| cell.to_string()).collect(),
        }),
      A{value, dims} => 
        ValueUi::A(ArrayValueUi { 
          typ: TypeUi::Array, 
          value: value.into_iter().map(|cell| cell.to_string()).collect(), 
          dims: dims.clone(),
        }),
      R{value, fields} => 
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
  pub value: Value,
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


impl<T: Into<Value>> From<Vec<T>> for Cell {
  fn from(value: Vec<T>) -> Self {
    let values: Vec<Value> = value.into_iter().map(Into::into).collect();
    let formula = join_cell_values((&values).iter(), ",");
    Cell {
      value: Value::L(values),
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



