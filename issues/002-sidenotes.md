# 002: Sidenotes in the margin

Status: open
Related: 001, 006

When the viewport is wide enough, footnotes should be set in the margin beside the line that references them, in the manner of Tufte, Gwern, and LessWrong. On narrower screens they stay at the foot of the entry.

## Notes

- The measure is 36rem, and `.sheet` gives up to 8rem of margin. Sidenotes probably need the right margin to open to about 14–16rem above a breakpoint (roughly 70rem wide), so the sheet layout has to change as well.
- Layout options:
  - **JS placement**: position each note next to its reference, pushing it down to avoid overlaps. This handles long notes and collisions; Gwern's `sidenotes.js` works this way.
  - **Build-time inlining**: emit each note's contents next to its reference as a floated `<aside>` (pure CSS, Tufte-style), with the collected foot section as the narrow-screen fallback. There is no JS, but long notes can collide.
- The Press already gathers footnote events (`src/markdown.rs`), so inlining at build time is cheap if we go that way.
- A short note might sit in the margin while a long one stays at the foot. An explicit per-note choice could be useful.
