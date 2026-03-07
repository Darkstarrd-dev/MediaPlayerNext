import { z } from "zod";

export const ffmpegProgressEventSchema = z.object({
  frame: z.number().int().nonnegative().optional(),
  fps: z.number().nonnegative().optional(),
  totalSize: z.number().int().nonnegative().optional(),
  outTimeMs: z.number().int().nonnegative().optional(),
  speed: z.number().nonnegative().optional(),
  progress: z.string(),
});

export type FfmpegProgressEvent = z.infer<typeof ffmpegProgressEventSchema>;
