// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tile;
mod tag;
mod handle;
mod constants;
mod board;

use std::sync::RwLock;

use tag::Tag;
use tauri::State;

use board::Board;
use tile::TileTrait;


#[derive(Default)]
struct BoardState{
  board: RwLock<Board<f64>>
}

#[tauri::command]
fn tile(state: State<BoardState>) -> tile::TileUi {
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

  let data = value.parse::<f64>().unwrap_or(0.0);
  board.set_pos(tag, pos, data);

  return board.render()
}

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}", name)
}

fn main() {
  tauri::Builder::default()
    .manage::<BoardState>( BoardState{ board: RwLock::new(Board::<f64>::default()) })
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