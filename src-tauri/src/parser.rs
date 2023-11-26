use std::any::Any;
use std::cmp::min;
use std::fmt::Debug;
use std::hash::Hash;
use std::convert::TryInto;
use const_str;
#[allow(unused)]
use slog::{info, warn};

use rust_decimal::Decimal;
// use rustc_hash::FxHashMap;
use log_derive::{logfn, logfn_inputs};

use crate::cell::{Val, Cell};
use crate::eval::{ObjectContext, Node};
use crate::eval::LIST_ELEMS;
use crate::tile::TileContext;
// use crate::tag::Tag;


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum Tok {
  Nil,
  Num,
  Op,
  Sym,
  KW,
  WS,
  Str,
  LPar, RPar,
  LBck, RBck,
  LBrc, RBrc,
}

impl Default for Tok {
  fn default() -> Self {
    Self::Nil
  }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
  pos: u32,
  len: u16,
  tag: Tok,
}

impl Token {
  pub fn new(tag: Tok, pos: u32, len: u16) -> Token {
    Token{
      pos: pos,
      len: len,
      tag: tag,
    }
  }
  pub fn empty(tag: Tok, pos: u32) -> Token {
    Token::new(tag, pos, 0)
  }
}

impl Default for Token {
  fn default() -> Self {
    Token{pos: 0, len: 0, tag: Tok::default()}
  }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub u32);
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueId(pub u32);

impl Default for NodeId {
  fn default() -> Self {
    NodeId(0)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RuleKey(usize);

const N_RULE_KEYS: usize = 3;
const fn rule_key(name: &'static str) -> RuleKey {
  if const_str::equal!(name, "expr_list") {
    RuleKey(2)
  } else if const_str::equal!(name, "expr") {
    RuleKey(1)
  } else {
    RuleKey(0)
  }
}

type MemoArray = [Option<Box<dyn Any>>; N_RULE_KEYS];

// #[derive(Debug)]
pub struct Parser {
  tokens: Vec<Token>,
  nodes: Vec<Node>,
  values: Vec<Val>,

  // memos: FxHashMap<RuleKey, Box<dyn Any>>,
  memos: MemoArray,

  buf: Vec<char>,
  pos: usize,
}

impl Debug for Parser {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("«")?;
    for (i, ch) in self.buf.iter().enumerate() {
      if i == self.pos {
        f.write_str("|")?;
      }
      f.write_str(ch.to_string().as_str())?;
    }
    f.write_str("»")?;
    Ok(())
  }
}

#[allow(unused)]
impl Parser {
  pub fn new<S: Into<String>>(input: S) -> Parser {
    Parser {
      tokens: vec![],
      nodes: vec![Node::default()],
      values: vec![Val::default()],
      memos: [None, None, None],
      buf: input.into().chars().collect(),
      pos: 0,
    }
  }

  fn tok_ctx(&self, tag: Tok) -> TokCtx {
    TokCtx{ tok: Token::empty(tag, self.pos as u32) }
  }

