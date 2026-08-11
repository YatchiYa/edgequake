"use client";

import { BookOpen, GitBranch, Lightbulb, MessageSquare, Search, Sparkles } from "lucide-react";
import { memo } from "react";
import { useTranslation } from "react-i18next";

import {
  getQueryEmptyCopy,
  isChatQueryMode,
} from "@/lib/query/query-empty-copy";
import type { QueryMode } from "@/types/query";

export interface QueryEmptyStateProps {
  onSuggestionClick?: (text: string) => void;
  graphStats?: { entities: number; relationships: number; types: number };
  /** Active query mode — Chat (bypass) uses chatbot copy, not KG copy. */
  mode?: QueryMode;
}

/** Empty query chat state with suggestions and optional graph stats (SPEC-017 UI-P3-005). */
export const QueryEmptyState = memo(function QueryEmptyState({
  onSuggestionClick,
  graphStats,
  mode = "mix",
}: QueryEmptyStateProps) {
  const { t } = useTranslation();
  const isChat = isChatQueryMode(mode);
  const copy = getQueryEmptyCopy(mode);

  const suggestionIcons = isChat
    ? [
        <MessageSquare key="0" className="h-4 w-4" />,
        <Lightbulb key="1" className="h-4 w-4" />,
        <Search key="2" className="h-4 w-4" />,
        <BookOpen key="3" className="h-4 w-4" />,
      ]
    : [
        <Search key="0" className="h-4 w-4" />,
        <Lightbulb key="1" className="h-4 w-4" />,
        <GitBranch key="2" className="h-4 w-4" />,
        <BookOpen key="3" className="h-4 w-4" />,
      ];

  const suggestions = copy.suggestions.map((text, i) => ({
    icon: suggestionIcons[i] ?? <Search className="h-4 w-4" />,
    text: isChat
      ? t(`query.chatSuggestions.${i}`, text)
      : t(`query.suggestions.${i}`, text),
  }));

  const hasData =
    !isChat &&
    graphStats &&
    (graphStats.entities > 0 || graphStats.relationships > 0);

  return (
    <div className="flex flex-col items-center justify-center h-full py-12 px-4 motion-safe:animate-fade-in-up">
      <div className="relative mb-8" aria-hidden="true">
        <div className="absolute inset-0 bg-gradient-to-r from-primary/40 to-primary/60 rounded-2xl blur-2xl opacity-20 motion-safe:animate-pulse-soft" />
        <div className="relative bg-gradient-to-br from-primary/80 to-primary rounded-2xl p-5 shadow-lg">
          {isChat ? (
            <MessageSquare className="h-10 w-10 text-primary-foreground" />
          ) : (
            <Sparkles className="h-10 w-10 text-primary-foreground" />
          )}
        </div>
      </div>

      <h2 className="text-2xl font-bold mb-2 text-center">
        {isChat
          ? t("query.chatEmptyTitle", copy.title)
          : t("query.emptyTitle", copy.title)}
      </h2>
      <p className="text-muted-foreground text-center mb-8 max-w-lg leading-relaxed">
        {isChat
          ? t("query.chatEmptyDescription", copy.description)
          : t("query.emptyDescription", copy.description)}
      </p>

      {hasData && (
        <div
          className="flex items-center gap-4 mb-8 px-6 py-3 bg-muted/30 rounded-full border border-border/50"
          role="status"
          aria-label={`${graphStats.entities} entities, ${graphStats.relationships} relationships, ${graphStats.types} types`}
        >
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-green-500" aria-hidden="true" />
            <span className="text-sm font-medium">{graphStats.entities}</span>
            <span className="text-xs text-muted-foreground">entities</span>
          </div>
          <div className="w-px h-4 bg-border" aria-hidden="true" />
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-amber-500" aria-hidden="true" />
            <span className="text-sm font-medium">{graphStats.relationships}</span>
            <span className="text-xs text-muted-foreground">relationships</span>
          </div>
          <div className="w-px h-4 bg-border" aria-hidden="true" />
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-blue-500" aria-hidden="true" />
            <span className="text-sm font-medium">{graphStats.types}</span>
            <span className="text-xs text-muted-foreground">types</span>
          </div>
        </div>
      )}

      {onSuggestionClick && (
        <div className="w-full max-w-2xl space-y-3">
          <p className="text-sm font-medium text-muted-foreground text-center mb-3">
            {t("query.tryAsking", "Try asking:")}
          </p>
          <div
            className="grid grid-cols-1 md:grid-cols-2 gap-2"
            role="list"
            aria-label={t("query.suggestedQueries", "Suggested queries")}
          >
            {suggestions.map((suggestion, i) => (
              <button
                key={i}
                onClick={() => onSuggestionClick(suggestion.text)}
                className="group flex items-start gap-3 text-left px-4 py-3.5 rounded-xl border bg-card hover:bg-muted/50 hover:border-primary/30 transition-all duration-200 hover:shadow-sm hover:-translate-y-0.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                role="listitem"
                aria-label={suggestion.text}
              >
                <div
                  className="p-1.5 rounded-lg bg-muted group-hover:bg-primary/10 transition-colors shrink-0"
                  aria-hidden="true"
                >
                  {suggestion.icon}
                </div>
                <span className="text-sm leading-relaxed">{suggestion.text}</span>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
});
