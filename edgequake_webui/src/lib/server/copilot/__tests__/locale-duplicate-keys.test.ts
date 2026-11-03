import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

// JSON.parse keeps the LAST duplicate key silently, so a duplicate overwrites real strings
// (members.empty did: a card's string hid the members page's {title, body}).
function duplicateKeys(text: string): string[] {
  const dups: string[] = [];
  // Scan object by object: track keys per nesting level.
  const stack: Set<string>[] = [];
  const path: string[] = [];
  const re = /"((?:[^"\\]|\\.)*)"\s*:|[{}]|"(?:[^"\\]|\\.)*"/g;
  let m: RegExpExecArray | null;
  let pendingKey = "";
  while ((m = re.exec(text))) {
    if (m[0] === "{") {
      stack.push(new Set());
      path.push(pendingKey);
    } else if (m[0] === "}") {
      stack.pop();
      path.pop();
    } else if (m[1] !== undefined) {
      const keys = stack[stack.length - 1];
      if (keys.has(m[1])) dups.push([...path.slice(1), m[1]].join("."));
      keys.add(m[1]);
      pendingKey = m[1];
    }
  }
  return dups;
}

describe("locale files", () => {
  it("detects a nested duplicate (and not equal keys in sibling objects)", () => {
    expect(duplicateKeys('{"a":{"b":"x","b":{"c":1}},"d":{"b":2}}')).toEqual(["a.b"]);
  });
  for (const lang of ["en", "fr", "zh"]) {
    it(`${lang}.json has no duplicate keys`, () => {
      const text = readFileSync(join(__dirname, "../../../../locales", `${lang}.json`), "utf8");
      expect(duplicateKeys(text)).toEqual([]);
    });
  }
});
