import { z } from "zod";

const idSchema = z.string().min(1);

export const libraryIdSchema = idSchema;
export const sourceIdSchema = idSchema;
export const mediaSourceIdSchema = idSchema;
export const archiveIdSchema = idSchema;
export const archiveEntryIdSchema = idSchema;
export const assetIdSchema = idSchema;
export const thumbnailKeySchema = idSchema;
export const taskIdSchema = idSchema;
export const playbackSessionIdSchema = idSchema;
export const subtitleSessionIdSchema = idSchema;
