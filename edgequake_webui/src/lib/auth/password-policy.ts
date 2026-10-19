/**
 * Password policy — the UI mirror of `PasswordService::validate_password_strength`
 * (edgequake-auth). The server stays authoritative; this only decides what to *tell* the
 * person while they type, so a rejection is never a surprise.
 */

export const MIN_LENGTH = 8;
export const MAX_LENGTH = 128;
const COMMON_PATTERNS = ["password", "12345678", "qwerty123", "admin123", "letmein", "welcome1"];

export type RuleId = "length" | "variety" | "common";
export type Strength = 0 | 1 | 2 | 3 | 4;

export interface PasswordRule {
  id: RuleId;
  ok: boolean;
}

export interface PasswordAssessment {
  rules: PasswordRule[];
  /** Server would accept it. */
  valid: boolean;
  /** 0 = empty … 4 = excellent; for the meter only. */
  strength: Strength;
  classes: number;
}

const has = (re: RegExp, s: string) => re.test(s);

/** Count of {upper, lower, digit, special} classes present. */
export function characterClasses(pw: string): number {
  return [has(/\p{Lu}/u, pw), has(/\p{Ll}/u, pw), has(/[0-9]/, pw), has(/[^\p{L}\p{N}]/u, pw)].filter(Boolean).length;
}

export function assessPassword(pw: string): PasswordAssessment {
  const classes = characterClasses(pw);
  // The server counts bytes (Rust `str::len`), not characters.
  const length = new TextEncoder().encode(pw).length;
  const lengthOk = length >= MIN_LENGTH && length <= MAX_LENGTH;
  const lower = pw.toLowerCase();
  const rules: PasswordRule[] = [
    { id: "length", ok: lengthOk },
    { id: "variety", ok: classes >= 3 },
    { id: "common", ok: pw.length > 0 && !COMMON_PATTERNS.some((c) => lower.includes(c)) },
  ];
  const valid = rules.every((r) => r.ok);
  return { rules, valid, classes, strength: pw ? strengthOf(length, classes, valid) : 0 };
}

function strengthOf(length: number, classes: number, valid: boolean): Strength {
  if (!valid) return 1;
  if (length >= 16 && classes === 4) return 4;
  if (length >= 12) return 3;
  return 2;
}
