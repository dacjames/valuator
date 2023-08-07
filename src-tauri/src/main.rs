// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tile;
mod tag;
mod handle;
mod constants;
mod board;

use std::{sync::Mutex};

use tauri::State;

use board::Board;


#[derive(Default)]
struct BoardState(Mutex<Option<Board<f64>>>);


#[tauri::command]
fn tile(state: State<'_, BoardState>) -> tile::TileUi {
  let mut board = Board::default();
  let tag = board.add_tile();

  board.set_pos(tag, [0, 0], 2.0);
  board.set_pos(tag, [0, 1], 17.5);
  board.set_pos(tag, [0, 2], 37.8);
  board.set_pos(tag, [1, 0], 3.0);

  return board.tile(tag).render();
}


#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}", name)
}

fn main() {
  tauri::Builder::default()
    .manage::<BoardState>( BoardState( Mutex::new(Some( Board::<f64>::default() )) ))
    .invoke_handler(tauri::generate_handler![greet])
    .invoke_handler(tauri::generate_handler![tile])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}