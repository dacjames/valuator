# Architecture

## Parser Architecture

The parser is hand-written following the pattern of a PEG parser. It borrows heavily from Guido van Rossum's excellent [series](https://medium.com/@gvanrossum_83706/peg-parsing-series-de5d41b2ed60) on the topic.

It operates in a single pass, tokenizing and parsing simultaneously.

## Parser

The Parser is build around a few key components:

- `buf`. Holds the code to be parsed as a vector of `char`s.
- `tokens`. Tokens in the buffer. **Tokens** are stored in only 8 bytes: a u32 index, u16 tag, and u16 length.
- `nodes`. Nodes in the parsed AST. Nodes are connected with handles (indexes in `nodes`) rather than pointers. Nodes are statically sized.
- `values`. Vec of parsed Values referenced by leaf nodes. Serves to intern values, which are dynamically sized.


### Parser State

The parser state consists of:

- `pos`. Position in the `buf` to be parsed left
- Vectors to store parsing objects
  - tokens, nodes, values
  - vectors are cheaply truncated during rollback.
  - objects may be cached in a future version.

### Pointer-Free

Nodes in the AST do not contain pointers. Instead, they  using integer identifiers to refer to other nodes (ex: BinOp refers to left and right) and to values (ex: Leaf of value `2.0`).

Avoiding pointers allows ASTs to be created and destroyed efficiently with minimal memory management overhead. Indexes do have less compiler checks than references. The parser mitigates this risk by intelligently managing object lifecycles (e.g. cacheing objects during backtracking) and avoid `unsafe` indexing optimizations.

`Node` variants be `Copy` and must never contain other Nodes, only `NodeId`.

## Context

Evaluator's architecture employs a pattern we refer to as "Call with Callee". This pattern can be found in the Rust (and Go) std library, with the `Display` trait. The Callee in that casee is the `fmt::Formatter` passed to the fmt method.

In evaluator, the callee argument is called a **Context**. Functions employing this structure are said to be *contextual*. Contexts are implemented as small traits that are composed together to bound the parameters to contextual functions.

### Contextual Functions

- ObjectContext
  - `get_value(v: ValId) -> Val`
  - `get_node(n: NodeId) -> Node`
  
- ParseContext (TODO)
  - `get_token(tok: Token) -> String`
  - `str_token(tok: Token) -> &str`
  - `get_char(tok: Token, offset: i32) -> char`
  
- TileContext
  - `get_pos(caller: CellId, pos: [usize; CARD]) -> Val`
  - `cell_pos(caller: CellId, pos: [usize; CARD]) -> (CellId, Cell)`
  - `get_labels(caller: CellId, labels: [String; CARD]) -> Val`
  - `cell_labels(caller: CellId, labels: [String; CARD]) -> (CellId, Cell)`

- BoardContext (TODO)
  - `tile_id(t: TileId()) -> Tile`
  - `tile_name(name: S) -> Tile`