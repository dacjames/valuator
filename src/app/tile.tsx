'use client'

import { useEffect, useState, createContext, useContext, Dispatch, SetStateAction } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { headers } from 'next/dist/client/components/headers'
import { BoardContext, BoardUi } from './board'

function TileHeader(props: {
  headers: Array<String>,
}) {
  return <thead>
    <tr className='bg-slate-100'>
      <th className='border-b font-medium p-4 pl-8 pb-3 text-slate-400 text-left' key={-1}></th>
      {props.headers.map( (item: String, index: number) => {
        return <th className='border-b font-medium p-4 pl-8 pb-3 text-slate-400 text-left'
                   key={index}>{item}</th>
      })}
    </tr>
  </thead>
}

function TileRow(props: {
  tag: number,
  row: number,
  rowData: Array<String>,
  label: String,
}) {
  const {board, setBoard} = useContext(BoardContext);

  function updateCell(tag: number, pos: Array<number>) {
    return (event: any) => {
      invoke<BoardUi>('update_cell', {tag: tag, pos: pos, value: event.target.value})
        .then(setBoard).catch(console.error)
    }
  }

  return <tr>
    <th className='border-b font-medium p-4 pl-8 pb-3 text-slate-400 bg-slate-100 text-left' key={-1}>{props.label}</th>
    {props.rowData.map((item: String, index: number) => {
      return <td className="border-b border-slate-200 p-4 pl-8 text-slate-400 bg-white" 
                 key={index}>
                  {item}
                  <br/>
                  <input onChange={updateCell(props.tag, [index, props.row])} defaultValue={item.toString()}></input>
                </td>
    })}
  </tr>
}

/**
 * UI Data for a Tile.
 * 
 * @interface TileUi 
 * @member rows is the number of rows in tile
 * @member cells contains the cell contents in row-major order
 */
export interface TileUi {
  tag: number,
  rows: number,
  cells: Array<String>,
  rowLabels: Array<String>,
  colLabels: Array<String>,
}


export default function Tile( props: {
    tile: TileUi,
    setBoard: Dispatch<SetStateAction<BoardUi>>,
}) {

  const r = props.tile.rows;
  const c = props.tile.cells.length / r;

  return <div className='border border-black rounded-xl overflow-hidden'>
    <table className="border-collapse table-auto w-full text-sm">
      <TileHeader headers={props.tile.colLabels} />
      <tbody>
        {Array(r).fill(0).map( (_, ir: number) => {
          const rowData = props.tile.cells.slice(ir*c, ir*c+c);
          return <TileRow key={ir} 
                          row={ir}
                          rowData={rowData}
                          tag={props.tile.tag}
                          label={props.tile.rowLabels[ir]} />
        })}
      </tbody>
    </table>
    
  </div>
}