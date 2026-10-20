/**
 * Build-time GitHub stats for the marketing site.
 * Fetches once per build; falls back to a known floor if the API is unavailable.
 */

const REPO = "raphaelmansuy/edgequake";
/** Floor when GitHub API is rate-limited or offline (live count was 2275 on 2026-10-20). */
export const GITHUB_STARS_FALLBACK = 2275;

export type GithubStats = {
  stars: number;
  forks: number;
  source: "api" | "fallback";
};

export function formatStars(count: number): string {
  return new Intl.NumberFormat("en-US").format(count);
}

export async function getGithubStats(): Promise<GithubStats> {
  try {
    const headers: HeadersInit = {
      Accept: "application/vnd.github+json",
      "User-Agent": "edgequake-website",
      "X-GitHub-Api-Version": "2022-11-28",
    };
    const token =
      import.meta.env.GITHUB_TOKEN ??
      (typeof process !== "undefined" ? process.env.GITHUB_TOKEN : undefined);
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }

    const res = await fetch(`https://api.github.com/repos/${REPO}`, {
      headers,
    });
    if (!res.ok) {
      console.warn(
        `[github] API ${res.status} for ${REPO}; using fallback stars`,
      );
      return {
        stars: GITHUB_STARS_FALLBACK,
        forks: 0,
        source: "fallback",
      };
    }
    const data = (await res.json()) as {
      stargazers_count?: number;
      forks_count?: number;
    };
    const stars =
      typeof data.stargazers_count === "number"
        ? data.stargazers_count
        : GITHUB_STARS_FALLBACK;
    return {
      stars: Math.max(stars, GITHUB_STARS_FALLBACK),
      forks: typeof data.forks_count === "number" ? data.forks_count : 0,
      source: "api",
    };
  } catch (err) {
    console.warn("[github] fetch failed; using fallback stars", err);
    return {
      stars: GITHUB_STARS_FALLBACK,
      forks: 0,
      source: "fallback",
    };
  }
}
