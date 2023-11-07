# Architecture

## Parser Architecture

The parser is hand-written following the pattern of a PEG parser. It borrows heavily from Guido van Rossum's excellent [series](https://medium.com/@gvanrossum_83706/peg-parsing-series-de5d41b2ed60) on the topic.

It operates in a single pass, tokenizing and parsing simultaneously.

## Parser

The Parser is build around a few key components:

- `buf`. Holds the code to be parsed as a vector of `char`s.
- `tokens`. Tokens in the buffer. **Tokens** are stored in only 8 bytes: a u32 index, u16 tag, and u16 length.
- `nodes`. Nodes in the parsed AST. Nodes are connected with handles (indexes in `nodes`) rather than pointers.
- `values`. Vec of parsed Values referenced by leaf nodes. Serves to intern values.


### Parser State

The parser state consists of:

- `pos`. Position in the `buf` to be parsed left
- Lengths of all the vectors: 
  - tokens, nodes, values
  - vectors are cheaply truncated during rollback.

### Pointer-Free

Nodes in the AST do not contain pointers. Instead, they  using integer identifiers to refer to other nodes (ex: BinOp refers to left and right) and to values (ex: Leaf of value `2.0`).

Avoiding pointers allows ASTs to be created and destroyed efficiently without memory management overhead. Using indexes is safe because the parser pushes objects onto their vec during construction.

`Node` variants must never contain other Nodes, only `NodeId`.