use core::fmt;
use std::cmp::min;
use std::collections::HashMap;

use crate::board::Board;
use crate::handle::{index_to_pos, pos_to_index, pos_to_cellid};
use crate::parser::{ValueId, NodeId, Token};
use crate::cell::{Val, Cell, CellId, self};
use crate::tile::{TileId, CellRef};
use crate::tile::TileContext;

use petgraph::{Graph, Directed};
use petgraph::prelude::DiGraph;
use petgraph::stable_graph::{StableGraph, DefaultIx, NodeIndex};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;
use rustc_hash::FxHashMap;


pub trait ObjectContext {
  fn get_value(&self, value: &ValueId) -> &Val;
  fn get_node(&self, node: &NodeId) -> &Node;
}

pub trait EvalContext:
  ObjectContext + TileContext {}

impl<T> EvalContext for T where T:
  ObjectContext + TileContext {}

type DepsIx = DefaultIx;
type DepsGraph = StableGraph<CellId, u32, Directed, DepsIx>;
type DepsLookup = FxHashMap<CellId, NodeIndex<DepsIx>>;

#[derive(Debug)]
pub struct EvalState<'a> {
  nodes: HashMap<NodeId, Node>,
  values: HashMap<ValueId, Val>,
  deps: DepsGraph,
  lookup: DepsLookup,
  cell: CellId,
  board: &'a Board<Cell>,
  tile: TileId,
}

#[allow(unused)]
impl EvalState<'_> {
  pub fn new(board: &Board<>, tile_id: TileId, cell_id: CellId) -> EvalState {
    let mut deps: DepsGraph  = StableGraph::new();
    let mut lookup = DepsLookup::default();
    let tile = board.tile(tile_id);

    tile.iter().for_each(|(id, cell)|{
      let ix = deps.add_node(id);
      println!("wtf: {id:?}");
      lookup.insert(id, ix);
    });

    EvalState{
      nodes: HashMap::new(),
      values: HashMap::new(),
      deps: deps,
      lookup: lookup,
      cell: cell_id,
      board: board,
      tile: tile_id,
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

impl ObjectContext for EvalState<'_> {
  fn get_value(&self, value: &ValueId) -> &Val {
    self.values.get(value).unwrap()
  }
  fn get_node(&self, node: &NodeId) -> &Node {
    self.nodes.get(node).unwrap()
  }
}

impl TileContext for EvalState<'_> {
  fn get_cell<const CARD: usize, R: Into<CellRef<CARD>>>(&mut self, cellref: R) -> (CellId, Cell) {
    let cellref: CellRef<CARD> = cellref.into();
    let tile = self.board.get_tile(self.tile).unwrap();

    let cell_id = tile.resolve(cellref);
    let dep_ix: NodeIndex = self.lookup[&cell_id];
    let self_ix: NodeIndex = self.lookup[&self.cell];
    self.deps.add_edge(self_ix, dep_ix, 1);
    (cell_id, tile.get_cell(cell_id))
  }
}



pub const LIST_ELEMS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[allow(unused)]
pub enum Node {
  Zero{},
  Leaf{value: ValueId},
  BinOp{op: char, lhs: NodeId, rhs: NodeId},
  UniOp{op: char, rhs: NodeId},
  Index{row: NodeId, col: NodeId},
  Addr{row: NodeId, col: NodeId},
  List{elems: [NodeId; LIST_ELEMS], len: usize, link: Option<NodeId>},
}

  use Node::*;

impl Default for Node {
  fn default() -> Self {
    Zero { }
  }
}

impl Node {
  pub fn eval(&self, ctx: &mut impl EvalContext) -> Val {
    match self {
      Leaf{value} => ctx.get_value(value).to_owned(),
      BinOp{op, lhs, rhs} => {
        let lnode = *ctx.get_node(lhs);
        let rnode = *ctx.get_node(rhs);
        let left = lnode.eval(ctx);
        let right = rnode.eval(ctx);

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
          let node = *ctx.get_node(nid);
          node.eval(ctx)
        }).collect();

        if *len > clamped_len {
          let get_node = *ctx.get_node(&link.unwrap());
          let rest = get_node.eval(ctx);
          match rest {
            Val::List(l) => vals.extend(l),
            _ => (),
          }
        }
        Val::List(vals)
      }

      Index { row, col } => {
        let row = *ctx.get_node(row);
        let col = *ctx.get_node(col);
        let r: i64 = row.eval(ctx).into();
        let c: i64 = col.eval(ctx).into();

        let (_id, cell) = ctx.get_cell([r as usize, c as usize]);
        cell.value
      },

      Addr { row, col } => {
        let row = *ctx.get_node(row);
        let col = *ctx.get_node(col);
        let r: String = row.eval(ctx).into();
        let c: String = col.eval(ctx).into();

        let (_id, cell) = ctx.get_cell([r, c]);
        cell.value
      }

      _ => Val::default(),
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_eval_basics() {
    fn dec(num: i64, scale: u32) -> Decimal {
      Decimal::new(num, scale)
    }

    let (board, tile) = Board::<Cell>::example();

    let mut state = EvalState::new(&board, tile, CellId(0));
    let ast = vec![
      Node::Leaf{value: state.push_value(Val::Num(dec(1, 0)))},
      Node::Leaf{value: state.push_value(Val::Num(dec(2, 0)))},
      Node::Leaf{value: state.push_value(Val::Int(2))},
      Node::Leaf{value: state.push_value(Val::Float(2.0))},
      Node::BinOp{op: '+', lhs: NodeId(0), rhs: NodeId(1)},
      Node::BinOp{op: '+', lhs: NodeId(0), rhs: NodeId(2)},
      Node::BinOp{op: '+', lhs: NodeId(0), rhs: NodeId(3)},
    ];

    state.load(&ast);

    let r1 = ast.get(ast.len()-3).unwrap().eval(&mut state);
    assert_eq!(r1, Val::Num(dec(3, 0)));

    let r2 = ast.get(ast.len()-2).unwrap().eval(&mut state);
    assert_eq!(r2, Val::Num(dec(3, 0)));

    let r3 = ast.get(ast.len()-1).unwrap().eval(&mut state);
    assert_eq!(r3, Val::Num(dec(3, 0)));
  }

  #[test]
  fn test_eval_index() {
    let (board, tile) = Board::<Cell>::example();

    let mut state = EvalState::new(&board, tile, CellId(0));
    let ast = vec![
      Node::Leaf{value: state.push_value(Val::Num(dec!(1)))},
      Node::Leaf{value: state.push_value(Val::Num(dec!(2)))},
      Node::Index{row: NodeId(0), col: NodeId(1)},
    ];

    state.load(&ast);

    let res = ast.get(ast.len()-1).unwrap().eval(&mut state);
    assert_eq!(Val::Bool(true), res);
  }

  #[test]
  fn test_eval_addr() {
    let (board, tile) = Board::<Cell>::example();

    let mut state = EvalState::new(&board, tile, CellId(0));
    let ast = vec![
      Node::Leaf{value: state.push_value(Val::Str("B".to_owned()))},
      Node::Leaf{value: state.push_value(Val::Str("3".to_owned()))},
      Node::Addr{row: NodeId(0), col: NodeId(1)},
    ];

    state.load(&ast);

    let res = ast.get(ast.len()-1).unwrap().eval(&mut state);
    assert_eq!(Val::Bool(true), res);
  }
}
