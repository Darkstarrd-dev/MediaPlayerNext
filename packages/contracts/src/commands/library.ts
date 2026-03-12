import { z } from "zod";
import { libraryIdSchema } from "../models/ids.js";
import { addLibraryInputSchema } from "../models/library.js";

export const libraryListRequestSchema = z.object({});

export const libraryAddRequestSchema = addLibraryInputSchema;

export const libraryGetRequestSchema = z.object({
  libraryId: libraryIdSchema,
});

export const libraryRemoveRequestSchema = z.object({
  libraryId: libraryIdSchema,
});

export const libraryNodesRequestSchema = z.object({
  libraryId: libraryIdSchema,
});
