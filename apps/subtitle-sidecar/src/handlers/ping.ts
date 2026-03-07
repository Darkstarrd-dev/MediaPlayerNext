import { protocolVersion, subtitleServiceName } from "../protocol.js";

export function handlePing() {
  return {
    service: subtitleServiceName,
    protocolVersion: protocolVersion,
    transport: "stdio",
  };
}
