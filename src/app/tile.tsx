'use client'

import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

function TileHeader() {
  return <thead>
    <tr>
      <th className='border-b font-medium p-4 pl-8 pb-3 text-slate-400 text-left'>Field</th>
      <th className='border-b font-medium p-4 pl-8 pb-3 text-slate-400 text-left'>Value</th>
    </tr>
  </thead>
}

function TileRow(props: {
  row: Array<String>
}) {
  return <tr>
    {props.row.map((item: String, index: number) => {
      return <td className="border-b border-slate-100 p-4 pl-8 text-slate-500" 
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
}


export default function Tile() {
  const [uiData, setUiData] = useState<TileUi>({rows: 0, cells: []});

  useEffect(() => {
    invoke<TileUi>('tile', {})
      .then((uiData: TileUi) => {
        console.log(uiData);
        setUiData(uiData);
      })
      .catch(console.error)
  }, [])

  const r = uiData.rows;

  return <div className='border border-black rounded-xl overflow-hidden'>
    <table className="border-collapse table-auto w-full text-sm">
      <TileHeader />
      <tbody className='bg-white'>
        {Array(r).fill(0).map( (_, ir: number) => {
          const rowData = uiData.cells.slice(ir*r, (ir+1)*r);
          return <TileRow row={rowData} />
        })}
      </tbody>
    </table>
  </div>
}