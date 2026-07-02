import { describe, expect, it } from "vitest";

import {
  isConversationGoneError,
  isConversationGoneStreamCode,
  isConversationNotFoundError,
} from "../conversation-errors";
import { ApiRequestError } from "@/lib/api/client";

describe("conversation-errors", () => {
  it("detects 404 conversation errors", () => {
    expect(isConversationNotFoundError(new ApiRequestError("missing", 404))).toBe(
      true,
    );
  });

  it("detects CONVERSATION_GONE stream code", () => {
    expect(isConversationGoneStreamCode("CONVERSATION_GONE")).toBe(true);
    expect(isConversationGoneStreamCode("SAVE_FAILED")).toBe(false);
  });

  it("detects conversation gone message text", () => {
    expect(
      isConversationGoneError(new Error("Conversation no longer exists")),
    ).toBe(true);
  });
});
