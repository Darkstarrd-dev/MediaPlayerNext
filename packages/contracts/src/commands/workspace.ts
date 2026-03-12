import { z } from "zod";
import { workspaceCursorSchema } from "../models/workspace.js";

export const workspaceCursorReadRequestSchema = z.object({});

export const workspaceCursorWriteRequestSchema = z.object({
  cursor: workspaceCursorSchema,
});
