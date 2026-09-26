# 005: Heather syntax highlighting via tree-sitter

Status: open

Fenced code marked `heather` should be highlighted. Heather already has a tree-sitter grammar (a simple one may need to be written for highlighting), so use that rather than writing a separate `.sublime-syntax` for syntect.

## Notes

- Currently `src/markdown.rs` highlights with syntect and emits classes prefixed `hl-` (`hl-keyword`, `hl-comment`, …), which `style.css` colours. `theme/syntaxes/` can hold extra `.sublime-syntax` files as a stopgap.
- Plan: add the `tree-sitter` and `tree-sitter-highlight` crates, plus the Heather grammar as a dependency (path or git) exposing its `LANGUAGE` and a `highlights.scm` query.
- In `Markdown::highlight`, dispatch by language: `heather` (and aliases such as `hth`?) goes through tree-sitter; everything else keeps using syntect.
- Map tree-sitter capture names to the same `hl-` classes so one stylesheet covers both: `@keyword` → `hl-keyword`, `@type` → `hl-storage hl-type` or a new `hl-type`, `@function` → `hl-entity hl-name`, `@comment` → `hl-comment`, `@string` → `hl-string`, `@number`/`@constant` → `hl-constant`, `@operator` → `hl-keyword hl-operator`, and so on. Add classes for type-theory-specific captures (universes, holes, binders) if the grammar distinguishes them.
- Worth doing at the same time: tree-sitter grammars exist for other languages we'll quote (Lean, Agda, Haskell), which could replace syntect for those too.
