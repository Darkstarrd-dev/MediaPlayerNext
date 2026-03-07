import { z } from "zod";
import { subtitleProgressSchema } from "../models/subtitle.js";

export const subtitleProgressEventSchema = subtitleProgressSchema;

export type SubtitleProgressEvent = z.infer<typeof subtitleProgressEventSchema>;
