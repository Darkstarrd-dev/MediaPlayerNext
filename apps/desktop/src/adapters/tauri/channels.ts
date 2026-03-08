export type Unsubscribe = () => void

export function unsupportedChannel(name: string): never {
  throw new Error(`${name} 尚未在 src-tauri 完成 channel 接线`)
}

export function createNoopSubscription(): Unsubscribe {
  return () => undefined
}
