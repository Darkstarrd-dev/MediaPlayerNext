import { z } from "zod";
import { assetIdSchema, playbackSessionIdSchema } from "./ids.js";

export const mediaProbeSchema = z.object({
  mime: z.string(),
  containerFormat: z.string().optional(),
  durationMs: z.number().int().optional(),
  width: z.number().int().positive().optional(),
  height: z.number().int().positive().optional(),
  videoCodec: z.string().optional(),
  audioCodec: z.string().optional(),
});

export const mediaUrlSchema = z.object({
  assetId: assetIdSchema,
  url: z.string().min(1),
  mime: z.string(),
});

export const playbackSessionStateSchema = z.enum([
  "opening",
  "paused",
  "playing",
  "stopped",
  "failed",
]);

export const playbackSessionSchema = z.object({
  sessionId: playbackSessionIdSchema,
  assetId: assetIdSchema,
  mediaUrl: z.string().min(1),
  state: playbackSessionStateSchema,
  positionMs: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export type MediaProbe = z.infer<typeof mediaProbeSchema>;
export type MediaUrl = z.infer<typeof mediaUrlSchema>;
export type PlaybackSessionState = z.infer<typeof playbackSessionStateSchema>;
export type PlaybackSession = z.infer<typeof playbackSessionSchema>;
