# 002: Sidenotes in the margin

Status: done
Related: 001, 006

When the viewport is wide enough, footnotes should be set in the margin beside the line that references them, in the manner of Tufte, Gwern, and LessWrong. On narrower screens they stay at the foot of the entry.

## Notes

- The measure is 36rem, and `.sheet` gives up to 8rem of margin. Sidenotes probably need the right margin to open to about 14–16rem above a breakpoint (roughly 70rem wide), so the sheet layout has to change as well.
- Layout options:
  - **JS placement**: position each note next to its reference, pushing it down to avoid overlaps. This handles long notes and collisions; Gwern's `sidenotes.js` works this way.
  - **Build-time inlining**: emit each note's contents next to its reference as a floated `<aside>` (pure CSS, Tufte-style), with the collected foot section as the narrow-screen fallback. There is no JS, but long notes can collide.
- The Press already gathers footnote events (`src/markdown.rs`), so inlining at build time is cheap if we go that way.
- A short note might sit in the margin while a long one stays at the foot. An explicit per-note choice could be useful.

## Resolution

Used JS placement, which handles notes of any content: paragraphs, display math, code.
- At widths ≥ 1120px, pages with footnotes get `.sheet.with-margin`: the sheet widens to measure + 2.5rem gutter + 14rem margin column, and the text column moves to the left.
- `footnotes.js` moves (not clones) each note's body into an absolutely positioned `<aside class="sidenote">`, aligned to the top of the line holding its first reference. A note is pushed down past the previous one when they would overlap, and the foot section is hidden meanwhile.
- Layout reruns on article resize (ResizeObserver), font load, MathJax startup, and window load. Crossing the breakpoint moves the bodies back.
- Hovering a reference or its sidenote highlights the other. Clicking a reference flashes its sidenote instead of jumping.
- Without JS, the notes stay at the foot.

Possible later refinements: allow a note to opt out and stay at the foot (e.g. very long notes), and let notes that overflow the article's bottom shift upward.
