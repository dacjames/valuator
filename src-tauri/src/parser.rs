use std::{collections::HashMap};

use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::cell::Value;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Token(u64);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(u32);

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

#[derive(Clone, Debug, PartialEq, PartialOrd)]
enum Node {
    Leaf{leaf: Token, value: Value},
    OpBin{op: Token, lhs: NodeId, rhs: NodeId},
    OpUni{op: Token, rhs: NodeId},
}
use Node::*;

impl Node {
  fn eval(&self, ctx: &impl EvalContext) -> Value {
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

struct Parser {
  tokens: Vec<Token>,
  nodes: Vec<Node>,

  bytes: Vec<char>,
  pos: usize,
}

impl Parser {
  fn new<S: Into<String>>(input: S) -> Parser {
    Parser { 
      tokens: vec![], 
      nodes: vec![], 
      bytes: input.into().chars().collect(), 
      pos: 0,
    }
  }
}

impl ParserOps<char> for Parser {
  fn get_pos(&self) -> usize {
      self.pos
  }
  fn set_pos(&mut self, p: usize) {
      self.pos = p;
  }
  fn next(&mut self) -> Option<char> {
    let item = self.bytes.get(self.pos)?;
    self.pos += 1;
    Some(*item)
  }

  fn like(&mut self, needle: char) -> Option<char> {
    let mut item = self.next()?;
    while item == ' ' || item == '\n' || item == '\t' {
      item = self.next()?;
    }
    if item == needle { 
      Some(needle) 
    } else {
      None
    }
  }
  fn exact(&mut self, needle: char) -> Option<char> {
    let item = self.next()?;
    if item == needle { 
      Some(needle) 
    } else {
      None
    }
  }
} 

trait ParserOps<T> {
  fn get_pos(&self) -> usize;
  fn set_pos(&mut self, p: usize);
  fn next(&mut self) -> Option<T>;
  fn like(&mut self, needle: char) -> Option<T>;
  fn exact(&mut self, needle: char) -> Option<T>;

  fn r_one(&mut self) -> Option<T> {
    self.like('1')
  }
  fn r_plus(&mut self) -> Option<T> {
    self.like('+')
  }
  fn r_minus(&mut self) -> Option<T> {
    self.like('-')
  }
  fn r_mult(&mut self) -> Option<T> {
    self.like('*')
  }
  fn r_div(&mut self) -> Option<T> {
    self.like('/')
  }
  fn r_lpar(&mut self) -> Option<T> {
    self.like('(')
  }
  fn r_rpar(&mut self) -> Option<T> {
    self.like(')')
  }

  fn r_term1(&mut self) -> Option<T> {
    self.r_one()
  }

  fn r_term2(&mut self) -> Option<T> {
    self.r_lpar()?;
    let expr = self.r_expr()?;
    self.r_rpar()?;
    Some(expr)
  }

  fn r_term(&mut self) -> Option<T> {
    let pos = self.get_pos();

    for expr in [
      |s: &mut Self|{s.r_term1()},
      |s: &mut Self|{s.r_term2()},
    ] {
      match expr(self) {
        Some(e) => return Some(e),
        None => self.set_pos(pos),
      };
    }

    None
  }

  fn r_binop(&mut self) -> Option<T> {
    let pos = self.get_pos();

    for expr in [
      |s: &mut Self|{s.r_plus()},
      |s: &mut Self|{s.r_minus()},
      |s: &mut Self|{s.r_mult()},
      |s: &mut Self|{s.r_div()},
    ] {
      match expr(self) {
        Some(e) => return Some(e),
        None => self.set_pos(pos),
      };
    }

    None
  }

  fn r_expr1(&mut self) -> Option<T> {
    self.r_term()?;
    let op = self.r_binop()?;
    self.r_expr()?;
    Some(op)
  }

  fn r_expr2(&mut self) -> Option<T> {
    self.r_term()
  }

  fn r_expr(&mut self) -> Option<T>  {
    let pos = self.get_pos();

    for expr in [
      |s: &mut Self|{s.r_expr1()},
      |s: &mut Self|{s.r_expr2()},
    ] {
      match expr(self) {
        Some(e) => return Some(e),
        None => self.set_pos(pos),
      };
    }
    None

  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parser_basics() {
    let mut p = Parser::new("hi");

    assert_eq!(p.next(), Some('h'));
    assert_eq!(p.next(), Some('i'));
    assert_eq!(p.next(), None);
    assert_eq!(p.next(), None);

    p = Parser::new("1  ");
    assert_eq!(p.r_expr(), Some('1'));

    p = Parser::new("1+1");
    assert_eq!(p.r_expr(), Some('+'));

    p = Parser::new("1-1");
    assert_eq!(p.r_expr(), Some('-'));

    p = Parser::new("1/1");
    assert_eq!(p.r_expr(), Some('/'));

    p = Parser::new("1-1");
    assert_eq!(p.r_expr(), Some('-'));

    p = Parser::new("(1)");
    assert_eq!(p.r_expr(), Some('1'));

    p = Parser::new("(1 +(1+ 1+1)+ 1)");
    assert_eq!(p.r_expr(), Some('+'));

    p = Parser::new("1 +1");
    assert_eq!(p.exact('1'), Some('1'));
    assert_eq!(p.exact('1'), None);
  }

  #[test]
  fn test_eval_basics() {
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