import { z } from "zod";
import { libraryIdSchema } from "../models/ids.js";
import { librarySummarySchema } from "../models/library.js";

export const libraryChangedEventSchema = z.object({
  type: z.literal("library.changed"),
  library: librarySummarySchema,
});

export const libraryRemovedEventSchema = z.object({
  type: z.literal("library.removed"),
  libraryId: libraryIdSchema,
});
