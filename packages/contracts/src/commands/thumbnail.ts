import { z } from "zod";
import { assetIdSchema, thumbnailKeySchema } from "../models/ids.js";
import { thumbnailProfileSchema } from "../models/thumbnail.js";

export const thumbnailEnsureRequestSchema = z.object({
  assetId: assetIdSchema,
  profile: thumbnailProfileSchema,
});

export const thumbnailGetRequestSchema = z.object({
  thumbnailKey: thumbnailKeySchema,
});
