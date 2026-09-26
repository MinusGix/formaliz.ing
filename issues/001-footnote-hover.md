# 001: Footnote previews on hover

Status: open
Related: 002, 006

Hovering or focusing a footnote marker in the text should show the note's contents in a small popover beside the marker, so you don't have to jump to the foot of the page.

## Notes

- The markup already exists: references are `<sup class="footnote-reference"><a href="#N">`, and the definitions are `.footnote-definition#N` inside `section.footnotes`.
- A small script (theme/static) can clone the definition's contents into a positioned popover. It should keep MathJax output, either by cloning the already-typeset DOM or by calling `MathJax.typesetPromise` on the clone.
- Tapping on touch devices should open and close the popover, not jump to the note. The plain link must still work without JS.
- Style it as a small printed slip: paper background, a thin rule border, and the note's number in the accent colour.
- When sidenotes (002) are showing, hover previews should turn off or just highlight the sidenote.
