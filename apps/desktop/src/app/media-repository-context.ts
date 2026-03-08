import { createContext } from 'react'
import type { MediaRepository } from '../repositories/media-repository'

export const MediaRepositoryContext = createContext<MediaRepository | null>(null)
