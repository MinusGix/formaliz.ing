# 006: Footnote back-references

Status: open
Related: 001, 002

Each footnote at the foot of an entry should link back to where it was referenced (↩), and the reference markers need ids to link to.

## Notes

- pulldown-cmark emits `<sup class="footnote-reference"><a href="#N">` with no id, and definitions get no back-link.
- In `src/markdown.rs`, replace `Event::FootnoteReference` with our own inline HTML carrying `id="fnref-N"` (and `-2`, `-3` when a note is referenced more than once). Append `<a class="footnote-backref" href="#fnref-N">↩</a>` to each definition.
- Also consider prefixing ids (`fn-N`) so they cannot collide with heading ids like `#1`.
