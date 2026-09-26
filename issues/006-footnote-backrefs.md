# 006: Footnote back-references

Status: done
Related: 001, 002

Each footnote at the foot of an entry should link back to where it was referenced (↩), and the reference markers need ids to link to.

## Notes

- pulldown-cmark emits `<sup class="footnote-reference"><a href="#N">` with no id, and definitions get no back-link.
- In `src/markdown.rs`, replace `Event::FootnoteReference` with our own inline HTML carrying `id="fnref-N"` (and `-2`, `-3` when a note is referenced more than once). Append `<a class="footnote-backref" href="#fnref-N">↩</a>` to each definition.
- Also consider prefixing ids (`fn-N`) so they cannot collide with heading ids like `#1`.

## Resolution

`src/markdown.rs` now renders footnotes itself:
- Numbers are assigned in order of first reference, regardless of names or where the definitions sit.
- References are `<sup class="footnote-ref" id="fnref-N">` (`fnref-N-2`, … for repeat citations). Notes are `<li class="footnote" id="fn-N">` in `section.footnotes > ol`.
- Each note ends with a `↩` link back to its first reference, placed on the note's last line when it ends in a paragraph.
