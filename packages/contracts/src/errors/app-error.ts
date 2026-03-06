import { z } from "zod";

export const appErrorCodeSchema = z.enum([
  "NOT_FOUND",
  "ALREADY_EXISTS",
  "UNSUPPORTED_FORMAT",
  "PERMISSION_DENIED",
  "INVALID_ARGUMENT",
  "IO_ERROR",
  "DB_ERROR",
  "EXTERNAL_TOOL_ERROR",
  "CANCELLED",
  "TIMEOUT",
  "INTERNAL_ERROR",
]);

export const appErrorSchema = z.object({
  code: appErrorCodeSchema,
  message: z.string(),
  retriable: z.boolean(),
  details: z.record(z.string(), z.unknown()).nullable().optional(),
});

export type AppErrorCode = z.infer<typeof appErrorCodeSchema>;
export type AppError = z.infer<typeof appErrorSchema>;
