/**
 * Entity path encoding + delete confirm query (PR #407 / workspace-scoped ids).
 */
import { describe, expect, it, vi, beforeEach } from "vitest";

const deleteMock = vi.fn(async () => undefined);
const getMock = vi.fn(async () => ({}));
const putMock = vi.fn(async () => ({}));

vi.mock("../../client", () => ({
  api: {
    get: (...args: unknown[]) => getMock(...args),
    put: (...args: unknown[]) => putMock(...args),
    delete: (...args: unknown[]) => deleteMock(...args),
    post: vi.fn(),
  },
  streamClient: {},
}));

import {
  deleteEntity,
  entityPath,
  getEntity,
  getEntityNeighborhood,
  updateEntity,
} from "../graph";

describe("entityPath", () => {
  it("encodes workspace-scoped ids so :: survives the URL path", () => {
    const id = "79d6e213-032d-402c-9325-aee3483d3185::MARC_DUBOIS";
    expect(entityPath(id)).toBe(
      `/graph/entities/${encodeURIComponent(id)}`,
    );
    expect(entityPath(id)).toContain("%3A%3A");
  });
});

describe("deleteEntity", () => {
  beforeEach(() => {
    deleteMock.mockClear();
  });

  it("calls DELETE with encoded id and confirm=true", async () => {
    const id = "79d6e213-032d-402c-9325-aee3483d3185::MARC_DUBOIS";
    await deleteEntity(id);
    expect(deleteMock).toHaveBeenCalledWith(
      `/graph/entities/${encodeURIComponent(id)}?confirm=true`,
    );
  });
});

describe("get/update/neighborhood use entityPath", () => {
  beforeEach(() => {
    getMock.mockClear();
    putMock.mockClear();
  });

  it("getEntity encodes the id", async () => {
    const id = "ws::NAME";
    await getEntity(id);
    expect(getMock).toHaveBeenCalledWith(
      `/graph/entities/${encodeURIComponent(id)}`,
    );
  });

  it("updateEntity encodes the id", async () => {
    const id = "ws::NAME";
    await updateEntity(id, { entity_type: "PERSON" } as never);
    expect(putMock).toHaveBeenCalledWith(
      `/graph/entities/${encodeURIComponent(id)}`,
      { entity_type: "PERSON" },
    );
  });

  it("getEntityNeighborhood encodes the id", async () => {
    const id = "ws::NAME";
    await getEntityNeighborhood(id, 2);
    expect(getMock).toHaveBeenCalledWith(
      `/graph/entities/${encodeURIComponent(id)}/neighborhood?depth=2`,
    );
  });
});
