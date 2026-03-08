import { z } from "zod";
import { archiveEntryIdSchema, sourceIdSchema, taskIdSchema } from "../models/ids.js";

export const archiveEntriesRequestSchema = z.object({
  sourceId: sourceIdSchema,
});

export const archiveEntryDetailRequestSchema = z.object({
  archiveEntryId: archiveEntryIdSchema,
});

export const archiveNormalizeRequestSchema = z.object({
  sourceId: sourceIdSchema,
});

export const archiveNormalizeStatusRequestSchema = z.object({
  taskId: taskIdSchema,
});
