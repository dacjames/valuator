'use client'

import { useEffect, useState, createContext, useContext, Dispatch, SetStateAction } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { headers } from 'next/dist/client/components/headers'
import Board, { BoardContext } from './board'
import { useRef, MutableRefObject } from 'react';
import { TileUi, BoardUi, CellUi,TypeUi, ValueUi } from './rpc';

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

function renderValue(value: ValueUi): any {
  switch (value.typ) {
    case TypeUi.Number: return <div>
      {value.value as String}
    </div>

    case TypeUi.List: 
    const values = value.value as String[];
    return <table>
        <tbody ><tr> {values.map((v) => {
            return <td className='border'>{v}</td>
        })}</tr></tbody>
    </table>

    case TypeUi.Boolean: return <div>
      {value.value}
    </div>

    default: return <div>
      {(()=>{console.log(value); return "Unknown Cell Value with Type: "+value.typ})()}
    </div>
  }
}

function TileCell(props: {
  tag: number,
  row: number,
  item: CellUi,
  index: number,
}) {
  const {board, setBoard} = useContext(BoardContext);
  const inputRef: MutableRefObject<HTMLInputElement> = useRef(null as unknown as HTMLInputElement);
  const [visibility, setVisibility] = useState(false);

  function cellUpdater(tag: number, pos: Array<number>) {
    return (event: any) => {
      console.log('wtf', event.target.value);
      invoke<BoardUi>('update_cell', {tag: tag, pos: pos, value: event.target.value})
        .then(setBoard).catch(console.error)
    }
  }

  function toggle() {
    setVisibility(!visibility); 
  }

  function focuser(inputRef: any) {
    return () => inputRef.current.focus()
  } 

  return <td className="relative z-0 border-b border-slate-200 p-4 pl-8 text-slate-400 bg-white" 
              key={props.index}>
      <div className={`${visibility? 'invisible' : 'visible'}`}
           onClick={() => { toggle(); setTimeout(focuser(inputRef), 0); }}>
        {renderValue(props.item.value)}
      </div>
      <input className={`border-2 absolute inset-0 ${visibility? 'visible' : 'invisible'}`}
        ref={inputRef}
        onChange={cellUpdater(props.tag, [props.index, props.row])}
        onBlur={toggle} 
        defaultValue={props.item.value.value.toString()}>
      </input>
  </td>

}

function TileRow(props: {
  tag: number,
  row: number,
  rowData: Array<CellUi>,
  label: String,
}) {
  return <tr>
    <th className='border-b font-medium p-4 pl-8 pb-3 text-slate-400 bg-slate-100 text-left' key={-1}>{props.label}</th>
    {props.rowData.map((item: CellUi, index: number) => {
      return <TileCell item={item} tag={props.tag} index={index} row={props.row}/>
    })}
  </tr>
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