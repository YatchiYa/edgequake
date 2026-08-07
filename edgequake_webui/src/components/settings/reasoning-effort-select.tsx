"use client";

/**
 * SPEC-109: shared reasoning-effort select (Auto + capability-filtered values).
 * Auto displays the effective / best-practice value so operators know what runs.
 */

import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  formatAutoEffortLabel,
  formatEffectiveBestPracticeHint,
} from "@/lib/settings/reasoning-effort-supported";
import { cn } from "@/lib/utils";

export const REASONING_EFFORT_VALUES = [
  "none",
  "minimal",
  "low",
  "medium",
  "high",
  "xhigh",
  "max",
] as const;

export type ReasoningEffortValue = (typeof REASONING_EFFORT_VALUES)[number];

const AUTO_VALUE = "__auto__";

export interface ReasoningEffortSelectProps {
  /** Current value; undefined/null/empty = Auto */
  value?: string | null;
  onChange: (value: string | undefined) => void;
  /** When set, only these efforts are offered (plus Auto). */
  supported?: string[];
  /**
   * Effective effort when Auto is chosen (catalog / role policy).
   * Shown on the Auto option and as a best-practice hint.
   * Use `"omit"` when the wire field is not sent.
   */
  effectiveWhenAuto?: string | null;
  disabled?: boolean;
  id?: string;
  label?: string;
  /** Hide the field label (pair with a shared row label / peer control). */
  hideLabel?: boolean;
  /** Hide the Auto best-practice hint (keeps dense toolbars aligned). */
  hideHint?: boolean;
  /**
   * Compact Auto trigger text ("Auto (inherit)") for dense toolbars.
   * Full effective wording stays on the Auto option + title tooltip.
   */
  compactTrigger?: boolean;
  /** Extra classes for the select trigger (height/width pairing with peers). */
  triggerClassName?: string;
  className?: string;
  "data-testid"?: string;
}

export function ReasoningEffortSelect({
  value,
  onChange,
  supported,
  effectiveWhenAuto,
  disabled,
  id = "reasoning-effort",
  label = "Reasoning effort",
  hideLabel = false,
  hideHint = false,
  compactTrigger = false,
  triggerClassName,
  className,
  "data-testid": testId = "reasoning-effort-select",
}: ReasoningEffortSelectProps) {
  const options =
    supported && supported.length > 0
      ? REASONING_EFFORT_VALUES.filter((v) =>
          supported.some((s) => s.toLowerCase() === v),
        )
      : [...REASONING_EFFORT_VALUES];

  const selectValue = value && value.trim() ? value.trim() : AUTO_VALUE;
  const isAuto = selectValue === AUTO_VALUE;
  const effective =
    effectiveWhenAuto && effectiveWhenAuto.trim()
      ? effectiveWhenAuto.trim()
      : "omit";
  const autoLabel = formatAutoEffortLabel(effective);
  const triggerAutoLabel = compactTrigger ? "Auto (inherit)" : autoLabel;
  const hint = formatEffectiveBestPracticeHint(effective);

  return (
    <div className={className ?? "space-y-2"} data-testid={testId}>
      {!hideLabel && (
        <Label htmlFor={id} className="text-sm font-medium">
          {label}
        </Label>
      )}
      <Select
        disabled={disabled}
        value={selectValue}
        onValueChange={(v) => {
          if (v === AUTO_VALUE) {
            onChange(undefined);
          } else {
            onChange(v);
          }
        }}
      >
        <SelectTrigger
          id={id}
          className={cn("w-full", triggerClassName)}
          data-testid={`${testId}-trigger`}
          title={isAuto && (hideHint || compactTrigger) ? hint : undefined}
        >
          {compactTrigger && isAuto ? (
            <SelectValue placeholder="Auto">{triggerAutoLabel}</SelectValue>
          ) : (
            <SelectValue placeholder="Auto" />
          )}
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={AUTO_VALUE} data-testid={`${testId}-auto-option`}>
            {autoLabel}
          </SelectItem>
          {options.map((effort) => (
            <SelectItem key={effort} value={effort}>
              {effort}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {isAuto && !hideHint && (
        <p
          className="text-xs text-muted-foreground leading-snug"
          data-testid={`${testId}-effective-hint`}
        >
          {hint}
        </p>
      )}
    </div>
  );
}
