import { z } from "zod";
import { assetIdSchema } from "../models/ids.js";
import { itemsListQuerySchema } from "../models/items.js";

export const itemsListRequestSchema = itemsListQuerySchema;

export const itemDetailRequestSchema = z.object({
  assetId: assetIdSchema,
});
