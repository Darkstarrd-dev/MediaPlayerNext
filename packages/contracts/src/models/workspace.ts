import { z } from "zod";
import { assetIdSchema, libraryIdSchema, sourceIdSchema } from "./ids.js";

export const workspaceCursorSchema = z.object({
  selectedLibraryId: libraryIdSchema.nullable().optional(),
  selectedNodeId: sourceIdSchema.nullable().optional(),
  itemsPageIndex: z.number().int().positive().nullable().optional(),
  selectedAssetId: assetIdSchema.nullable().optional(),
});

export type WorkspaceCursor = z.infer<typeof workspaceCursorSchema>;
