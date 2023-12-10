
use std::collections::HashMap;
use std::fmt;

use itertools::Itertools;
use log_derive::{logfn, logfn_inputs};
use petgraph::Directed;
use petgraph::stable_graph::{StableGraph, NodeIndex, DefaultIx};
use serde::{Serialize, Deserialize};

use crate::constants::*;
use crate::eval::MainContext;
#[allow(unused)]
use crate::handle::{pos_to_cellid, index_to_pos, pos_to_index};
use crate::cell::{CellOps, Val, Cell, CellId, CRef, CellRef};
use crate::parser::Parser;
use crate::rpc::{TileUi, CellUi};


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct TileId(pub usize);

impl TileId {
  pub fn next(&self) -> TileId {
    TileId(self.0 + 1)
  }
}


pub trait TileContext {
  fn get_cell<const CARD: usize, R: Into<CellRef<CARD>>+fmt::Debug>(&mut self, cellref: R) -> (CellId, Cell);
}

type DepsIx = DefaultIx;
type DepsGraph = StableGraph<CellId, u32, Directed, DepsIx>;
type DepsLookup = HashMap<CellId, NodeIndex<DepsIx>>;

pub struct Tile<Cell: CellOps>{
  pub tag: TileId,
  pub rows: usize,
  pub cols: usize,
  cells: [Cell; ROW_MAX * COL_MAX],
  lbls: [String; ROW_MAX + COL_MAX],
  pub deps: DepsGraph,
  pub lookup: DepsLookup,
}

impl<Cell: CellOps> fmt::Debug for Tile<Cell> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("Tile(")?;
    f.write_fmt(format_args!("{:?}", self.tag))?;
    f.write_str(")")
  }
}

pub struct TileIter<'a, Cell: CellOps>{
  tile: &'a Tile<Cell>,
  curr: usize,
}

impl<'a, Cell: CellOps> Iterator for TileIter<'a, Cell> {
  type Item = (CellId, &'a Cell);
  fn next(&mut self) -> Option<Self::Item> {
    // TODO remove empty cells from tile iteration
    if self.curr >= (ROW_MAX * COL_MAX) {
      return None
    }

    let id = CellId(self.curr as u32);
    let cell: &Cell = self.tile.cells.get(self.curr).unwrap();
    self.curr += 1;
    Some((id, cell))
  }
}

impl<Cell: CellOps> Tile<Cell> {
  #[allow(unused)]
  pub fn iter<'a>(&'a self) -> TileIter<'a, Cell> {
    TileIter{
      tile: self,
      curr: 0
    }
  }
}

pub struct TileState<'a> {
  tile: &'a mut Tile<Cell>,
  cell: CellId,
}

impl TileContext for TileState<'_> {
  fn get_cell<const CARD: usize, R: CRef<CARD>>(&mut self, cref: R) -> (CellId, Cell) {
    let cellref: CellRef<CARD> = cref.into();
    self.tile.track_dep(self.cell, cellref.clone());
    (self.tile.resolve(cellref.clone()), self.tile.get_cell(cellref))  
  }
}

impl Tile<Cell> {
  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  pub fn eval_cell<const CARD: usize, R: Into<CellRef<CARD>>+std::fmt::Debug>(&mut self, tile: TileId, cref: R) -> Option<Cell> {
    let cellid = self.resolve(cref);
    let cell = self.get_cell_by_id(cellid);
    let mut p = Parser::new(cell.formula.clone());

    match p.parse() {
      Some(node) => {
        let mut state = TileState{tile: self, cell: cellid};
        let mut ctx = MainContext{parser: &p, state: &mut state};
        let res = node.eval(&mut ctx);
        
        let deps = self.cell_deps(cellid);

        let cell = Cell{ value: res, ..cell };
        self.set_cell(cellid, cell.clone());

        for dep in deps {
          self.eval_cell(tile, dep).map(|c|self.set_cell_by_id(dep, c));
        }

        Some(cell)
      },
      None => {
        self.update_cell(cellid, |cell|
          Cell{ value: Val::Str("error".to_owned()), ..cell}
        );
        None
      }
    }
    // None
  }
}

