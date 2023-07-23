'use client'

import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

export default function Greet() {
  const [message, setMessage] = useState('');

  useEffect(() => {
    invoke<string>('greet', { name: 'Intrepid User' })
      .then((message: string) => {
        setMessage(message);
      })
      .catch(console.error)
  }, [])

  // Necessary because we will have to use Greet as a component later.
  return <p>Greetings: {message}</p>
}