# 009: Print stylesheet

Status: open

A journal styled as typeset paper should print, or save to PDF, like one.

## Notes

- `@media print`: plain white paper; no grain, no dark theme; ink in black or near-black, with the accent kept for footnote numerals.
- Replace the masthead with a small running head. Drop the nav, the colophon's links and the theme toggle.
- Sidenotes: the margin column's absolute layout won't paginate. Print the foot section instead, i.e. show `.footnotes` and hide `.sidenote-column`. Or, for real footnotes per page, investigate `float: footnote` (only in Paged Media engines such as WeasyPrint and Prince, not browsers).
- Avoid page breaks inside display math, code blocks, tables and figures (`break-inside: avoid`), and after headings (`break-after: avoid`).
- Print the URL after external links (`a[href^="http"]::after { content: " (" attr(href) ")" }`), maybe only in the foot section to keep the text clean.
- `@page` margins, and page numbers where supported.
