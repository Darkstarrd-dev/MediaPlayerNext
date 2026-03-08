import { z } from "zod";
import { taskProgressSchema } from "../models/tasks.js";

export const scanProgressEventSchema = taskProgressSchema.extend({
  taskType: z.literal("scan"),
});

export type ScanProgressEvent = z.infer<typeof scanProgressEventSchema>;
