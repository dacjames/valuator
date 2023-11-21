// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate log;
#[macro_use(slog_o, slog_kv)]
extern crate slog;
extern crate slog_stdlog;
extern crate slog_scope;
extern crate slog_term;
extern crate slog_async;
use slog::Drain;

mod tile;
mod tag;
mod handle;
mod constants;
mod board;
mod cell;
mod rpc;
mod parser;
mod eval;

use std::sync::RwLock;

use rpc::TileUi;
use tag::Tag;
use tauri::State;

use board::Board;
use cell::Cell;
use parser::Parser;


#[derive(Default)]
struct BoardState{
  board: RwLock<Board<Cell>>
}

#[tauri::command]
fn tile(state: State<BoardState>) -> TileUi {
  let mut board = state.board.write().unwrap();

  let tag = board.add_tile();

  board.set_pos(tag, [0, 0], 2.0);
  board.set_pos(tag, [0, 1], 17.5);
  board.set_pos(tag, [0, 2], 37.8);
  board.set_pos(tag, [1, 0], 3.0);

  return board.tile(tag).render();
}

#[tauri::command]
fn board(state: State<BoardState>) -> board::BoardUi {
  let board = state.board.read().unwrap();
  // let board = Board::<f64>::default();

  let mut parser = Parser::new("dummy");
  let _silly = parser.parse();

  board.render()
}

#[tauri::command]
fn add_tile(state: State<BoardState>) -> board::BoardUi {
  let mut board = state.board.write().unwrap();

  let tag = board.add_tile();

  board.set_pos(tag, [0, 0], 2.0);
  board.set_pos(tag, [0, 1], 17.5);
  board.set_pos(tag, [0, 2], 37.8);
  board.set_pos(tag, [1, 0], 3.0);

  board.set_pos(tag, [1, 1], vec![1.0, 2.0]);
  board.set_pos(tag, [1, 2], true);

  board.render()
}

#[tauri::command]
fn add_column(state: State<BoardState>, tag: Tag) -> board::BoardUi {
  let mut board = state.board.write().unwrap();

  let cols = board.tile(tag).cols;
  // println!("wtf: {:?}", cols);
  board.set_pos(tag, [cols], 0.0);

  return board.render()
}

#[tauri::command]
fn add_row(state: State<BoardState>, tag: Tag) -> board::BoardUi {
  let mut board = state.board.write().unwrap();

  let rows = board.tile(tag).rows;
  let cols = board.tile(tag).cols;
  // println!("wtf: {:?} {:?}", cols, rows);
  board.set_pos(tag, [cols - 1, rows], 0.0);

  return board.render()
}

#[tauri::command]
fn update_cell(state: State<BoardState>, tag: Tag, pos: [usize; 2], value: String) -> board::BoardUi {
  let mut board = state.board.write().unwrap();
  let formula = value.clone();

  let mut p = Parser::new(value);
  match p.parse() {
    Some(node) => {
      let res = node.eval(&p);
      board.set_pos(tag, pos, Cell{
        value: res, 
        formula: formula, 
        style: "".to_owned(),
      });
    },
    None => board.set_pos(tag, pos, 0)
  }

  return board.render()
}

fn main() {
  let decorator = slog_term::TermDecorator::new().build();
  let drain = slog_term::FullFormat::new(decorator).build().fuse();
  let drain = slog_async::Async::new(drain).build().fuse();
  let logger = slog::Logger::root(drain, slog_o!("version" => env!("CARGO_PKG_VERSION")));

  let _scope_guard = slog_scope::set_global_logger(logger);
  let _log_guard = slog_stdlog::init().unwrap();
  // Note: this `info!(...)` macro comes from `log` crate
  info!("standard logging redirected to slog");

  tauri::Builder::default()
    .manage::<BoardState>( BoardState{ board: RwLock::new(Board::<Cell>::default()) })
    .invoke_handler(tauri::generate_handler![
        board, 
        tile,
        add_tile,
        add_column,
        add_row,
        update_cell,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}