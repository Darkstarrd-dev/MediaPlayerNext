import { z } from "zod";
import { playbackSessionSchema } from "../models/playback.js";

export const playbackOpenedEventSchema = z.object({
  type: z.literal("playback.opened"),
  session: playbackSessionSchema,
});

export const playbackStoppedEventSchema = z.object({
  type: z.literal("playback.stopped"),
  session: playbackSessionSchema,
});
