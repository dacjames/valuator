use std::cmp::min;
use std::collections::HashMap;

use crate::board::Board;
use crate::handle::Handle;
use crate::parser::{ValueId, NodeId, Token};
use crate::cell::{Val, Cell};

pub trait EvalContext {
  fn get_value(&self, node: &ValueId) -> &Val;
  fn get_node(&self, node: &NodeId) -> &Node;
  fn cell_value<const CARD: usize>(&self, hdl: impl Handle<CARD>) -> Val;
}
  
pub struct EvalState {
  nodes: HashMap<NodeId, Node>,
  values: HashMap<ValueId, Val>,
  dep_graph: StableGraph<Cell, u32, Directed>,
  board: Board<Cell>,
}

#[allow(unused)]
impl EvalState {
  pub fn new(board: Board<>) -> EvalState {
    EvalState{
      nodes: HashMap::new(),
      values: HashMap::new(),
      dep_graph: StableGraph::new(),
      board: board,
    }
  }

  fn insert(&mut self, node: NodeId, ast: Node) {
    self.nodes.insert(node, ast);
  }

  pub fn load(&mut self, tree: &Vec<Node>) {
    for (i, ast) in tree.iter().enumerate() {
      let node = NodeId(i as u32);
      self.insert(node, ast.to_owned())
    }
  }

  pub fn push_value(&mut self, value: Val) -> ValueId{
    let id = ValueId(self.values.len() as u32);
    self.values.insert(id, value);
    id
  }
}

impl EvalContext for EvalState {
  fn get_value(&self, value: &ValueId) -> &Val {
    self.values.get(value).unwrap()
  }
  fn get_node(&self, node: &NodeId) -> &Node {
    self.nodes.get(node).unwrap()
  }
  fn cell_value<const CARD: usize>(&self, hdl: impl Handle<CARD>) -> Val {
    panic!("not impl") 
  }
}
  
pub const LIST_ELEMS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[allow(unused)]
pub enum Node {
  Zero{},
  Symbol{tok: Token},
  Leaf{tok: Token, value: ValueId},
  BinOp{op: char, lhs: NodeId, rhs: NodeId},
  UniOp{op: char, rhs: NodeId},
  Index{row: NodeId, col: NodeId},
  Addr{row: NodeId, col: NodeId},
  List{elems: [NodeId; LIST_ELEMS], len: usize, link: Option<NodeId>},
}

  use Node::*;
use petgraph::data::Build;
use petgraph::{Graph, Directed};
use petgraph::prelude::DiGraph;
use petgraph::stable_graph::StableGraph;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

impl Default for Node {
  fn default() -> Self {
    Zero { }
  }
}

impl Node {
  pub fn eval(&self, ctx: &impl EvalContext) -> Val {
    match self {
      Leaf{tok: _, value} => ctx.get_value(value).to_owned(),
      BinOp{op, lhs, rhs} => {
        let left = ctx.get_node(lhs).eval(ctx);
        let right = ctx.get_node(rhs).eval(ctx);

        use Val::*;

        let f: fn(Decimal, Decimal) -> Decimal = match *op {
          '+' => |l,r|l + r,
          '-' => |l,r|l - r,
          '/' => |l,r|l / r,
          '*' => |l,r|l * r,
          _ => |_l, _r|Decimal::new(0, 0),
        };

        match (left, right) {
          (List(l), Num(r)) => List(
            l.iter().map(|v|{
              let d = Decimal::from(v);
              Num(f(d, r))
            }).collect()
          ),
          (Num(l), List(r)) => List(
            r.iter().map(|v|{
              let d = Decimal::from(v);
              Num(f(l, d))
            }).collect()
          ),
          (Num(l), Num(r)) => Num(f(l,r)),
          (Num(l), Int(r)) => Num(f(l, Decimal::from(r))),
          (Int(l), Num(r)) => Num(f(Decimal::from(l), r)),
          (Num(l), Float(r)) => Num(f(l, Decimal::from_f64(r).unwrap())),
          (Float(l), Num(r)) => Num(f(Decimal::from_f64(l).unwrap(), r)),
          (Num(l), Bool(r)) => Num(f(l, Decimal::from(&Bool(r)))),
          (Bool(l), Num(r)) => Num(f(Decimal::from(&Bool(l)), r)),
          _ => Val::Num(Decimal::from(0)),
        }
      },

      List { elems, len, link } => {
        let clamped_len = min(*len, LIST_ELEMS);
        let mut vals: Vec<Val> = elems.iter().take(clamped_len).map(|nid|{
          let node = ctx.get_node(nid);
          node.eval(ctx)
        }).collect();

        if *len > clamped_len {
          let rest = ctx.get_node(&link.unwrap()).eval(ctx);
          match rest {
            Val::List(l) => vals.extend(l),
            _ => (),
          }
        }
        Val::List(vals)
      }
      _ => Val::default(),
    }
  }
}
