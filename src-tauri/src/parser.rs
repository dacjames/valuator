use std::any::Any;
use std::hash::Hash;
use std::{collections::HashMap};
use std::convert::TryInto;

use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::cell::Value;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
enum TokTag {
  NumTok,
  OpTok,
  SymTok,
  KWTok,
  WSTok,
  StringTok,
  LParTok, RParTok,
  LBckTok, RBckTok,
  LBrcTok, RBrcTok,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
  pos: u32,
  len: u16,
  tag: TokTag,
}

impl Token {
  fn new(tag: TokTag, pos: u32, len: u16) -> Token {
    Token{
      pos: pos,
      len: len,
      tag: tag,
    }
  }
  fn empty(tag: TokTag, pos: u32) -> Token {
    Token::new(tag, pos, 0)
  }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(u32);
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueId(u32);

impl Default for NodeId {
  fn default() -> Self {
    NodeId(0)
  }
}

pub trait EvalContext {
  fn get_value(&self, node: &ValueId) -> &Value;
  fn get_ast(&self, node: &NodeId) -> &Node;
}

struct EvalState {
  nodes: HashMap<NodeId, Node>,
  values: HashMap<ValueId, Value>,
}

impl EvalState {
  fn new() -> EvalState {
    EvalState{
      nodes: HashMap::new(),
      values: HashMap::new(),
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

  fn push_value(&mut self, value: Value) -> ValueId{
    let id = ValueId(self.values.len() as u32);
    self.values.insert(id, value);
    id
  }
}

impl EvalContext for EvalState {
  fn get_value(&self, value: &ValueId) -> &Value {
    self.values.get(value).unwrap()
  }
  fn get_ast(&self, node: &NodeId) -> &Node {
    self.nodes.get(node).unwrap()
  }
}

impl EvalContext for Parser {
  fn get_value(&self, node: &ValueId) -> &Value {
    &self.values[node.0 as usize]
  }
  fn get_ast(&self, node: &NodeId) -> &Node {
    &self.nodes[node.0 as usize]
  }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Node {
  Zero{},
  Nil{ch: char},
  Symbol{tok: Token},
  Leaf{tok: Token, value: ValueId},
  BinOp{op: char, lhs: NodeId, rhs: NodeId},
  UniOp{op: char, rhs: NodeId},
  Index{row: NodeId, col: NodeId},
  Addr{row: NodeId, col: NodeId},
  List{elems: [NodeId; 8], len: usize, link: Option<NodeId>},
}
use Node::*;

impl Default for Node {
  fn default() -> Self {
    Zero { }
  }
}

impl Node {
  pub fn eval(&self, ctx: &impl EvalContext) -> Value {
    match self {
      Leaf{tok: _, value} => ctx.get_value(value).to_owned(),
      BinOp{op, lhs, rhs} => {
        let left = ctx.get_ast(lhs).eval(ctx);
        let right = ctx.get_ast(rhs).eval(ctx);

        use Value::*;

        let f: fn(Decimal, Decimal) -> Decimal = match *op {
          '+' => |l,r|l + r,
          '-' => |l,r|l - r,
          '/' => |l,r|l / r,
          '*' => |l,r|l * r,
          _ => |l, r|Decimal::new(0, 0),
        };

        match (left, right) {
          (N(l), N(r)) => N(f(l,r)),
          (N(l), I(r)) => N(f(l, Decimal::from(r))),
          (I(l), N(r)) => N(f(Decimal::from(l), r)),
          (N(l), F(r)) => N(f(l, Decimal::from_f64(r).unwrap())),
          (F(l), N(r)) => N(f(Decimal::from_f64(l).unwrap(), r)),
          _ => Value::N(Decimal::from(0)),
        }
      },
      List { elems, len, link } => {
        if link.is_some() {
          panic!("list linking not impl");
        }
        let vals: Vec<Value> = elems.iter().take(*len).map(|nid|{
          let node = ctx.get_ast(nid);
          node.eval(ctx)
        }).collect();
        Value::L(vals)
      }
      _ => Value::default(),
    }
  }
}

type Rule<T> = fn(&mut Parser) -> Option<T>;
// type Rule = impl Fn(&mut Parser) -> Option<char>;


struct TokCtx{tok: Token}

impl TokCtx {
  fn end(&self, pos: u32) -> Token {
    let mut tok = self.tok;
    if pos < self.tok.pos {
      let this = self.tok.pos;
      panic!("invalid end {pos} < {this}")
    }
    tok.len = (pos - self.tok.pos) as u16;
    return tok
  }
}

#[derive(Debug, Clone, Copy)]
struct ParseState {
  pos: usize,
  len_toks: usize,
  len_nodes: usize,
}


pub struct Parser {
  tokens: Vec<Token>,
  nodes: Vec<Node>,
  values: Vec<Value>,

  memos: HashMap<&'static str, Box<dyn Any>>,

  buf: Vec<char>,
  pos: usize,
}

impl Parser {
  pub fn new<S: Into<String>>(input: S) -> Parser {
    Parser { 
      tokens: vec![], 
      nodes: vec![Node::default()], 
      values: vec![Value::default()],
      memos: HashMap::new(),
      buf: input.into().chars().collect(), 
      pos: 0,
    }
  }

  fn tok_ctx(&self, tag: TokTag) -> TokCtx {
    TokCtx{ tok: Token::empty(tag, self.pos as u32) }
  }

  fn push_tok<T: Copy + Default>(&mut self, tag: TokTag, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<T> {
    let tokctx = self.tok_ctx(tag);
    let res = rule(self);
    match res {
      Some(_) => {
        self.tokens.push(tokctx.end(self.pos as u32));
        res
      }
      None => res,
    }
  } 

  fn push_node(&mut self, node: Node) -> NodeId {
    let id = self.nodes.len() as u32;
    self.nodes.push(node);
    NodeId(id)
  }

  fn push_value(&mut self, value: Value) -> ValueId {
    let id = self.values.len() as u32;
    self.values.push(value);
    ValueId(id)
  }

  fn yield_tok<T: Copy + Default>(&mut self, tag: TokTag, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<Token> {
    let tokctx = self.tok_ctx(tag);
    let res = rule(self);
    match res {
      Some(_) => {
        let tok = tokctx.end(self.pos as u32);
        self.tokens.push(tok);
        Some(tok)
      }
      None => None,
    }
  } 

  fn tok_value(&self, tok: Token) -> String {
    let p = tok.pos as usize;
    let len = tok.len as usize;
    let val: String = self.buf[p..p+len].iter().collect();
    val
  }

  fn tok_values(&self) -> Vec<String> {
    self.tokens.iter().map(|t|{ self.tok_value(*t) }).collect()
  }

  fn get_pos(&self) -> usize {
      self.pos
  }

  fn peak_tok(&self) -> Token {
    self.tokens[self.tokens.len()-1]
  }

  fn get_char(&self, tok: Token) -> char {
    self.buf[tok.pos as usize]
  }

  fn set_pos(&mut self, p: usize) {
      self.pos = p;
  }

  fn next(&mut self) -> Option<char> {
    let item = self.buf.get(self.pos)?;
    self.pos += 1;
    Some(*item)
  }


  fn reset(&mut self) {
    self.set_pos(0);
    self.tokens.truncate(0);
    self.nodes.truncate(1);
  }

  fn save(&self) -> ParseState {
    ParseState{
      pos: self.get_pos(),
      len_toks: self.tokens.len(),
      len_nodes: self.nodes.len(),
    }
  }

  fn rollback(&mut self, state: ParseState) {
    self.set_pos(state.pos);
    self.tokens.truncate(state.len_toks);
    self.nodes.truncate(state.len_nodes);
  }

  fn match_ws(&mut self) -> Option<char> {
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
      if self.pos != self.buf.len() {
        self.pos -= 1;
      }
      return first;
    }
    None
  }

  fn ws(&mut self) -> Option<char> {
    self.push_tok(TokTag::WSTok, |s|{s.match_ws()})
  }

  fn maybe_ws(&mut self) -> Option<char> {
    self.maybe(|s|{s.ws()})
  }

  fn char(&mut self, needle: char) -> Option<char> {
    let item = self.next()?;
    if item == needle { 
      Some(needle) 
    } else {
      None
    }
  }

  fn not_char(&mut self, needle: char) -> Option<char> {
    let item = self.next()?;
    if item != needle { 
      Some(needle) 
    } else {
      None
    }
  }

  fn string<S: Into<String>>(&mut self, needle: S) -> Option<char> {
    let needle_string: String = needle.into();
    let mut iter = needle_string.chars();
    let mut res = self.char(iter.next()?)?;
    for ch in iter {
      res = self.char(ch)?;
    }
    Some(res)
  }

  fn char_class(&mut self, chars: &str) -> Option<char> {
    let item = self.next()?;
    if chars.contains(item) {
      Some(item)
    } else {
      None
    }
  }

  fn nocase_class(&mut self, chars: &str) -> Option<char> {
    let item = self.next()?;
    if chars.contains(item) || chars.contains(item.to_lowercase().next().unwrap()) {
      Some(item)
    } else {
      None
    }
  }

  fn not_class(&mut self, chars: &str) -> Option<char> {
    let item = self.next()?;
    if !chars.contains(item) {
      Some(item)
    } else {
      None
    }
  }

  fn select<const N: usize, T: Clone + Default>(&mut self, rules: [Rule<T>; N]) -> Option<T> {
    let state = self.save();

    for rule in rules {
      match rule(self) {
        Some(e) => return Some(e),
        None => self.rollback(state),
      };
    }
    None
  }

  fn maybe<T: Copy + Default>(&mut self, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<T> {
    let state = self.save();
    match rule(self) {
      Some(ch) => Some(ch),
      None => {
        // TODO keep an eye on maybe performance
        // currently more efficient to waste node/token memory than to rollback.
        self.set_pos(state.pos);
        Some(T::default())
      },
    }
  }

  fn one_or_more<T: Copy + Default>(&mut self, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<T> {
    let res = rule(self)?;
    self.zero_or_more(rule)?;
    Some(res)
  }

  fn zero_or_more<T: Copy + Default>(&mut self, mut rule: impl FnMut(&mut Parser) -> Option<T>) -> Option<T> {
    let mut state = self.save();
    let mut res = rule(self);
    let mut last: Option<T> = Some(T::default());

    while res.is_some() {
      state = self.save();
      last = res;
      res = rule(self);
    }

    // rollback the None match at the end of the sequence
    self.rollback(state);
    last
  }

  // saved_result = None
  // def oracle_expr():
  //   if saved_result is None:
  //       return False
  //   return saved_result
  fn oracle<T: Copy + Default + 'static>(&self, name: &'static str, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<T> {
    let saved = self.memos.get(name)?;
    let res = *saved.downcast_ref::<Option<T>>()?;
    res
  }

  // def expr_wrapper():
  //   global saved_result
  //   saved_result = None
  //   parsed_length = 0
  //   while True:
  //       new_result = expr()
  //       if not new_result:
  //           break
  //       new_parsed_length = <calculate size of new_result>
  //       if new_parsed_length <= parsed_length:
  //           break
  //       saved_result = new_result
  //       parsed_length = new_parsed_length
  //   return saved_result
  fn left<T: Copy + Default + 'static>(&mut self, name: &'static str, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<T> {
    let mut saved: Option<T> = None;
    let state = self.save();
    let mut parsed = state.pos - self.pos;

    loop {
      let res = rule(self);
      if res.is_none() {
        break;
      }
      let new_parsed = self.pos - state.pos;
      if new_parsed <= parsed {
        break;
      }
      saved = res;
      self.memos.insert(name, Box::new(saved));
      parsed = new_parsed;
    }
    saved
  }

  fn r_num(&mut self) -> Option<Node> {

    self.yield_tok(TokTag::NumTok, |s| {
      s.char_class("123456789")?;
      s.zero_or_more(|s|s.char_class("0123456789"))?;
      s.maybe(|s|s.char('.'))?;
      s.zero_or_more(|s|s.char_class("0123456789"))
    }).and_then(|tok|{
      let decval = Decimal::from_str_radix(&self.tok_value(tok), 10).unwrap_or(Decimal::default());
      Some(Node::Leaf { tok, value: self.push_value(Value::N(decval)) })
    })
  }

  fn match_string(&mut self, bookend: char) -> Option<char> {
    self.char(bookend)?;
    self.zero_or_more(move |s|{s.not_char(bookend)})?;
    self.char(bookend)?;
    Some(bookend)
  }

  fn r_string(&mut self) -> Option<Node> {
    self.yield_tok(TokTag::StringTok, |s| {
      s.select([
        |s|{s.match_string('\'')},
        |s|{s.match_string('"')},
      ])
    }).and_then(|tok|{
      let pos = tok.pos as usize;
      let end = tok.len as usize + pos;
      let body: String = self.buf[pos+1..end-1].iter().collect();
      Some(Leaf{ tok: tok, value: self.push_value(Value::S(body)) })
    })
  }

  fn match_plus(&mut self) -> Option<char> {
    self.char('+')
  }
  fn match_minus(&mut self) -> Option<char> {
    self.char('-')
  }
  fn match_mult(&mut self) -> Option<char> {
    self.char('*')
  }
  fn match_div(&mut self) -> Option<char> {
    self.char('/')
  }
  
  fn r_term_literal(&mut self) -> Option<Node> {
    self.select([
      |s|{s.r_num()},
      |s|{s.r_string()},
    ])
  }

  fn match_lpar(&mut self) -> Option<char> {
    self.push_tok(TokTag::LParTok,|s|s.char('('))
  }
  fn match_rpar(&mut self) -> Option<char> {
    self.push_tok(TokTag::RParTok, |s|s.char(')'))
  }

  fn r_term_paren(&mut self) -> Option<Node> {
    self.match_lpar()?;
    let expr = self.r_expr()?;
    self.match_rpar()?;
    Some(expr)
  }

  fn r_term(&mut self) -> Option<Node> {
    self.select([
      |s|s.r_term_literal(),
      |s|s.r_term_paren(),
    ])
  }

  fn match_binop(&mut self) -> Option<char> {
    self.push_tok(TokTag::OpTok, |s|{
      s.select([
        |s|s.match_plus(),
        |s|s.match_minus(),
        |s|s.match_mult(),
        |s|s.match_div(),
      ])
    })
  }

  fn r_expr_binop(&mut self) -> Option<Node> {
    let lnode = self.r_term()?;
    let left = self.push_node(lnode);

    self.maybe_ws()?;
    let op = self.match_binop()?;
    self.maybe_ws()?;
    let rnode = self.r_expr()?;
    let right = self.push_node(rnode);
    Some(BinOp { op: op, lhs: left, rhs: right })
  }

  fn r_expr_list2(&mut self) -> Option<Node> {
    let lnode = self.r_term()?;
    let first = self.push_node(lnode);

    let mut elems = vec![first];

    self.maybe_ws()?;
    self.char(',')?;
    self.zero_or_more(|s|{
      s.maybe_ws()?;
      s.maybe(|s|s.char(','))?;
      let node = s.r_expr()?;
      let nid = s.push_node(node);
      elems.push(nid);
      Some(node)
    })?;

    if elems.len() > 8 {
      panic!("linked list not impl");
    }

    let len = elems.len();
    elems.extend(vec![NodeId(0); 8 - len]);
    let elems_array: [NodeId; 8] =  elems.try_into().unwrap();
    Some(List { elems: elems_array, len: len, link: None })
  }

  fn r_expr_list(&mut self) -> Option<Node> {
    let lnode = self.oracle("expr", |s|s.r_expr())?;
    // let lnode = self.r_term()?;
    let first = self.push_node(lnode);

    let mut elems = vec![first];

    self.maybe_ws()?;
    self.char(',')?;
    self.zero_or_more(|s|{
      s.maybe_ws()?;
      s.maybe(|s|s.char(','))?;
      let node = s.r_term()?;
      let nid = s.push_node(node);
      elems.push(nid);
      Some(node)
    })?;

    if elems.len() > 8 {
      panic!("linked list not impl");
    }

    let len = elems.len();
    elems.extend(vec![NodeId(0); 8 - len]);
    let elems_array: [NodeId; 8] =  elems.try_into().unwrap();
    Some(List { elems: elems_array, len: len, link: None })
  }

  fn r_sym(&mut self) -> Option<Node> {
    self.yield_tok(TokTag::SymTok, |s|{
      s.one_or_more(|s|{ s.nocase_class("abcdefghijklmnopqrstuvwxyz") })
    }).and_then(|tok|{
      Some(Symbol{ tok: tok })
    })

  }

  // fn r_expr_assign(&mut self) -> Option<Node> {
  //   self.push_tok(TokTag::KWTok, |s|{
  //     s.select([
  //       |s|{s.string("val")},
  //       |s|{s.string("var")},
  //     ])
  //   })?;
  //   self.ws()?;
  //   self.r_sym()?;
  //   self.maybe_ws()?;
  //   let op = self.char('=')?;
  //   self.maybe_ws()?;
  //   self.r_expr()?;
  //   // Some(op)
  //   Some(Nil{ch: op})
  // }

  fn match_compound(&mut self, start: (char, TokTag), end: (char, TokTag), cb: impl Fn(NodeId, NodeId) -> Node) -> Option<Node> {
    self.push_tok(start.1, |s|s.char(start.0))?;
    self.maybe_ws()?;
    let row_expr = self.r_expr()?;
    let row = self.push_node(row_expr);
    self.maybe_ws()?;
    let col = self.maybe(|s|{
      s.char(',')?;
      let col_expr = s.r_expr()?;
      let col = s.push_node(col_expr);
      s.maybe_ws()?;
      Some(col)
    }).unwrap_or(NodeId(0));
    self.push_tok(end.1, |s|s.char(end.0))?;

    Some(cb(row, col))
  }

  fn r_expr_index(&mut self) -> Option<Node> {
    self.match_compound(('[', TokTag::LBckTok), (']', TokTag::RBckTok), |r, c| {
      Index { row: r, col: c}
    })
  }

  fn r_expr_addr(&mut self) -> Option<Node> {
    self.match_compound(('{', TokTag::LBrcTok), ('}',  TokTag::RBrcTok), |r, c| {
      Addr { row: r, col: c}
    })
  }

  fn r_expr_lookup(&mut self) -> Option<Node> {
    self.r_sym()
  }

  fn r_expr_inner(&mut self) -> Option<Node>  {
    self.maybe_ws()?;
    let res = self.select([
      |s| s.r_expr_binop(),
      |s| s.r_expr_list(),
      |s| s.r_term(),
      // |s| s.r_expr_assign(),
      |s| s.r_expr_lookup(),
      |s| s.r_expr_index(),
      |s| s.r_expr_addr(),
    ])?;
    self.maybe_ws()?;
    Some(res)
  }

  fn r_expr(&mut self) -> Option<Node> {
    self.left("expr", |s|s.r_expr_inner())
  }

  fn r_expr2(&mut self) -> Option<Node> {
    self.r_expr_inner()
  }

  pub fn scan(&mut self) -> Vec<String> {
    match self.r_expr() {
      Some(_) => self.tok_values(),
      None => vec![],
    }
  }

  pub fn reparse(&mut self) -> Option<Node> {
    self.reset();
    self.r_expr()
  }

  pub fn parse(&mut self) -> Option<Node> {
    self.r_expr()
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! vec_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
  }

  #[test]
  fn test_parser_basics() {
    use Node::*;
    let mut p = Parser::new("hi");

    assert_eq!(p.next(), Some('h'));
    assert_eq!(p.next(), Some('i'));
    assert_eq!(p.next(), None);
    assert_eq!(p.next(), None);

    p = Parser::new("1  ");
    assert_eq!(p.scan(), vec_strings!["1", "  "]);

    p = Parser::new("    x     y");
    assert_eq!(p.ws(), Some(' '));
    assert_eq!(p.next(), Some('x'));
    assert_eq!(p.ws(), Some(' '));
    assert_eq!(p.ws(), None);

    p = Parser::new("111111");
    assert_eq!(p.scan(), vec_strings!["111111"]);

    p = Parser::new("10");
    assert_eq!(p.scan(), vec_strings!["10"]);

    p = Parser::new("999");
    assert_eq!(p.scan(), vec_strings!["999"]);

    p = Parser::new("399+84729");
    assert_eq!(p.scan(), vec_strings!["399","+","84729"]);

    p = Parser::new("1+1");
    assert_eq!(p.scan(), vec_strings!["1","+","1"]);

    p = Parser::new("1-1");
    assert_eq!(p.scan(), vec_strings!["1","-","1"]);

    p = Parser::new("1/1");
    assert_eq!(p.scan(), vec_strings!["1","/","1"]);

    p = Parser::new("1*1");
    assert_eq!(p.scan(), vec_strings!["1","*","1"]);

    p = Parser::new("(1)");
    assert_eq!(p.scan(), vec_strings!["(","1",")"]);

    p = Parser::new("(1+(1 +1 +1) +1)");
    assert_eq!(p.scan().len(), 16);
  }

  #[test]
  fn test_parser_tokens() {
    // let mut p = Parser::new("999");
    let mut p = Parser::new("789+234");
    assert!(p.parse().is_some());
    assert_eq!(p.tokens.len(), 3);
    assert_eq!(p.tok_values(), vec_strings!["789","+","234"]);
  }

  #[test]
  fn test_parser_strings() {
    // let mut p = Parser::new("999");
    let mut p = Parser::new("'asdf'");
    assert!(p.parse().is_some());
    assert_eq!(p.tokens.len(), 1);
    assert_eq!(p.tok_values(), vec_strings!["'asdf'"]);

    let mut p = Parser::new("\"qwerty\"");
    assert!(p.parse().is_some());
    assert_eq!(p.tokens.len(), 1);
    assert_eq!(p.tok_values(), vec_strings!["\"qwerty\""]);
  }

  #[test]
  fn test_parser_index() {
    let mut p = Parser::new("[1, 2]");
    assert!(p.parse().is_some());
    assert_eq!(p.tok_values(), vec_strings!("[", "1", " ", "2", "]"))
  }

  #[test]
  fn test_parser_addr() {
    let mut p = Parser::new("{a,Z}");
    assert!(p.parse().is_some());
    assert_eq!(p.tok_values(), vec_strings!("{", "a", "Z", "}"))
  }

  // #[test]
  // fn test_parser_assignment() {
    
  //   let mut p = Parser::new("val x= 1");
  //   assert!(p.parse().is_some());
  //   assert_eq!(p.tok_values(), vec_strings!["val", " ", "x", " ", "1"]);
  // }

  #[test]
  fn test_parser_list() {
    let mut p = Parser::new("1,2,3");
    assert!(p.parse().is_some());
    assert_eq!(p.tok_values(), vec_strings!["1","2","3"]);
  }

  #[test]
  fn test_parse_eval_math() {
    let mut p = Parser::new("3*7*(1+1)/2");
    let node = p.parse();
    assert!(node.is_some());

    let res = node.unwrap().eval(&p);
    assert_eq!(res, Value::N(Decimal::new(21,0)))
  }
  
  #[test]
  fn test_parse_eval_values() {
    let mut p = Parser::new("1,2,3");
    let node = p.parse();
    assert!(node.is_some());

    // TODO - left recursive or manually unwrap in x_or_more

    let res = node.unwrap().eval(&p);
    assert_eq!(res, Value::L(vec![
      Value::N(Decimal::from(1)),
      Value::N(Decimal::from(2)),
      Value::N(Decimal::from(3)),
    ]))
  }

  #[test]
  fn test_eval_basics() {
    use Node::*;
    use Value::*;
    use TokTag::*;

    fn dec(num: i64, scale: u32) -> Decimal {
      Decimal::new(num, scale)
    }

    let mut state = EvalState::new();
    let ast = vec![
      Leaf{tok: Token::empty(NumTok,0), value: state.push_value(N(dec(1, 0)))},
      Leaf{tok: Token::empty(NumTok,0), value: state.push_value(N(dec(2, 0)))},
      Leaf{tok: Token::empty(NumTok,0), value: state.push_value(I(2))},
      Leaf{tok: Token::empty(NumTok,0), value: state.push_value(F(2.0))},
      BinOp{op: '+', lhs: NodeId(0), rhs: NodeId(1)},
      BinOp{op: '+', lhs: NodeId(0), rhs: NodeId(2)},
      BinOp{op: '+', lhs: NodeId(0), rhs: NodeId(3)},
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