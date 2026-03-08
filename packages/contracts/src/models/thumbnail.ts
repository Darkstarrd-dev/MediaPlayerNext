import { z } from "zod";
import { assetIdSchema, taskIdSchema, thumbnailKeySchema } from "./ids.js";
import { taskStateSchema } from "./tasks.js";

export const thumbnailProfileSchema = z.enum(["grid-sm", "grid-md", "detail-md", "detail-lg"]);

export const thumbnailEnsureResultSchema = z.object({
  assetId: assetIdSchema,
  thumbnailKey: thumbnailKeySchema,
  profile: thumbnailProfileSchema,
  width: z.number().int().positive(),
  height: z.number().int().positive(),
  format: z.string().min(1),
  diskPath: z.string().min(1),
  byteSize: z.number().int().nonnegative(),
  state: z.string().min(1),
  cacheHit: z.boolean(),
});

export const thumbnailProgressEventSchema = z.object({
  taskId: taskIdSchema,
  assetId: assetIdSchema,
  state: taskStateSchema,
  profile: thumbnailProfileSchema,
  thumbnailKey: thumbnailKeySchema.optional(),
  message: z.string().min(1).optional(),
});

export type ThumbnailProfile = z.infer<typeof thumbnailProfileSchema>;
export type ThumbnailEnsureResult = z.infer<typeof thumbnailEnsureResultSchema>;
export type ThumbnailProgressEvent = z.infer<typeof thumbnailProgressEventSchema>;
