# 001: Footnote previews on hover

Status: done
Related: 002, 006

Hovering or focusing a footnote marker in the text should show the note's contents in a small popover beside the marker, so you don't have to jump to the foot of the page.

## Notes

- The markup already exists: references are `<sup class="footnote-reference"><a href="#N">`, and the definitions are `.footnote-definition#N` inside `section.footnotes`.
- A small script (theme/static) can clone the definition's contents into a positioned popover. It should keep MathJax output, either by cloning the already-typeset DOM or by calling `MathJax.typesetPromise` on the clone.
- Tapping on touch devices should open and close the popover, not jump to the note. The plain link must still work without JS.
- Style it as a small printed slip: paper background, a thin rule border, and the note's number in the accent colour.
- When sidenotes (002) are showing, hover previews should turn off or just highlight the sidenote.

## Resolution

Done in `theme/static/footnotes.js`, together with 002 and 006. When sidenotes aren't showing:
- Hovering a reference (120 ms delay) or giving it keyboard focus shows a popover containing a clone of the note, with its typeset MathJax.
- On touch screens, a tap toggles the popover instead of jumping to the foot; tapping outside or pressing Esc closes it.
- The popover's numeral links to the note at the foot.
- It sits below the reference, or above it if it doesn't fit below, and is kept within the viewport.
