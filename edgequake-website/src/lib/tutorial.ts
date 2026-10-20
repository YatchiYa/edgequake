/**
 * Visual product tour chapter index (published at /tutorial/).
 */

export type TutorialChapter = {
  slug: string;
  title: string;
  summary: string;
  /** Filename under public/tutorial/ for the chapter hero image (optional). */
  image?: string;
};

export const TUTORIAL_CHAPTERS: TutorialChapter[] = [
  {
    slug: "01-what-this-is",
    title: "What this is",
    summary: "What the product is for, before you click anything.",
  },
  {
    slug: "02-sign-in-and-getting-around",
    title: "Sign in and getting around",
    summary: "The sign-in card, the sidebar, and the organization switcher.",
    image: "02-sign-in.png",
  },
  {
    slug: "03-dashboard",
    title: "Dashboard",
    summary: "Home counts and the three shortcuts.",
    image: "03-dashboard.png",
  },
  {
    slug: "04-documents",
    title: "Documents",
    summary: "The file list and the drop zone.",
    image: "04-documents.png",
  },
  {
    slug: "05-one-document",
    title: "One document",
    summary: "Preview, extracted text, and where each fact came from.",
    image: "05-document.png",
  },
  {
    slug: "06-ask-a-question",
    title: "Ask a question",
    summary: "The question box, six search modes, and answers with sources.",
    image: "06-answer.png",
  },
  {
    slug: "07-the-map",
    title: "The map",
    summary: "The knowledge graph, and what you see when you pick one dot.",
    image: "07-graph.png",
  },
  {
    slug: "08-extra-knowledge",
    title: "Extra knowledge",
    summary: "Glossaries you add by hand.",
    image: "08-knowledge.png",
  },
  {
    slug: "09-workspaces",
    title: "Workspaces",
    summary: "Which model reads the files, and rebuild or delete.",
    image: "09-workspace.png",
  },
  {
    slug: "10-pipeline",
    title: "Pipeline",
    summary: "The queue that turns files into the map.",
    image: "10-pipeline.png",
  },
  {
    slug: "11-costs",
    title: "Costs",
    summary: "What the model calls have cost.",
    image: "11-costs.png",
  },
  {
    slug: "12-api-explorer",
    title: "API Explorer",
    summary: "The same product, as web requests a program can send.",
    image: "12-api-explorer.png",
  },
  {
    slug: "13-settings",
    title: "Settings",
    summary: "Theme, language, connections, and how answers behave.",
    image: "13-settings.png",
  },
  {
    slug: "14-members-and-invites",
    title: "Members and invites",
    summary: "Invite teammates, manage roles, and issue one-time sign-in links.",
    image: "14-members.png",
  },
];

export function chapterIndex(slug: string): number {
  return TUTORIAL_CHAPTERS.findIndex((c) => c.slug === slug);
}

export function adjacentChapters(slug: string): {
  prev?: TutorialChapter;
  next?: TutorialChapter;
} {
  const i = chapterIndex(slug);
  if (i < 0) return {};
  return {
    prev: i > 0 ? TUTORIAL_CHAPTERS[i - 1] : undefined,
    next: i < TUTORIAL_CHAPTERS.length - 1 ? TUTORIAL_CHAPTERS[i + 1] : undefined,
  };
}
