use std::collections::HashMap;

use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::cell::Value;



#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Token(u64);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(u32);

#[derive(Clone, Debug, PartialEq, PartialOrd)]
enum Node {
    Leaf{leaf: Token, value: Value},
    OpBin{op: Token, lhs: NodeId, rhs: NodeId},
    OpUni{op: Token, rhs: NodeId},
}


use Node::*;

struct Formula {
    bytes: String,
    tokens: Vec<Token>,
    nodes: Vec<Node>,
}

trait EvalContext {
  fn get_ast(&self, node: &NodeId) -> &Node;
}

struct EvalState {
  nodes: HashMap<NodeId, Node>
}

impl EvalState {
  fn new() -> EvalState {
    EvalState{
      nodes: HashMap::new(),
    }
  }

  fn insert(&mut self, node: NodeId, ast: Node) {
    self.nodes.insert(node, ast);
  }

  fn load(&mut self, tree: &Vec<Node>) {
    for (i, ast) in tree.iter().enumerate() {
      self.insert(NodeId(i as u32), ast.to_owned())
    }
  }
}

impl EvalContext for EvalState {
  fn get_ast(&self, node: &NodeId) -> &Node {
    self.nodes.get(node).unwrap()
  }
}

impl Node {
  pub fn eval(&self, ctx: &impl EvalContext) -> Value {
    match self {
      Leaf{leaf, value} => value.to_owned(),
      OpBin{op, lhs, rhs} => {
        let left = ctx.get_ast(lhs).eval(ctx);
        let right = ctx.get_ast(rhs).eval(ctx);

        use Value::*;

        match (left, right) {
          (N(l), N(r)) => N(l + r),
          (N(l), I(r)) => N(l + Decimal::from(r)),
          (I(l), N(r)) => N(Decimal::from(l) + r),
          (N(l), F(r)) => N(l + Decimal::from_f64(r).unwrap()),
          (F(l), N(r)) => N(Decimal::from_f64(l).unwrap() + r),
          _ => Value::N(Decimal::from(0)),
        }
      },
      _ => Value::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parser_basics() {
    use Node::*;
    use Value::*;

    fn dec(num: i64, scale: u32) -> Decimal {
      Decimal::new(num, scale)
    }

    let mut state = EvalState::new();
    let ast = vec![
      Leaf{leaf: Token(0), value: N(dec(1, 0))},
      Leaf{leaf: Token(0), value: N(dec(2, 0))},
      Leaf{leaf: Token(0), value: I(2)},
      Leaf{leaf: Token(0), value: F(2.0)},
      OpBin{op: Token(0), lhs: NodeId(0), rhs: NodeId(1)},
      OpBin{op: Token(0), lhs: NodeId(0), rhs: NodeId(2)},
      OpBin{op: Token(0), lhs: NodeId(0), rhs: NodeId(3)},
    ];

    state.load(&ast);    

    let r1 = ast.get(ast.len()-3).unwrap().eval(&state);
    assert_eq!(r1, N(dec(3, 0)));

    let r2 = ast.get(ast.len()-2).unwrap().eval(&state);
    assert_eq!(r2, N(dec(3, 0)));
    
    let r3 = ast.get(ast.len()-1).unwrap().eval(&state);
    assert_eq!(r3, N(dec(3, 0)));
  }
}