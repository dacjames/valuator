# Architecture

## Parser Architecture

The parser is hand-written following the pattern of a PEG parser. It operates in a single pass, tokenizing and parsing simultaneously.

### `CodeString`

The `CodeString` struct contains three members:

- `bytes`. The utf-8 encoded byte string containing the code to be parsed.
- `tokens`. Tokens in the byte string. Stored in only 8 bytes: a u32 index, u16 tag, and u16 length.
- `nodes`. Nodes in the parsed AST. Nodes are connected with handles (indexes into nodes) rather than pointers.

