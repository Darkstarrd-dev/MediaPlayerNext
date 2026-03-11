import { z } from "zod";

export const runtimeInfoSchema = z.object({
  databasePath: z.string().min(1),
  thumbnailCachePath: z.string().min(1),
});

export const setRuntimeStoragePathsInputSchema = z
  .object({
    databaseDir: z.string().min(1).optional(),
    thumbnailCacheDir: z.string().min(1).optional(),
  })
  .refine(
    (input) => input.databaseDir !== undefined || input.thumbnailCacheDir !== undefined,
    {
      message: "At least one storage path must be provided",
    },
  );

export type RuntimeInfo = z.infer<typeof runtimeInfoSchema>;
export type SetRuntimeStoragePathsInput = z.infer<typeof setRuntimeStoragePathsInputSchema>;
