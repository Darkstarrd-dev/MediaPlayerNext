import { z } from "zod";
import { libraryIdSchema, sourceIdSchema } from "./ids.js";

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

export const sidebarNodeKindSchema = z.enum([
  "image",
  "video",
  "archive",
  "audio",
  "other",
]);

export const sidebarNodeSummarySchema = z.object({
  nodeId: z.string().min(1),
  libraryId: libraryIdSchema,
  sourceId: sourceIdSchema,
  label: z.string().min(1),
  normalizedPath: z.string().min(1),
  kind: sidebarNodeKindSchema,
});

export type LibrarySummary = z.infer<typeof librarySummarySchema>;
export type LibraryDetail = z.infer<typeof libraryDetailSchema>;
export type AddLibraryInput = z.infer<typeof addLibraryInputSchema>;
export type SidebarNodeSummary = z.infer<typeof sidebarNodeSummarySchema>;
