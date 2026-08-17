import { describe, expect, test } from "bun:test";
import {
  pipelineSuggestsLiveWork,
  shouldReserveFeedbackSlot,
} from "../documents-layout-stability";

describe("documents-layout-stability", () => {
  test("pipelineSuggestsLiveWork reads busy / running / queued", () => {
    expect(pipelineSuggestsLiveWork(undefined)).toBe(false);
    expect(pipelineSuggestsLiveWork({ is_busy: true } as never)).toBe(true);
    expect(
      pipelineSuggestsLiveWork({
        is_busy: false,
        running_tasks: 1,
        pending_tasks: 0,
      } as never),
    ).toBe(true);
    expect(
      pipelineSuggestsLiveWork({
        is_busy: false,
        running_tasks: 0,
        queued_tasks: 2,
      } as never),
    ).toBe(true);
    expect(
      pipelineSuggestsLiveWork({
        is_busy: false,
        running_tasks: 0,
        pending_tasks: 0,
        queued_tasks: 0,
        processing_tasks: 0,
      } as never),
    ).toBe(false);
  });

  test("shouldReserveFeedbackSlot only on cold load when busy/hint", () => {
    expect(
      shouldReserveFeedbackSlot({
        hasLiveWork: true,
        isInitialLoading: true,
        pipelineStatus: { is_busy: true } as never,
        liveWorkHint: true,
      }),
    ).toBe(false);

    expect(
      shouldReserveFeedbackSlot({
        hasLiveWork: false,
        isInitialLoading: false,
        pipelineStatus: { is_busy: true } as never,
        liveWorkHint: true,
      }),
    ).toBe(false);

    expect(
      shouldReserveFeedbackSlot({
        hasLiveWork: false,
        isInitialLoading: true,
        pipelineStatus: { is_busy: true } as never,
        liveWorkHint: false,
      }),
    ).toBe(true);

    expect(
      shouldReserveFeedbackSlot({
        hasLiveWork: false,
        isInitialLoading: true,
        pipelineStatus: undefined,
        liveWorkHint: true,
      }),
    ).toBe(true);

    expect(
      shouldReserveFeedbackSlot({
        hasLiveWork: false,
        isInitialLoading: true,
        pipelineStatus: undefined,
        liveWorkHint: false,
      }),
    ).toBe(false);
  });
});
