import type { PropsWithChildren } from 'react'
import { MediaRepositoryContext } from './media-repository-context'
import type { MediaRepository } from '../repositories/media-repository'

export interface MediaRepositoryProviderProps extends PropsWithChildren {
  repository: MediaRepository
}

export function MediaRepositoryProvider({
  repository,
  children,
}: MediaRepositoryProviderProps) {
  return (
    <MediaRepositoryContext.Provider value={repository}>{children}</MediaRepositoryContext.Provider>
  )
}
