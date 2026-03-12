import { z } from "zod";
import { assetIdSchema } from "../models/ids.js";
import { itemsListQuerySchema, itemsListResultSchema } from "../models/items.js";

export const itemsListRequestSchema = itemsListQuerySchema;
export const itemsListResponseSchema = itemsListResultSchema;

export const itemDetailRequestSchema = z.object({
  assetId: assetIdSchema,
});
