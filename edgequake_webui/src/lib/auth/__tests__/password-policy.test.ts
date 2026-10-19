import { describe, expect, it } from "vitest";
import { assessPassword, characterClasses } from "../password-policy";

describe("assessPassword (mirrors edgequake-auth validate_password_strength)", () => {
  it("accepts a password with 3 classes, enough length and no common pattern", () => {
    const a = assessPassword("Str0ng!Passw0rd#2026");
    expect(a.valid).toBe(true);
    expect(a.strength).toBe(4);
  });

  it("rejects short, single-class and common-pattern passwords", () => {
    expect(assessPassword("Ab1!").rules.find((r) => r.id === "length")?.ok).toBe(false);
    expect(assessPassword("alllowercaseonly").rules.find((r) => r.id === "variety")?.ok).toBe(false);
    expect(assessPassword("Welcome1!x").rules.find((r) => r.id === "common")?.ok).toBe(false);
    expect(assessPassword("MyPASSWORD99!").valid).toBe(false);
  });

  it("requires 3 of 4 character classes", () => {
    expect(characterClasses("abcDEF12")).toBe(3);
    expect(assessPassword("abcDEF12").valid).toBe(true);
    expect(assessPassword("abcdefgh12").valid).toBe(false);
  });

  it("measures length in bytes like the server and caps at 128", () => {
    expect(assessPassword("Aa1!" + "é".repeat(2)).rules[0].ok).toBe(true); // 4 + 4 bytes
    expect(assessPassword("Aa1!" + "x".repeat(125)).rules[0].ok).toBe(false);
  });

  it("scores empty as 0 and weak as 1", () => {
    expect(assessPassword("").strength).toBe(0);
    expect(assessPassword("abc").strength).toBe(1);
  });
});
