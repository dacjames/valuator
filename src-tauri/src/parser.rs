use std::collections::HashMap;

use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::cell::Value;



#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Token(u64);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Node(u32);

#[derive(Clone, Debug, PartialEq, PartialOrd)]
enum Ast {
    Leaf{leaf: Token, value: Value},
    OpBin{op: Token, lhs: Node, rhs: Node},
    OpUni{op: Token, rhs: Node},
}


use Ast::*;

struct Formula {
    bytes: String,
    tokens: Vec<Token>,
    nodes: Vec<Ast>,
}

trait EvalContext {
  fn get_ast(&self, node: &Node) -> &Ast;
}

struct EvalState {
  nodes: HashMap<Node, Ast>
}

impl EvalState {
  fn new() -> EvalState {
    EvalState{
      nodes: HashMap::new(),
    }
  }
  fn insert(&mut self, node: Node, ast: Ast) {
    self.nodes.insert(node, ast);
  }
}

impl EvalContext for EvalState {
  fn get_ast(&self, node: &Node) -> &Ast {
    self.nodes.get(node).unwrap()
  }
}

impl Ast {
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
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_parser_basics() {
    use Ast::*;
    use Value::*;

    let mut state = EvalState::new();
    let tree = vec![
      Leaf{leaf: Token(0), value: N(Decimal::new(1, 0))},
      Leaf{leaf: Token(0), value: N(Decimal::new(2, 0))},
      Leaf{leaf: Token(0), value: I(2 as i64)},
      Leaf{leaf: Token(0), value: F(2.0_f64)},
      OpBin{op: Token(0), lhs: Node(0), rhs: Node(1)},
      OpBin{op: Token(0), lhs: Node(0), rhs: Node(2)},
      OpBin{op: Token(0), lhs: Node(0), rhs: Node(3)},
    ];

    for (i, ast) in tree.iter().enumerate() {
      state.insert(Node(i as u32), ast.to_owned())
    }

    let r1 = tree.get(tree.len()-3).unwrap().eval(&state);
    assert_eq!(r1, N(Decimal::new(3, 0)));

    let r2 = tree.get(tree.len()-2).unwrap().eval(&state);
    assert_eq!(r2, N(Decimal::new(3, 0)));
    
    let r3 = tree.get(tree.len()-1).unwrap().eval(&state);
    assert_eq!(r3, N(Decimal::new(3, 0)));
  }
}