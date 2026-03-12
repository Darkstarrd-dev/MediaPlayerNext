import { z } from "zod";
import { assetIdSchema, libraryIdSchema, mediaSourceIdSchema } from "./ids.js";

export const workspaceCursorSchema = z.object({
  selectedLibraryId: libraryIdSchema.nullable().optional(),
  selectedSidebarNodeId: z.string().min(1).nullable().optional(),
  selectedMediaSourceId: mediaSourceIdSchema.nullable().optional(),
  selectedNodeId: z.string().min(1).nullable().optional(),
  itemsPageIndex: z.number().int().positive().nullable().optional(),
  selectedAssetId: assetIdSchema.nullable().optional(),
});

export type WorkspaceCursor = z.infer<typeof workspaceCursorSchema>;
