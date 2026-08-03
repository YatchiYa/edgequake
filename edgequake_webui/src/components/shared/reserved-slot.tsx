/**
 * SPEC-100 — Reserved geometry wrapper for async chrome (CLS).
 *
 * Always mounts a min-height floor; shows `skeleton` while reserved, else children.
 */
"use client";

import { cn } from "@/lib/utils";
import type { CSSProperties, ReactNode } from "react";

export interface ReservedSlotProps {
  /** Stable floor in px (or CSS length via style override). */
  minHeightPx: number;
  /** When true, show skeleton instead of children. */
  reserved?: boolean;
  skeleton?: ReactNode;
  children?: ReactNode;
  className?: string;
  style?: CSSProperties;
  "data-testid"?: string;
}

export function ReservedSlot({
  minHeightPx,
  reserved = false,
  skeleton,
  children,
  className,
  style,
  "data-testid": testId = "spec100-reserved-slot",
}: ReservedSlotProps) {
  return (
    <div
      className={cn("min-w-0", className)}
      style={{ minHeight: minHeightPx, ...style }}
      data-testid={testId}
      data-reserved={reserved ? "true" : "false"}
    >
      {reserved ? skeleton : children}
    </div>
  );
}

export default ReservedSlot;
