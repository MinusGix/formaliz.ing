+++
title = "A Typesetting Specimen"
date = 2026-09-26
description = "Every feature of the press, set in one place."
tags = ["meta", "type theory"]
draft = true
epistemic = "A test sheet. Entirely confident that it exists; less so that it is beautiful."
+++

This sheet exists to show what the press can set. It is a draft, and so never printed to the public site, but it appears while serving locally.
Note that a single newline in the source is a line break on the page,
exactly as it was written.

A blank line begins a new paragraph. Quotes are "curled" and dashes---like these---are set properly.

## Mathematics

Inline mathematics sits in the line: the identity type $a =_A b$ for $a, b : A$, or the naturals $\N$ via a macro from `site.toml`. Things that would trouble Markdown, like $a_1 * b_2 < c_3$, are left alone.

Display mathematics stands on its own:

$$
\prod_{x : A} \sum_{y : B(x)} P(x, y) \;\to\; \sum_{f : \prod_{x : A} B(x)} \prod_{x : A} P(x, f(x))
$$

A fenced block marked `math` works as well, and supports numbering:

```math
\begin{equation}
  \frac{\Gamma \vdash a : A \qquad \Gamma \vdash b : B(a)}{\Gamma \vdash (a, b) : \textstyle\sum_{x : A} B(x)}
\end{equation}
```

## Code

```rust
/// The press's own escape function.
pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;")
}
```

```haskell
data Nat = Z | S Nat

plus :: Nat -> Nat -> Nat
plus Z     n = n
plus (S m) n = S (plus m n)
```

### A Subsection

Footnotes may be written anywhere[^where], and are gathered at the foot of the entry.[^second] Where the page is wide enough, they are set in the margin beside the line that cites them;[^margin] where it is not, hovering or tapping the numeral shows the note in place. Notes cited close together stack rather than collide.[^stack]

[^margin]: Like this one. A sidenote may run to several paragraphs, and carry mathematics: for $f : A \to B$, the fibre over $b$ is
$$\mathsf{fib}_f(b) \;:\equiv\; \sum_{a : A} f(a) = b.$$

    A second paragraph, indented under the note, belongs to it too.

[^stack]: This note was cited in the same paragraph as the one above, so it is pushed below it.

[^where]: Including right here, mid-document.

> A block quotation, for when someone else said it better.
> It keeps its line breaks, too.

---

| Theory | Univalence | Canonicity |
|--------|:----------:|:----------:|
| MLTT   | no         | yes        |
| HoTT   | axiom      | no         |
| CTT    | theorem    | yes        |

Definition
: A term whose meaning is fixed by fiat.

- [x] Markdown
- [x] MathJax
- [ ] Sidenotes

[^second]: A second note, with math: $\lambda x.\, x$.
