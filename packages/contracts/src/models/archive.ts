import { z } from "zod";
import { archiveEntryIdSchema, archiveIdSchema, sourceIdSchema, taskIdSchema } from "./ids.js";

export const archiveEntrySummarySchema = z.object({
  id: archiveEntryIdSchema,
  archiveId: archiveIdSchema,
  entryPath: z.string().min(1),
  entryName: z.string().min(1),
  pageIndex: z.number().int().nonnegative(),
  mediaKind: z.string().min(1),
  width: z.number().int().positive().optional(),
  height: z.number().int().positive().optional(),
});

export const archiveEntryDetailSchema = z.object({
  archiveEntryId: archiveEntryIdSchema,
  archiveId: archiveIdSchema,
  sourceId: sourceIdSchema,
  archivePath: z.string().min(1),
  entryPath: z.string().min(1),
  mediaKind: z.string().min(1),
});

export const archiveNormalizeResultSchema = z.object({
  taskId: taskIdSchema,
  sourceId: sourceIdSchema,
  archiveId: archiveIdSchema,
  normalizedZipPath: z.string().min(1),
  extractedFileCount: z.number().int().nonnegative(),
  indexedEntries: z.number().int().nonnegative(),
  archiveStatus: z.string().min(1),
});

export type ArchiveEntrySummary = z.infer<typeof archiveEntrySummarySchema>;
export type ArchiveEntryDetail = z.infer<typeof archiveEntryDetailSchema>;
export type ArchiveNormalizeResult = z.infer<typeof archiveNormalizeResultSchema>;
