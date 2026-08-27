export type AppRoute =
  | { name: 'portal' }
  | { name: 'room'; roomId: string }
  | { name: 'clear'; roomId: string }
  | { name: 'not-found' }

const roomPath = /^\/rooms\/([^/]+)\/?$/
const clearPath = /^\/rooms\/([^/]+)\/clear\/?$/

export function resolveRoute(pathname: string): AppRoute {
  if (pathname === '/') return { name: 'portal' }

  const clearMatch = clearPath.exec(pathname)
  if (clearMatch?.[1]) return { name: 'clear', roomId: decodeURIComponent(clearMatch[1]) }

  const roomMatch = roomPath.exec(pathname)
  if (roomMatch?.[1]) return { name: 'room', roomId: decodeURIComponent(roomMatch[1]) }

  return { name: 'not-found' }
}
