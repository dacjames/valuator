'use client'

import { Context, Dispatch, SetStateAction, createContext, useContext, useEffect, useState } from 'react'
import { TileUi} from './tile'
import Tile from './tile'
import { invoke } from '@tauri-apps/api/tauri'
import { headers } from 'next/dist/client/components/headers'

/**
 * UI Data for a Tile.
 * 
 * @interface BoardUi
 * @member tiles list of tiles in the board
 */
export interface BoardUi {
  tiles: Array<TileUi>,
}

function defaultBoard(): BoardUi {
  return {
    tiles: [],
  }
}

interface BoardState {
  board: BoardUi,
  setBoard: Dispatch<SetStateAction<BoardUi>>,
}

export const BoardContext = createContext({board: defaultBoard(), setBoard: (_: SetStateAction<BoardUi>) => {}});

export function BoardApp() {
  const [board, setBoard] = useState(defaultBoard());
  return (
    <BoardContext.Provider value={{board, setBoard}}>
      <Board />
    </BoardContext.Provider>
  )
}

export default function Board() {
  const {board, setBoard} = useContext(BoardContext);

  useEffect(() => {
    invoke<BoardUi>('board', {})
      .then(setBoard).catch(console.error)
  }, [setBoard])

  function addTile() {
    invoke<BoardUi>('add_tile', {})
      .then(setBoard).catch(console.error)
  }

  function addColumn(tag: number) {
    return () => {
      invoke<BoardUi>('add_column', {tag: tag})
        .then(setBoard).catch(console.error)
    }
  }

  function addRow(tag: number) {
    return () => {
      invoke<BoardUi>('add_row', {tag: tag})
        .then(setBoard).catch(console.error)
    }
  } 

  return <div className="z-10 w-full max-w-5xl items-center justify-between font-mono text-sm lg:flex">
    {board.tiles.map( (tile: TileUi, index: number) => {
      return <div>
        <Tile key={index} setBoard={setBoard} tile={tile} />
        <button className="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-full" 
              onClick={ addColumn(tile.tag) }>
          Add Column
        </button>
        <button className="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-full" 
              onClick={ addRow(tile.tag) }>
          Add Row
        </button>
      </div>
    })}
    <button className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full" 
            onClick={addTile}>
      Add Tile
    </button>
  </div>
}