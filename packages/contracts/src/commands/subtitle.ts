import { z } from "zod";
import { assetIdSchema, subtitleSessionIdSchema } from "../models/ids.js";

export const subtitlePingRequestSchema = z.object({});

export const subtitleHealthRequestSchema = z.object({});

export const subtitleStartSessionRequestSchema = z.object({
  assetId: assetIdSchema.optional(),
});

export const subtitleStopSessionRequestSchema = z.object({
  sessionId: subtitleSessionIdSchema,
});

export const subtitleGetProgressRequestSchema = z.object({
  sessionId: subtitleSessionIdSchema,
});

export const subtitleShutdownRequestSchema = z.object({});
