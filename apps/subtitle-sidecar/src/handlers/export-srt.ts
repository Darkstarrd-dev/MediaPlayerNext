import type { AppError } from "../protocol.js";

export function handleExportSrt(): AppError {
  return {
    code: "UNSUPPORTED_FORMAT",
    message: "export_srt is not implemented in B8",
    retriable: false,
  };
}
