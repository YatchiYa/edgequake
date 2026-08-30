/** Shared LaTeX markdown sample for unit/E2E fixture pages. */
export const LATEX_MARKDOWN_SAMPLE = `
# LaTeX rendering proof

Inline dollar: $E = mc^2$

Block dollar:

$$
\\int_0^1 x^2 \\, dx = \\frac{1}{3}
$$

Paren inline: \\( \\alpha + \\beta = \\gamma \\)

Bracket block:

\\[
\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}
\\]

## HTML-in-codespan (PDF extraction artifact)

Reward \`r<sub>i</sub>(W)\` and cost \`c<sub>i</sub>(W)\` with reference \`y<sub>i</sub><sup>*</sup>\`.
`.trim();
