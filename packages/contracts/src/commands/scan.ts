import { z } from "zod";
import { libraryIdSchema } from "../models/ids.js";

export const scanStartRequestSchema = z.object({
  libraryId: libraryIdSchema,
});

export const scanResumeRequestSchema = z.object({
  libraryId: libraryIdSchema,
});

export const scanStatsRequestSchema = z.object({
  libraryId: libraryIdSchema,
});

export const scanSnapshotRequestSchema = z.object({
  libraryId: libraryIdSchema,
});
