/**
 * SPEC-020 query UI helpers — DRY orchestration for hybrid query proofs.
 */
import { expect, type Page } from "@playwright/test";
import { GOTO_OPTS, waitForStreamingComplete } from "./app-ready";

const ASSISTANT_SELECTOR =
  'article[aria-label="Assistant response"], .message-assistant, [data-testid="assistant-message"], .prose';

export async function gotoQueryPage(page: Page): Promise<void> {
  await page.goto("/query", GOTO_OPTS);
  await expect(page.locator("textarea.query-input").first()).toBeVisible({
    timeout: 20_000,
  });
}

export async function submitQueryAndWait(
  page: Page,
  question: string,
  options?: { processingTimeoutMs?: number; answerTimeoutMs?: number },
): Promise<string> {
  const input = page.locator("textarea.query-input").first();
  await input.fill(question);
  await input.press("Enter");

  const processing = page.getByText(/Processing your query/i);
  if (await processing.isVisible({ timeout: 10_000 }).catch(() => false)) {
    await expect(processing).toBeHidden({
      timeout: options?.processingTimeoutMs ?? 180_000,
    });
  }

  const assistant = page.locator(ASSISTANT_SELECTOR).last();
  await expect(assistant).toBeVisible({
    timeout: options?.answerTimeoutMs ?? 120_000,
  });
  return (await assistant.textContent()) ?? "";
}

export function isGroundedSarahChenAnswer(text: string): boolean {
  return /Sarah Chen|senior engineer|EDGEQUAKE/i.test(text);
}

export function isMockProviderAnswer(text: string): boolean {
  return /Mock response/i.test(text);
}

/** Live LLM answer acceptable when grounded OR substantive with extracted entities. */
export function isAcceptableLiveLlmAnswer(
  text: string,
  entityCount: number,
): boolean {
  if (isMockProviderAnswer(text) || text.length < 30) return false;
  if (isGroundedSarahChenAnswer(text)) return true;
  return (
    entityCount > 0 &&
    /context|knowledge|engineer|EDGEQUAKE|overview|extract|GraphRAG/i.test(text)
  );
}

export async function assertSourceCitationsVisible(page: Page): Promise<void> {
  await expect(
    page.getByRole("button", { name: /Source citations/i }),
  ).toBeVisible({ timeout: 15_000 });
}

/** Query on empty workspace must not crash (no ingested docs). */
export async function assertQueryOnEmptyWorkspaceSafe(page: Page): Promise<string> {
  await gotoQueryPage(page);
  const answerText = await submitQueryAndWait(
    page,
    "What is EDGEQUAKE?",
    { answerTimeoutMs: 120_000, processingTimeoutMs: 120_000 },
  );
  expect(answerText.length).toBeGreaterThan(0);
  const html = (await page.content()).toLowerCase();
  expect(html).not.toContain("application error");
  return answerText;
}

/** Empty query must not crash; input remains usable. */
export async function assertEmptyQuerySafe(page: Page): Promise<void> {
  const input = page.locator("textarea.query-input").first();
  await input.fill("");
  await input.press("Enter");
  await expect(input).toBeVisible({ timeout: 5_000 });
  const html = (await page.content()).toLowerCase();
  expect(html).not.toContain("application error");
}

/** After query, streaming must complete (textarea re-enabled). */
export async function assertStreamingCompleted(page: Page): Promise<void> {
  await waitForStreamingComplete(page, 120_000);
  const input = page.locator("textarea.query-input").first();
  await expect(input).toBeEnabled({ timeout: 15_000 });
}

export async function openSourceCitationsPanel(page: Page): Promise<void> {
  const btn = page.getByRole("button", { name: /Source citations/i });
  await btn.click();
  await expect(
    page
      .locator('[data-testid="source-citations"]')
      .or(page.getByText(/\d+\s+Sources?/i))
      .first(),
  ).toBeVisible({ timeout: 10_000 });
}
