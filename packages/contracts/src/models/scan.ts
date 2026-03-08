import { z } from "zod";
import { libraryIdSchema, taskIdSchema } from "./ids.js";

export const scanRunResultSchema = z.object({
  libraryId: libraryIdSchema,
  discovered: z.number().int().nonnegative(),
  insertedOrUpdated: z.number().int().nonnegative(),
  skippedUnchanged: z.number().int().nonnegative(),
  tombstoned: z.number().int().nonnegative(),
  taskId: taskIdSchema,
});

export const scanStatsSchema = z.object({
  libraryId: libraryIdSchema,
  sourceCount: z.number().int().nonnegative(),
  activeSourceCount: z.number().int().nonnegative(),
  missingSourceCount: z.number().int().nonnegative(),
});

export type ScanRunResult = z.infer<typeof scanRunResultSchema>;
export type ScanStats = z.infer<typeof scanStatsSchema>;
