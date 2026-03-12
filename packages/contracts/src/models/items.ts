import { z } from "zod";
import {
  archiveEntryIdSchema,
  archiveIdSchema,
  assetIdSchema,
  libraryIdSchema,
  mediaSourceIdSchema,
  sourceIdSchema,
} from "./ids.js";

export const itemSourceKindSchema = z.enum(["file", "archive_entry"]);

export const itemsListQuerySchema = z.object({
  libraryId: libraryIdSchema,
  mediaSourceId: mediaSourceIdSchema.optional(),
  sourceId: sourceIdSchema.optional(),
  page: z.number().int().positive().optional(),
  pageSize: z.number().int().positive().optional(),
});

export const itemListEntrySchema = z.object({
  assetId: assetIdSchema,
  sourceKind: itemSourceKindSchema,
  sourceRefId: z.string().min(1),
  libraryId: libraryIdSchema,
  mediaSourceId: mediaSourceIdSchema.optional(),
  sourceId: sourceIdSchema,
  archiveId: archiveIdSchema.optional(),
  entryPath: z.string().min(1).optional(),
  mime: z.string().min(1),
  thumbnailKey: z.string().min(1).optional(),
});

export const itemDetailSchema = z.object({
  assetId: assetIdSchema,
  sourceKind: itemSourceKindSchema,
  mime: z.string().min(1),
  libraryId: libraryIdSchema,
  sourceId: sourceIdSchema,
  filePath: z.string().min(1).optional(),
  archiveId: archiveIdSchema.optional(),
  archiveEntryId: archiveEntryIdSchema.optional(),
  archivePath: z.string().min(1).optional(),
  entryPath: z.string().min(1).optional(),
});

export type ItemSourceKind = z.infer<typeof itemSourceKindSchema>;
export type ItemsListQuery = z.infer<typeof itemsListQuerySchema>;
export type ItemListEntry = z.infer<typeof itemListEntrySchema>;
export type ItemDetail = z.infer<typeof itemDetailSchema>;