impl<C: CellOps>  Tile<C>{
  pub fn new(tag: TileId) -> Tile<C> {
    let mut lbls: [String; ROW_MAX + COL_MAX] = Default::default();

    ('a' ..= 'z').take(COL_MAX).enumerate().for_each( |(i, ch)| {
      lbls[i] = ch.to_string().to_ascii_uppercase();
    });

    (1 ..= ROW_MAX).take(ROW_MAX).enumerate().for_each( |(i, n)| {
      lbls[COL_MAX + i] = n.to_string();
    });

    let cells: [C; ROW_MAX * COL_MAX] = std::array::from_fn(|_| C::default());

    return Tile {
      tag: tag,
      rows: 0,
      cols: 0,
      cells: cells,
      lbls: lbls,
      deps: DepsGraph::default(),
      lookup: DepsLookup::default(),
    }
  }

  #[allow(unused)]
  pub fn len(&self) -> usize {
    return self.rows * self.cols;
  }

  pub fn get_cell_by_id(&self, cell: CellId) -> C {
    return self.cells[cell.0 as usize].clone()
  }

  pub fn set_cell_by_id(&mut self, cell: CellId, data: C) {
    let index = cell.0 as usize;

    let (col, row) = index_to_pos(index);
    if col >= self.cols {
      self.cols = col + 1;
    }
    if row >= self.rows {
      self.rows = row + 1;
    }

    let ix = self.deps.add_node(cell);
    
    self.lookup.entry(cell).or_insert_with(||ix);

    self.cells[index] = data;
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  pub fn cell_deps<const CARD: usize, R: Into<CellRef<CARD>>+std::fmt::Debug>(&self, cellref: R) -> Vec<CellId> {
    let cellid = self.resolve(cellref);
    let ix: NodeIndex = self.lookup[&cellid];

    self.deps.neighbors(ix)
      .map(|target|*self.deps.node_weight(target).unwrap())
      .collect_vec()
  }

  pub fn get_cell<const CARD: usize, R: Into<CellRef<CARD>>+std::fmt::Debug>(&self, cellref: R) -> C {
    let cellid = self.resolve(cellref);
    self.get_cell_by_id(cellid)
  }

  pub fn set_cell<const CARD: usize, R: Into<CellRef<CARD>>+std::fmt::Debug>(&mut self, cellref: R, data: C) {
    let cellid = self.resolve(cellref);
    self.set_cell_by_id(cellid, data)
  }

  pub fn update_cell<const CARD: usize, R: Into<CellRef<CARD>>+std::fmt::Debug>(&mut self, cellref: R, f: impl FnOnce(C) -> C) -> Option<C>{
    let cellid = self.resolve(cellref);
    let old = self.get_cell_by_id(cellid);
    let new = f(old.clone());
    self.set_cell_by_id(cellid, new.clone());
    Some(new)
  }


  fn pos_for<const CARD: usize>(&self, lbls: [String; CARD]) -> [usize; CARD] {
    let mut pos: [usize; CARD] = [0; CARD];

    for (i, lbl) in lbls.iter().enumerate() {
      pos[i] =
        match self.lbls.iter().position(
          |hay| { hay.eq(lbl) }
        ) {
          Some(n) => if n < COL_MAX { n } else { n - COL_MAX },
          None => 0,
        };
    };

    return pos
  }

  pub fn resolve<const CARD: usize, R: Into<CellRef<CARD>>+std::fmt::Debug>(&self, cellref: R) -> CellId {
    let cellref: CellRef<CARD> = cellref.into();
    match cellref {
      CellRef::Pos(pos) => pos_to_cellid(pos),
      CellRef::Label(labels) => pos_to_cellid(self.pos_for(labels)),
      CellRef::Id(cellid) => cellid,
    }
  }

  #[logfn(Trace)]
  #[logfn_inputs(Trace)]
  pub fn track_dep<const CR: usize, const CQ: usize, R, Q>(&mut self, downstream: R, upstream: Q) -> String
    where R: Into<CellRef<CR>>+std::fmt::Debug, Q: Into<CellRef<CQ>>+std::fmt::Debug {
    let downstream = self.resolve(downstream);
    let upstream = self.resolve(upstream);

    let upstream_ix: NodeIndex = self.lookup[&upstream];
    let downstream_ix: NodeIndex = self.lookup[&downstream];

    // The edge points upstream -> downstream so we can scan upstream.neighbors()
    // to recalculate values when upstream changes.
    self.deps.add_edge(upstream_ix, downstream_ix, 1);

    format_args!("{upstream:?} @ {upstream_ix:?} -> {downstream:?} @ {downstream_ix:?}").to_string()
  }


  pub fn render(&self) -> TileUi {
    let c = self.cols;
    let r = self.rows;
    let mut cells: Vec<CellUi> = vec![Default::default(); c*r];

    // 0, 0 => 0
    // 0, 1 => 2
    // 0, 2 => 4
    // 1, 0 => 1
    // 1, 1 => 3
    // 1, 2 => 5
    for ic in 0..c {
      for ir in 0..r {
        cells[ir * c + ic] = self.get_cell([ic, ir]).render();
      }
    }

    return TileUi {
      tag: self.tag,
      rows: r as u32,
      cells: cells,
      colLabels: self.lbls.iter().take(c).cloned().collect(),
      rowLabels: self.lbls.iter().skip(COL_MAX).take(r).cloned().collect(),
    }
  }
}


#[cfg(test)]
mod tests {
    use crate::rpc::ScalarValueUi;
    use crate::rpc::{ValueUi, TypeUi};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_tile_labels() {
      let mut t = Tile::<isize>::new(TileId(0));

      let pos1 = t.pos_for(["A".to_owned()]);
      assert_eq!([0], pos1);

      let pos2 = t.pos_for(["B".to_owned(), "2".to_owned()]);
      assert_eq!([1, 1], pos2);

      assert_eq!([0, 0], t.pos_for(["A".to_owned(), "1".to_owned()]));
      assert_eq!([0, 1], t.pos_for(["A".to_owned(), "2".to_owned()]));

      t.set_cell([0, 0], 1);
      t.set_cell([0, 1], 2);
      t.set_cell([1, 0], 3);
      t.set_cell([1, 1], 4);

      assert_eq!(t.get_cell(["A".to_owned()]), 1);
      assert_eq!(t.get_cell(["A".to_owned(), "1".to_owned()]), 1);
      assert_eq!(t.get_cell(["A".to_owned(), "2".to_owned()]), 2);
    }

