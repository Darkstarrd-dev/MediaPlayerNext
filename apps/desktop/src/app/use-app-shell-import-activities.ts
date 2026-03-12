import { useCallback, useState } from 'react'

export type ImportActivityStatus = 'running' | 'completed' | 'failed'

export interface ImportActivity {
  id: string
  title: string
  source: string
  status: ImportActivityStatus
  detail: string
  createdAt: string
}

export function useAppShellImportActivities() {
  const [importActivities, setImportActivities] = useState<ImportActivity[]>([])

  const appendImportActivity = useCallback((activity: Omit<ImportActivity, 'id' | 'createdAt'>): string => {
    const nextActivity: ImportActivity = {
      ...activity,
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      createdAt: new Date().toISOString(),
    }

    setImportActivities((current) => [nextActivity, ...current].slice(0, 8))
    return nextActivity.id
  }, [])

  const updateImportActivity = useCallback(
    (activityId: string, patch: Partial<Omit<ImportActivity, 'id' | 'createdAt'>>) => {
      setImportActivities((current) =>
        current.map((activity) =>
          activity.id === activityId
            ? {
                ...activity,
                ...patch,
              }
            : activity,
        ),
      )
    },
    [],
  )

  return {
    importActivities,
    appendImportActivity,
    updateImportActivity,
  }
}
