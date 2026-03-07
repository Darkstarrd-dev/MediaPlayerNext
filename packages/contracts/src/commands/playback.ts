import { z } from "zod";
import { assetIdSchema, playbackSessionIdSchema } from "../models/ids.js";

export const playbackProbeRequestSchema = z.object({
  assetId: assetIdSchema,
});

export const playbackOpenRequestSchema = z.object({
  assetId: assetIdSchema,
});

export const playbackStatusRequestSchema = z.object({
  sessionId: playbackSessionIdSchema,
});

export const playbackSeekRequestSchema = z.object({
  sessionId: playbackSessionIdSchema,
  positionMs: z.number().int().nonnegative(),
});
