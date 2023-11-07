# Valuator

Valuator is a spreadsheet-like application designed for financial modelling. It is being developed for two reasons:

1. To learn Rust. For real. 
2. To replace a module system written in Google Sheets that I use to model real estate business.

## Project Layout

- `src-tauri`: Rust source code for the Tauri based **backend**.
- `src/app`: Typescript source code for the Next.js based **frontend**.
- [`docs`](docs/main.md): contains project and development docs.

## Running

From `src-tauri/`:

- `make test`. Run unit tests.
- `make dev`. Run Tauri app in development mode.