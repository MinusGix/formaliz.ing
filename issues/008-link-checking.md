# 008: Build-time link checking

Status: open

The build should fail, or warn loudly while serving, when an entry links to something that doesn't exist.

## What to check

- Internal links (`/notes/foo/`, relative links, `#fragment`s) resolve to a page the build wrote, or to a heading or footnote id on it.
- Images and other `src` references resolve to a file in the output.
- Footnote references have a definition, and definitions are referenced. pulldown-cmark silently drops a reference with no definition, leaving literal `[^x]` text.
- Once 003 lands, `[[wikilinks]]` resolve.
- External links are out of scope for the normal build. An opt-in `press check --external` could HEAD them, rate-limited.

## Notes

- Easiest done after everything is written: walk the output directory, collect every `href`/`src`/`id` per page (a light HTML scan is enough, since the Press produced the markup), then check.
- Links are root-relative (`/theme/...`), so resolve them against the site root.
- Drafts: a published page linking to a draft counts as broken in `build` but not in `serve`.
