import { useContext } from 'react'
import { MediaRepositoryContext } from './media-repository-context'
import type { MediaRepository } from '../repositories/media-repository'

export function useMediaRepository(): MediaRepository {
  const repository = useContext(MediaRepositoryContext)
  if (repository === null) {
    throw new Error('MediaRepositoryProvider 尚未挂载')
  }
  return repository
}