  fn push_tok<T: Copy + Default>(&mut self, tag: Tok, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<T> {
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

  fn push_value(&mut self, value: Val) -> ValueId {
    let id = self.values.len() as u32;
    self.values.push(value);
    ValueId(id)
  }

  fn yield_tok<T: Copy + Default>(&mut self, tag: Tok, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<Token> {
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
    self.memos = [None, None, None];
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
    self.push_tok(Tok::WS, |s|{s.match_ws()})
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

  fn char_caseins(&mut self, needle: char) -> Option<char> {
    let item = self.next()?;
    if item.to_lowercase().next()? == needle.to_lowercase().next()? { 
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

  fn scan_string<S: Into<String>>(&mut self, needle: S, char_rule: impl Fn(&mut Parser, char) -> Option<char>) -> Option<char> {
    let needle_string: String = needle.into();
    let mut iter = needle_string.chars();
    let mut res = char_rule(self, iter.next()?)?;
    for ch in iter {
      res = self.char(ch)?;
    }
    Some(res)
  }

  fn string<S: Into<String>>(&mut self, needle: S) -> Option<char> {
    self.scan_string(needle, |s, ch|s.char(ch))
  }

  fn string_caseins<S: Into<String>>(&mut self, needle: S) -> Option<char> {
    self.scan_string(needle, |s, ch|s.char_caseins(ch))
  }

  fn class(&mut self, chars: &'static str) -> Option<char> {
    let item = self.next()?;
    if chars.contains(item) {
      Some(item)
    } else {
      None
    }
  }

  fn class_caseins(&mut self, chars: &'static str) -> Option<char> {
    let item = self.next()?;
    if chars.contains(item) || chars.contains(item.to_lowercase().next().unwrap()) {
      Some(item)
    } else {
      None
    }
  }


  fn not_class(&mut self, chars: &'static str) -> Option<char> {
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

  /// "Calls" a left-recursive rule.
  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn left<T: Copy + Default + 'static + Debug>(&self, key: RuleKey) -> Option<T> {
    let saved = self.memos.get(key.0)?;
    if let Some(saved) = saved {
      let res = *saved.downcast_ref::<Option<T>>()?;
      res
    } else {
      None
    }
  }

  #[logfn_inputs(Trace)]
  fn _leftpoline(&self, key: RuleKey) {
    // dummy
  }
  
  #[logfn(Trace)]
  /// Marks a rule as left-recursive
  fn leftpoline<T: Copy + Default + 'static + Debug>(&mut self, key: RuleKey, rule: impl Fn(&mut Parser) -> Option<T>) -> Option<T> {
    self._leftpoline(key);
    let mut saved: Option<T> = None;
    let state = self.save();
    let mut len_parsed = state.pos - self.pos;

    // call rule repeatedly so long as it finds longer parse
    // this does apparantly have a mathematic proof but
    // I only read a post by Guido who read the paper.
    // Thanks, Guido.
    loop {
      let res = rule(self);
      if res.is_none() {
        break;
      }
      let new_len = self.pos - state.pos;
      if new_len <= len_parsed {
        break;
      }
      saved = res;
      // save the value in the memo for left "call"
      self.memos[key.0] = Some(Box::new(saved));
      len_parsed = new_len;
    }
    saved
  }

  
  // fn x_num_nonzero(&mut self) -> Option<Node> {
  //   self.yield_tok(Tok::Num, |s| {
  //     s.maybe(|s|s.char('-'))?;
  //     s.class("123456789")?;
  //     s.zero_or_more(|s|s.class("0123456789"))?;
  //     s.maybe(|s|s.char('.'))?;
  //     s.zero_or_more(|s|s.class("0123456789"))
  //   }).and_then(|tok|{
  //     let decval = Decimal::from_str_radix(&self.tok_value(tok), 10).unwrap_or(Decimal::default());
  //     Some(Node::Leaf { tok, value: self.push_value(Val::Num(decval)) })
  //   })
  // }

  fn match_num_nonzero(&mut self) -> Option<char> {
    self.maybe(|s|s.char('-'))?;
    self.class("123456789")?;
    self.zero_or_more(|s|s.class("0123456789"))?;
    self.maybe(|s|s.char('.'))?;
    self.zero_or_more(|s|s.class("0123456789"))
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn match_num_zero(&mut self) -> Option<char> {
    self.char('0') 
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_num(&mut self) -> Option<Node> {
    self.yield_tok(Tok::Num, |s|s.select([
      |s|s.match_num_zero(),
      |s|s.match_num_nonzero(),
    ])).and_then(|tok|{
      let decval = Decimal::from_str_radix(&self.tok_value(tok), 10).unwrap_or(Decimal::default());
      Some(Node::Leaf { value: self.push_value(Val::Num(decval)) })
    })
  }

  fn match_string(&mut self, bookend: char) -> Option<char> {
    self.char(bookend)?;
    self.zero_or_more(move |s|{s.not_char(bookend)})?;
    self.char(bookend)?;
    Some(bookend)
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_string(&mut self) -> Option<Node> {
    self.yield_tok(Tok::Str, |s| {
      s.select([
        |s|{s.match_string('\'')},
        |s|{s.match_string('"')},
      ])
    }).and_then(|tok|{
      let pos = tok.pos as usize;
      let end = tok.len as usize + pos;
      let body: String = self.buf[pos+1..end-1].iter().collect();
      Some(Node::Leaf{ value: self.push_value(Val::Str(body)) })
    })
  }
  fn match_bool(&mut self, needle: &'static str, value: bool) -> Option<Node> {
    self.yield_tok(Tok::KW, |s|{
      s.string(needle)
    }).and_then(|tok|{
      Some(Node::Leaf { value: self.push_value(Val::Bool(value)) })
    })
  }
  
  fn r_true(&mut self) -> Option<Node> {
    self.match_bool("true", true)
  }

  fn r_false(&mut self) -> Option<Node> {
    self.match_bool("false", false)
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_bool(&mut self) -> Option<Node> {
    self.select([
      |s|{s.r_true()},
      |s|{s.r_false()},
    ])
  }

  fn match_plus(&mut self) -> Option<char> { self.char('+') }
  fn match_minus(&mut self) -> Option<char> { self.char('-') }
  fn match_star(&mut self) -> Option<char> { self.char('*') }
  fn match_fslash(&mut self) -> Option<char> { self.char('/') }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_term_literal(&mut self) -> Option<Node> {
    self.select([
      |s|{s.r_num()},
      |s|{s.r_string()},
      |s|{s.r_bool()},
    ])
  }

  fn match_lpar(&mut self) -> Option<char> {
    self.push_tok(Tok::LPar,|s|s.char('('))
  }
  fn match_rpar(&mut self) -> Option<char> {
    self.push_tok(Tok::RPar, |s|s.char(')'))
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_term_paren(&mut self) -> Option<Node> {
    self.match_lpar()?;
    let expr = self.r_expr()?;
    self.match_rpar()?;
    Some(expr)
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_term(&mut self) -> Option<Node> {
    self.select([
      |s|s.r_term_literal(),
      |s|s.r_term_sym(),
      |s|s.r_term_paren(),
    ])
  }

  fn match_binop(&mut self) -> Option<char> {
    self.push_tok(Tok::Op, |s|{
      s.select([
        |s|s.match_plus(),
        |s|s.match_minus(),
        |s|s.match_star(),
        |s|s.match_fslash(),
      ])
    })
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_expr_binop(&mut self) -> Option<Node> {
    let lnode = self.r_term()?;
    let left = self.push_node(lnode);

    self.maybe_ws()?;
    let op = self.match_binop()?;
    self.maybe_ws()?;
    let rnode = self.r_expr()?;
    let right = self.push_node(rnode);
    Some(Node::BinOp { op: op, lhs: left, rhs: right })
  }

  /// Construct a list from a zero_or_more list match
  fn build_list(&mut self, elems: Vec<NodeId>) -> Node  {
    let len = elems.len();
    let clampled_len = min(len, LIST_ELEMS);

    if len <= clampled_len { 
      let padding = vec![NodeId(0); LIST_ELEMS - clampled_len];
      let extended = [elems, padding].concat();
      return Node::List {
        elems: extended.try_into().unwrap(),
        len: len,
        link: None,
      };
    }

    let link_node = self.build_list(elems[LIST_ELEMS..].to_vec());
    Node::List {
      elems: elems[..LIST_ELEMS].try_into().unwrap(),
      len: len,
      link: Some(self.push_node(link_node)),
    }
  }

  /// Construct a list from a left_rec list match
  fn cons_list(&mut self, lnode: &Node, left: NodeId, right: NodeId) -> Node {
    match lnode {
      &Node::List{elems, len: LIST_ELEMS, link: None} => {
        let empty_node = Node::List{elems: [NodeId(0); LIST_ELEMS], len: 0, link: None};
        let empty = self.push_node(empty_node);
        let link = self.cons_list(&empty_node, empty, right);
        Node::List{elems: elems, len: LIST_ELEMS + 1, link: Some(self.push_node(link))}
      },
      &Node::List{elems, len, link: Some(link)} => {
        let orig_link = *self.get_node(&link);
        let new_link_node = self.cons_list(&orig_link, link, right);
        let new_link = self.push_node(new_link_node);
        Node::List{elems: elems, len: len + 1, link: Some(new_link)}
      }
      &Node::List{mut elems, len, link} => {
        elems[len] = right;
        Node::List{elems: elems, len: len+1, link: link}
      }, 
      _ => {
        let mut elems = [NodeId(0); LIST_ELEMS];
        elems[0] = left;
        elems[1] = right;
        Node::List{elems: elems, len: 2, link: None}
      }, 
    }
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn match_list_zero_or_more(&mut self) -> Option<Node> {
    let lnode: Node = self.left(rule_key("expr"))?;
    let first = self.push_node(lnode);

    let mut elems = vec![first];

    self.maybe_ws()?;
    self.char(',')?;
    self.zero_or_more(|s|{
      s.maybe_ws()?;
      let node = s.r_term()?;
      let nid = s.push_node(node);
      s.maybe_ws()?;
      s.maybe(|s|s.char(','))?;
      s.maybe_ws()?;
      elems.push(nid);
      Some(node)
    })?;
    Some(self.build_list(elems))
  }


  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn match_list_left_rec(&mut self) -> Option<Node> {
    // TODO benchmark match_list_left_rec against match_list_zero_or_more
    let lnode: Node = self.left(rule_key("expr"))?;
    let left = self.push_node(lnode);

    self.maybe_ws()?;
    self.char(',')?;
    self.maybe_ws()?;
    let rnode = self.r_term()?;
    let right = self.push_node(rnode);

    Some(self.cons_list(&lnode, left, right))
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_expr_list(&mut self) -> Option<Node> {
    self.match_list_left_rec()
  }


  fn r_term_sym(&mut self) -> Option<Node> {
    self.yield_tok(Tok::Sym, |s|{
      s.one_or_more(|s|{ s.class_caseins("abcdefghijklmnopqrstuvwxyz") })
    }).and_then(|tok|{
      // todo cache value
      let value = self.tok_value(tok);
      Some(Node::Leaf { value: self.push_value(Val::Str(value)) })
    })
  }

  fn match_compound(&mut self, start: (char, Tok), end: (char, Tok), cb: impl Fn(NodeId, NodeId) -> Node) -> Option<Node> {
    self.push_tok(start.1, |s|s.char(start.0))?;
    self.maybe_ws()?;
    let inner = self.r_expr()?;
    let mut row = NodeId(0);
    let mut col = NodeId(0);

    if let Node::List{elems, len, link: _} = inner {
      row = elems[0];
      col = elems[1];
    } else {
      row = self.push_node(inner);
    }

    self.push_tok(end.1, |s|s.char(end.0))?;

    Some(cb(row, col))
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_expr_index(&mut self) -> Option<Node> {
    self.match_compound(('[', Tok::LBck), (']', Tok::RBck), |r, c| {
      Node::Index { row: r, col: c}
    })
  }

  fn r_expr_addr(&mut self) -> Option<Node> {
    self.match_compound(('{', Tok::LBrc), ('}',  Tok::RBrc), |r, c| {
      Node::Addr { row: r, col: c}
    })
  }

  fn r_expr_lookup(&mut self) -> Option<Node> {
    self.r_term_sym()
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn match_expr(&mut self) -> Option<Node>  {
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
  
  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  fn r_expr(&mut self) -> Option<Node> {
    self.leftpoline(rule_key("expr"), |s|s.match_expr())
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


impl ObjectContext for Parser {
  fn get_value(&self, node: &ValueId) -> &Val {
    &self.values[node.0 as usize]
  }
  fn get_node(&self, node: &NodeId) -> &Node {
    &self.nodes[node.0 as usize]
  }
}

impl TileContext for Parser {
  fn get_pos<const CARD: usize>(&mut self, _pos: [usize; CARD]) -> Cell {
    panic!("not impl!")
  }
  fn get_labels<const CARD: usize>(&mut self, _pos: [String; CARD]) -> Cell {
    panic!("not impl!")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;
  use crate::{board::Board, cell::Cell};
  use crate::eval::EvalState;
  use slog::{Drain, Logger, o};

  macro_rules! vec_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
  }

  #[allow(unused)]
  fn test_logger() -> slog_scope::GlobalLoggerGuard {
    let decorator = slog_term::PlainSyncDecorator::new(slog_term::TestStdoutWriter);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let logger = Logger::root(drain, o!());

    let guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    guard
  }

  #[test]
  fn test_parser_basics() {
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

    p = Parser::new("0");
    assert_eq!(p.scan(), vec_strings!["0"]);
    
    p = Parser::new("111111");
    assert_eq!(p.scan(), vec_strings!["111111"]);

    p = Parser::new("10");
    assert_eq!(p.scan(), vec_strings!["10"]);

    p = Parser::new("999");
    assert_eq!(p.scan(), vec_strings!["999"]);

    p = Parser::new("-42");
    assert_eq!(p.scan(), vec_strings!["-42"]);

    p = Parser::new("399+84729");
    assert_eq!(p.scan(), vec_strings!["399","+","84729"]);

    p = Parser::new("1+1");
    assert_eq!(p.scan(), vec_strings!["1","+","1"]);

    p = Parser::new("1-1");
    assert_eq!(p.scan(), vec_strings!["1","-","1"]);

    p = Parser::new("1--1");
    assert_eq!(p.scan(), vec_strings!["1","-","-1"]);
    p = Parser::new("1 --1");
    assert_eq!(p.scan(), vec_strings!["1"," ","-","-1"]);

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
    let mut p = Parser::new("789+234");
    assert!(p.parse().is_some());
    assert_eq!(p.tokens.len(), 3);
    assert_eq!(p.tok_values(), vec_strings!["789","+","234"]);
  }

  #[test]
  fn test_parser_strings() {
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
    // let _scope_guard = test_logger();
    
    let mut p = Parser::new("[1, 2]");
    let res = p.parse();
    assert!(res.is_some());
    assert_eq!(p.tok_values(), vec_strings!("[","1"," ","2","]"));
    let ast = res.unwrap();
    assert_eq!(Node::Index{ row: NodeId(1), col: NodeId(2) }, ast);

    p = Parser::new("[1]");
    let res = p.parse();
    assert!(res.is_some());
    assert_eq!(p.tok_values(), vec_strings!("[","1","]"));
    let ast = res.unwrap();
    assert_eq!(Node::Index{ row: NodeId(1), col: NodeId(0) }, ast);
  }

  #[test]
  fn test_parser_addr() {
    let mut p = Parser::new("{a,Z}");
    let res = p.parse();
    assert!(res.is_some());
    assert_eq!(p.tok_values(), vec_strings!("{", "a", "Z", "}"));
    let ast = res.unwrap();
    assert_eq!(Node::Addr { row: NodeId(1), col: NodeId(2) }, ast);
  }


  #[test]
  #[allow(non_snake_case)]
  fn test_parse_eval_list() {
    // let _scope_guard = test_logger();

    let mut p = Parser::new("1,2,3");
    assert!(p.parse().is_some());
    assert_eq!(p.tok_values(), vec_strings!["1","2","3"]);

    let mut p = Parser::new("1,2,(3,4,5)");
    assert!(p.reparse().is_some());
    assert_eq!(p.tok_values(), vec_strings!["1","2","(","3","4","5",")"]);

    let mut p = Parser::new("1,2,3,4,5,6,7,8,9,10,11,12");
    let list_opt = p.parse();
    assert!(list_opt.is_some());
    assert_eq!(p.tok_values(), vec_strings!["1","2","3","4","5","6","7","8","9","10","11","12"]);
    let list = list_opt.unwrap();
    assert!(match list {
      Node::List { elems: _, len: _, link} => link.is_some(),
      _ => false,
    });
    match list {
      Node::List { elems: _, len: _, link: Some(n)} =>
        assert!(matches!(p.get_node(&n), _List)),
      _ => assert!(false),
    };

    let list_val = list.eval(&mut p);
    assert!(matches!(&list_val, _List));
    assert_eq!(list_val, Val::List(vec![
      Val::Num(dec!(1)),
      Val::Num(dec!(2)),
      Val::Num(dec!(3)),
      Val::Num(dec!(4)),
      Val::Num(dec!(5)),
      Val::Num(dec!(6)),
      Val::Num(dec!(7)),
      Val::Num(dec!(8)),
      Val::Num(dec!(9)),
      Val::Num(dec!(10)),
      Val::Num(dec!(11)),
      Val::Num(dec!(12)),
    ]));
  }

  #[test]
  fn test_parse_eval_math() {
    let mut p = Parser::new("3*7*(1+1)/2");
    let node = p.parse();
    assert!(node.is_some());

    let res = node.unwrap().eval(&mut p);
    assert_eq!(res, Val::Num(Decimal::new(21,0)))
  }

  #[test]
  fn test_parse_eval_values() {
    let mut p = Parser::new("1,2,3");
    let node = p.parse();
    assert!(node.is_some());

    let res = node.unwrap().eval(&mut p);
    assert_eq!(res, Val::List(vec![
      Val::Num(Decimal::from(1)),
      Val::Num(Decimal::from(2)),
      Val::Num(Decimal::from(3)),
    ]))
  }

  #[test]
  fn test_util_rule_key() {
    assert_eq!(RuleKey(0), rule_key("asdf"))
  }

  
}
