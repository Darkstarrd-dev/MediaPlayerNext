import { z } from "zod";
import { archiveEntryIdSchema, mediaSourceIdSchema, sourceIdSchema, taskIdSchema } from "../models/ids.js";

export const archiveEntriesRequestSchema = z.object({
  sourceId: sourceIdSchema.optional(),
  mediaSourceId: mediaSourceIdSchema.optional(),
});

export const archiveEntryDetailRequestSchema = z.object({
  archiveEntryId: archiveEntryIdSchema,
});

export const archiveNormalizeRequestSchema = z.object({
  sourceId: sourceIdSchema.optional(),
  mediaSourceId: mediaSourceIdSchema.optional(),
});

export const archiveNormalizeStatusRequestSchema = z.object({
  taskId: taskIdSchema,
});
