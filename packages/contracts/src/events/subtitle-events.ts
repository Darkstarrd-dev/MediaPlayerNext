import { z } from "zod";
import { subtitleHostSchema, subtitleSessionSchema } from "../models/subtitle.js";

export const subtitleSidecarCrashedEventSchema = z.object({
  type: z.literal("subtitle.sidecar.crashed"),
  host: subtitleHostSchema,
});

export const subtitleSessionUpdatedEventSchema = z.object({
  type: z.literal("subtitle.session.updated"),
  session: subtitleSessionSchema,
});
