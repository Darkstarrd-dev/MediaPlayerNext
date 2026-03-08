import { z } from "zod";
import { appErrorSchema } from "../errors/app-error.js";
import { libraryIdSchema, taskIdSchema } from "../models/ids.js";

export const scanFinishedEventSchema = z.object({
  type: z.literal("scan.finished"),
  libraryId: libraryIdSchema,
  taskId: taskIdSchema,
});

export const scanFailedEventSchema = z.object({
  type: z.literal("scan.failed"),
  libraryId: libraryIdSchema,
  taskId: taskIdSchema,
  error: appErrorSchema,
});