    #[test]
    fn test_tile_basics() {
      let mut t = Tile::<isize>::new(TileId(0));
      t.set_cell([0, 0],  1);
      t.set_cell([0, 1], 2);
      t.set_cell([1, 0], 3);
      t.set_cell([1, 1], 4);


      assert_eq!(t.get_cell([0]), 1);
      assert_eq!(t.get_cell([0, 0]), 1);
      assert_eq!(t.get_cell([0, 1]), 2);
      assert_eq!(t.get_cell([1]), 3);
      assert_eq!(t.get_cell([1, 0]), 3);
      assert_eq!(t.get_cell([1, 1]), 4);
    }

    // todo fix test_tile_render
    #[ignore = "test is broken, not the unit"]
    #[test]
    fn test_tile_render() {
      let mut t = Tile::<isize>::new(TileId(0));
      t.set_cell([0, 0], 1);
      t.set_cell([0, 1], 2);
      t.set_cell([0, 2], 3);
      t.set_cell([1, 0], 4);
      t.set_cell([1, 1], 5);
      t.set_cell([1, 2], 6);

      let ui = t.render();

      assert_eq!(ui.cells.len() as u32 / ui.rows, 2);
      assert_eq!(ui.rows, 3);

      assert_eq!(ui.colLabels.len(), 2);
      assert_eq!(ui.rowLabels.len(), 3);

      assert_eq!(ui.colLabels, vec![
        "A".to_owned(),
        "B".to_owned(),
      ]);

      fn string_cell(value: &str) -> CellUi {
        return CellUi {
          value: ValueUi::V(ScalarValueUi{typ: TypeUi::String, value: value.to_owned()}),
          formula: value.to_owned(),
          style: String::new(),
        }
      }

      let expected_cells: Vec<CellUi> = vec![
        // COL: A          COL: B
        string_cell("1"), string_cell("4"),  // ROW: 1
        string_cell("2"), string_cell("5"),  // ROW: 2
        string_cell("3"), string_cell("6"),  // ROW: 3
       ];

      assert_eq!(ui.cells, expected_cells);
    }

    #[test]
    fn test_dumb() {
      let mut map: HashMap<i32, (usize, usize)> = HashMap::new();
      map.insert(1, (1,1));
      if let Some(x) = map.get_mut(&1) {
        x.0 = 2;
      }
      assert_eq!(map[&1], (2, 1));
    }
}
