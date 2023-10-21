use std::collections::HashMap;

use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::cell::Value;
use scopeguard::defer;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Token {
  pos: u32,
  len: u16,
  tag: u16,
}

impl Token {
  fn new(tag: u16, orig: usize, curr: usize) -> Token {
    if curr < orig {
      panic!("Negative token length!")
    }
    Token { 
      pos: orig as u32, 
      len: (curr - orig) as u16,
      tag: tag,
    }
  }
}

trait TokenSaver {
  fn save_token(&mut self, tok: Token);
} 

struct TokenContext {
  token: Token,
  saver: dyn TokenSaver
}

impl Drop for TokenContext {
  fn drop(&mut self) {
    self.saver.save_token(self.token);
  }
}

// struct Token(u64);


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

type Rule = fn(&mut Parser) -> Option<char>;

struct Parser {
  tokens: Vec<Token>,
  nodes: Vec<Node>,

  bytes: Vec<char>,
  pos: usize,
}

impl TokenSaver for Parser {
  fn save_token(&mut self, tok: Token) { 
      self.tokens.push(tok);
  }
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

  // fn start(&mut self, tag: u16) -> TokenContext {
  //   let tok = Token::new(tag, self.pos, self.pos);
  //   TokenContext { token: tok, saver: self as (&mut TokenSaver) }
  // }

  fn next_nonws(&mut self) -> Option<char> {
    let mut item = self.next()?;
    while item.is_whitespace() {
      item = self.next()?;
    }
    Some(item)
  }

  fn whitespace(&mut self) -> Option<char> {
    fn is_some_whitespace(item: Option<char>) -> bool {
      item.is_some() && item.unwrap().is_whitespace()
    }
    let mut item = self.next();
    let first = item;
    let mut matched = false;
    while is_some_whitespace(item) {
      matched = true;
      item = self.next();
    }
    if matched {
      self.pos -= 1;
      return first;
    }
    None
  }

  fn like_char(&mut self, needle: char) -> Option<char> {
    let item = self.next_nonws()?;
    if item == needle { 
      Some(needle) 
    } else {
      None
    }
  }

  fn like_str<S: Into<String>>(&mut self, needle: S) -> Option<char> {
    let needle_string: String = needle.into();
    let mut iter = needle_string.chars();
    let mut res = self.like_char(iter.next()?)?;
    for ch in iter {
      res = self.like_char(ch)?;
    }
    Some(res)
  }

  fn exact(&mut self, needle: char) -> Option<char> {
    let item = self.next()?;
    if item == needle { 
      Some(needle) 
    } else {
      None
    }
  }

  fn select<const N: usize>(&mut self, rules: [Rule; N]) -> Option<char> {
    let pos = self.get_pos();

    for rule in rules {
      match rule(self) {
        Some(e) => return Some(e),
        None => self.set_pos(pos),
      };
    }
    None
  }

  fn char_class(&mut self, chars: &str) -> Option<char> {
    let item = self.next_nonws()?;
    if chars.contains(item) {
      Some(item)
    } else {
      None
    }
  }

  fn maybe(&mut self, rule: Rule) -> Option<char> {
    let pos = self.get_pos();
    match rule(self) {
      Some(ch) => Some(ch),
      None => {
        self.set_pos(pos);
        Some('\0')
      },
    }
  }

  fn one_or_more(&mut self, rule: Rule) -> Option<char> {
    let res = rule(self)?;
    self.zero_or_more(rule)?;
    Some(res)
  }

  fn zero_or_more(&mut self, rule: Rule) -> Option<char> {
    let mut pos = self.get_pos();
    let mut res = rule(self);
    let mut last: Option<char> = Some('\0');

    while res.is_some() {
      pos = self.get_pos();
      last = res;
      res = rule(self);
    }

    // rollback the None match at the end of the sequence
    self.set_pos(pos);
    last
  }

  fn r_one(&mut self) -> Option<char> {
    self.maybe(|s|{s.like_char('e')})?;
    self.one_or_more(|s|{s.char_class("1")})
  }

  fn r_ten(&mut self) -> Option<char> {
    self.like_str("10")
  }

  fn r_num(&mut self) -> Option<char> {
    self.one_or_more(|s|{s.char_class("0123456789")})
  }
  fn r_plus(&mut self) -> Option<char> {
    self.like_char('+')
  }
  fn r_minus(&mut self) -> Option<char> {
    self.like_char('-')
  }
  fn r_mult(&mut self) -> Option<char> {
    self.like_char('*')
  }
  fn r_div(&mut self) -> Option<char> {
    self.like_char('/')
  }
  fn r_lpar(&mut self) -> Option<char> {
    self.like_char('(')
  }
  fn r_rpar(&mut self) -> Option<char> {
    self.like_char(')')
  }

  fn r_term1(&mut self) -> Option<char> {
    self.r_ten()
  }

  fn r_term2(&mut self) -> Option<char> {
    self.select([
      |s|{s.r_one()},
      |s|{s.r_num()},
    ])
  }

  fn r_term3(&mut self) -> Option<char> {
    self.r_lpar()?;
    let expr = self.r_expr()?;
    self.r_rpar()?;
    Some(expr)
  }

  fn r_term(&mut self) -> Option<char> {
    self.select([
      |s|{s.r_term1()},
      |s|{s.r_term2()},
      |s|{s.r_term3()},
    ])
  }

  fn r_binop(&mut self) -> Option<char> {
    self.select([
      |s|{s.r_plus()},
      |s|{s.r_minus()},
      |s|{s.r_mult()},
      |s|{s.r_div()},
    ])
  }

  fn r_expr1(&mut self) -> Option<char> {
    self.r_term()?;
    let op = self.r_binop()?;
    self.r_expr()?;
    Some(op)
  }

  fn r_expr2(&mut self) -> Option<char> {
    self.r_term()
  }

  fn r_expr(&mut self) -> Option<char>  {
    self.select([
      |s: &mut Self|{s.r_expr1()},
      |s: &mut Self|{s.r_expr2()},
    ])
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


    p = Parser::new("    x     y");
    assert_eq!(p.whitespace(), Some(' '));
    assert_eq!(p.next(), Some('x'));
    assert_eq!(p.whitespace(), Some(' '));
    assert_eq!(p.whitespace(), None);

    p = Parser::new("e1");
    assert_eq!(p.r_expr(), Some('1'));

    p = Parser::new("111111");
    assert_eq!(p.r_expr(), Some('1'));

    p = Parser::new("10");
    assert_eq!(p.r_expr(), Some('0'));

    p = Parser::new("999");
    assert_eq!(p.r_expr(), Some('9'));

    p = Parser::new("399+84729");
    assert_eq!(p.r_expr(), Some('+'));

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

    p = Parser::new("(1+(1 +1 +1) +1)");
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
      Leaf{leaf: Token::new(0,0,0), value: N(dec(1, 0))},
      Leaf{leaf: Token::new(0,0,0), value: N(dec(2, 0))},
      Leaf{leaf: Token::new(0,0,0), value: I(2)},
      Leaf{leaf: Token::new(0,0,0), value: F(2.0)},
      OpBin{op: Token::new(0,0,0), lhs: NodeId(0), rhs: NodeId(1)},
      OpBin{op: Token::new(0,0,0), lhs: NodeId(0), rhs: NodeId(2)},
      OpBin{op: Token::new(0,0,0), lhs: NodeId(0), rhs: NodeId(3)},
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