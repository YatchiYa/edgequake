/** SPEC-169 — small pure formatters for the Members screen. */

const PALETTE = [
  "bg-sky-500/15 text-sky-700 dark:text-sky-300",
  "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
  "bg-violet-500/15 text-violet-700 dark:text-violet-300",
  "bg-amber-500/15 text-amber-700 dark:text-amber-300",
  "bg-rose-500/15 text-rose-700 dark:text-rose-300",
  "bg-teal-500/15 text-teal-700 dark:text-teal-300",
  "bg-indigo-500/15 text-indigo-700 dark:text-indigo-300",
  "bg-orange-500/15 text-orange-700 dark:text-orange-300",
] as const;

function hash(text: string): number {
  let h = 0;
  for (let i = 0; i < text.length; i++) h = (h * 31 + text.charCodeAt(i)) >>> 0;
  return h;
}

/** Stable tint per person so the same face keeps the same colour everywhere. */
export function avatarTone(seed: string): string {
  return PALETTE[hash(seed.toLowerCase()) % PALETTE.length];
}

/** "Ada Lovelace" → "AL", "ada@x.io" → "A". */
export function initials(name: string): string {
  const base = name.includes("@") ? name.split("@")[0] : name;
  const parts = base.split(/[\s._-]+/).filter(Boolean);
  const letters = parts.length > 1 ? parts[0][0] + parts[parts.length - 1][0] : (parts[0] ?? "?").slice(0, 2);
  return letters.toUpperCase();
}

/** The label a row is known by: username, else the invited email. */
export function displayName(m: { username?: string | null; email: string }): string {
  return m.username?.trim() || m.email;
}

const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 31_536_000],
  ["month", 2_592_000],
  ["week", 604_800],
  ["day", 86_400],
  ["hour", 3_600],
  ["minute", 60],
];

/** Locale-aware "3 days ago" / "in 2 days". */
export function relativeTime(iso: string, now = Date.now(), locale?: string): string {
  const diff = Math.round((new Date(iso).getTime() - now) / 1000);
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  for (const [unit, secs] of UNITS) {
    if (Math.abs(diff) >= secs) return rtf.format(Math.round(diff / secs), unit);
  }
  return rtf.format(0, "second");
}

/** True once the invitation window has closed or closes within a day. */
export function expiresSoon(iso: string | null | undefined, now = Date.now()): boolean {
  return !!iso && new Date(iso).getTime() - now < 86_400_000;
}
