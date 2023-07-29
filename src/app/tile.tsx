'use client'

import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { headers } from 'next/dist/client/components/headers'

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
  row: Array<String>,
  label: String,
}) {
  return <tr>
    <th className='border-b font-medium p-4 pl-8 pb-3 text-slate-400 bg-slate-100 text-left' key={-1}>{props.label}</th>
    {props.row.map((item: String, index: number) => {
      return <td className="border-b border-slate-200 p-4 pl-8 text-slate-400 bg-white" 
                 key={index}>{item}</td>
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
interface TileUi {
  rows: number,
  cells: Array<String>,
  rowLabels: Array<String>,
  colLabels: Array<String>,
}


export default function Tile() {
  const [uiData, setUiData] = useState<TileUi>({rows: 0, cells: [], rowLabels: [], colLabels:[]});

  useEffect(() => {
    invoke<TileUi>('tile', {})
      .then((uiData: TileUi) => {
        console.log(uiData);
        setUiData(uiData);
      })
      .catch(console.error)
  }, [])

  const r = uiData.rows;
  const c = uiData.cells.length / r;

  return <div className='border border-black rounded-xl overflow-hidden'>
    <table className="border-collapse table-auto w-full text-sm">
      <TileHeader headers={uiData.colLabels} />
      <tbody>
        {Array(r).fill(0).map( (_, ir: number) => {
          // const rowData = uiData.cells.slice(ir*r, (ir+1)*r);
          const rowData = uiData.cells.slice(ir*c, ir*c+c);
          return <TileRow key={ir} row={rowData} label={uiData.rowLabels[ir]} />
        })}
      </tbody>
    </table>
  </div>
}