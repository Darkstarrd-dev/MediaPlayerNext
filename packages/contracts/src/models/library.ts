import { z } from "zod";
import { libraryIdSchema, mediaSourceIdSchema } from "./ids.js";

export const librarySummarySchema = z.object({
  id: libraryIdSchema,
  rootPath: z.string().min(1),
  libraryType: z.string().min(1),
  scanMode: z.string().min(1),
  createdAt: z.string().min(1),
  updatedAt: z.string().min(1),
});

export const libraryDetailSchema = librarySummarySchema;

export const addLibraryInputSchema = z.object({
  rootPath: z.string().min(1),
});

export const sidebarNodeTypeSchema = z.enum(["folder", "media_source"]);

export const sidebarSourceTypeSchema = z.enum(["package", "directory"]);

export const sidebarNodeSummarySchema = z.object({
  nodeId: z.string().min(1),
  libraryId: libraryIdSchema,
  label: z.string().min(1),
  nodeType: sidebarNodeTypeSchema,
  parentNodeId: z.string().min(1).optional(),
  treePath: z.array(z.string().min(1)),
  depth: z.number().int().nonnegative(),
  mediaSourceId: mediaSourceIdSchema.optional(),
  sourceType: sidebarSourceTypeSchema.optional(),
  itemCount: z.number().int().nonnegative().optional(),
  hasDirectMediaChild: z.boolean(),
  kind: z.string().min(1),
});

export type LibrarySummary = z.infer<typeof librarySummarySchema>;
export type LibraryDetail = z.infer<typeof libraryDetailSchema>;
export type AddLibraryInput = z.infer<typeof addLibraryInputSchema>;
export type SidebarNodeSummary = z.infer<typeof sidebarNodeSummarySchema>;
