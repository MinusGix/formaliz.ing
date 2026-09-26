# 003: Wikilinks and backlinks

Status: open

Support `[[target]]` and `[[target|shown text]]` links between entries, and list backlinks ("Referenced by") at the end of each entry.

## Notes

- Resolution: match the target against slugs, and maybe titles, across all sections of the same site. Allow `[[notes/foo]]` when a name is ambiguous. An unresolved link should fail the build, or warn loudly and render as a visibly broken link while serving.
- Cross-site links: `[[heather:log/foo]]` could resolve against the other site's `base_url`, provided the Press loads both sites' content in one pass.
- pulldown-cmark does not parse wikilinks, so either pre-process the source (careful with code spans, code blocks, and math) or handle them in the event stream, where `[[` arrives as `Text` events split around brackets. The event-stream approach is safer.
- Backlinks need two passes: load and index every page, then render. `content::load` already loads everything before rendering, so this fits.
- Possibly later: previews on hover for internal links (Gwern-style), reusing the popover from 001.
