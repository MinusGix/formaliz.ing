# 007: Admonitions

Status: open

Styled call-out blocks for side matter: asides, notes, warnings, digressions. They should look like part of the printed page, not like a docs site: no coloured boxes with icons.

## Syntax

- pulldown-cmark 0.13 parses GitHub's alert syntax behind `Options::ENABLE_GFM`: `> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]` arrive as `Tag::BlockQuote(Some(BlockQuoteKind::…))`. This is cheap to support, and it degrades to a plain blockquote in other renderers.
- That fixed set doesn't fit this journal. We'd want kinds like *aside*, *digression*, *nota bene*, *caveat*, and *aside for the formally inclined*. Options:
  - Keep the `> [!KIND]` syntax but parse it ourselves. When a blockquote's first text is `[!word]` (optionally followed by a title: `> [!aside] On naming`), strip it and emit our own markup. The kinds come from a table, possibly configurable in `site.toml`.
  - Fenced containers, `::: aside` … `:::`. pulldown-cmark doesn't parse these, so they would need pre-processing, which interacts badly with code blocks.
  - The first option is preferred.
- A collapsible variant (`> [!digression]-` → `<details>`) would suit long digressions.

## Styling ideas

- A marginal label in small caps or IM Fell italic ("Nota bene.", "Caveat.", "Digression."), set like a run-in head, as in the epistemic-status line.
- Rules rather than boxes: thin rules above and below, or a double rule at the left in brass, as blockquotes have. Warnings could use the oxblood accent and perhaps a ☞ manicule (check glyph coverage in EB Garamond and add its range to `scripts/fonts.sh` if needed).
- Slightly smaller type and a narrower measure, so they read as subordinate.
- On wide screens, some kinds (asides) could go to the margin, sharing the layout from 002.

## Related: theorem-like environments

Type-theory and maths posts will want Definition, Theorem, Lemma, Proof (∎), and Example. The same `[!kind]` mechanism could produce them with numbering (*Theorem 3.*, *Definition 2 (Univalence).*), styled like old journals: italic run-in heads and small-caps labels. They could become their own issue once the admonition machinery exists.
