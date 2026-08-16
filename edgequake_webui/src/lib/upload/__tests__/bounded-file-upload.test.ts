import { describe, expect, it } from "vitest";

import {
  createBoundedExecutor,
  createUploadId,
  fileUploadFingerprint,
  perFileUploadErrorMessage,
  updateByUploadId,
} from "../bounded-file-upload";

function fileLike(name: string, size: number, lastModified: number): File {
  return { name, size, lastModified } as File;
}

describe("bounded file upload coordinator", () => {
  it("never runs more than three tasks across overlapping selections", async () => {
    const executor = createBoundedExecutor(3);
    let active = 0;
    let maximumActive = 0;

    const run = (id: number) =>
      executor.run(async () => {
        active += 1;
        maximumActive = Math.max(maximumActive, active);
        await new Promise<void>((resolve) => setTimeout(resolve, 10));
        active -= 1;
        return id;
      });

    const firstSelection = [0, 1, 2].map(run);
    const secondSelection = [3, 4, 5].map(run);

    await expect(
      Promise.all([...firstSelection, ...secondSelection]),
    ).resolves.toEqual([0, 1, 2, 3, 4, 5]);
    expect(maximumActive).toBe(3);
  });

  it("attempts every queued task when one task fails", async () => {
    const executor = createBoundedExecutor(2);
    const attempted: number[] = [];

    const results = await Promise.allSettled(
      [0, 1, 2, 3].map((id) =>
        executor.run(async () => {
          attempted.push(id);
          if (id === 1) throw new Error("admission failed");
          return id;
        }),
      ),
    );

    expect(attempted.sort()).toEqual([0, 1, 2, 3]);
    expect(
      results.filter((result) => result.status === "rejected"),
    ).toHaveLength(1);
  });

  it("suppresses only an exact in-flight file fingerprint", () => {
    expect(fileUploadFingerprint(fileLike("a.md", 10, 100))).toBe(
      fileUploadFingerprint(fileLike("a.md", 10, 100)),
    );
    expect(fileUploadFingerprint(fileLike("a.md", 11, 100))).not.toBe(
      fileUploadFingerprint(fileLike("a.md", 10, 100)),
    );
    expect(fileUploadFingerprint(fileLike("b.md", 10, 100))).not.toBe(
      fileUploadFingerprint(fileLike("a.md", 10, 100)),
    );
  });

  it("updates out-of-order rows by stable upload identity", () => {
    const entries = [
      { uploadId: "first", progress: 0 },
      { uploadId: "second", progress: 0 },
    ];

    const secondDone = updateByUploadId(entries, "second", { progress: 100 });
    const firstHalf = updateByUploadId(secondDone, "first", { progress: 50 });

    expect(firstHalf).toEqual([
      { uploadId: "first", progress: 50 },
      { uploadId: "second", progress: 100 },
    ]);
  });

  it("creates a distinct client identity for every accepted row", () => {
    expect(createUploadId()).not.toBe(createUploadId());
  });

  it("SPEC-132: timeout/error releases the slot so queued files still run", async () => {
    const executor = createBoundedExecutor(1);
    const order: string[] = [];

    const hung = executor.run(async () => {
      order.push("hung-start");
      await new Promise<void>((_, reject) =>
        setTimeout(
          () => reject(new Error("Upload timed out after 50ms")),
          30,
        ),
      );
    });

    const next = executor.run(async () => {
      order.push("next");
      return "ok";
    });

    await expect(hung).rejects.toThrow(/timed out/i);
    await expect(next).resolves.toBe("ok");
    expect(order).toEqual(["hung-start", "next"]);
  });

  it("SPEC-132: per-file timeout copy mentions siblings continue", () => {
    expect(
      perFileUploadErrorMessage(
        new Error("Upload timed out after 60000ms"),
      ),
    ).toMatch(/other selected files continue/i);
  });
});
