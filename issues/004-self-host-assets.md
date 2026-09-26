# 004: Self-host fonts (and MathJax)

Status: open

Stop loading EB Garamond, IM Fell English (+SC), and JetBrains Mono from Google Fonts, and serve them from `/theme/fonts/`.

## Notes

- Why: privacy (no requests to Google), stability, and access to the full OpenType features. Google's subsets may strip `smcp` (true small caps) and `onum`, and the theme relies on both. If the self-hosted EB Garamond has real small caps, `font-variant: small-caps` stops being synthesized.
- Source the fonts from upstream (the EB Garamond repo by Octavio Pardo, Igino Marini's IM Fell fonts, JetBrains Mono releases). All are OFL, so include the licence files.
- Subset to Latin, Latin Extended, Greek, and math-adjacent symbols with `pyftsubset`, keeping the `smcp,c2sc,onum,lnum,tnum,liga,dlig` features, and ship woff2.
- Add `<link rel="preload">` for the body face.
- MathJax is also loaded from jsdelivr (v3). Consider vendoring it into `theme/static/` and moving to MathJax 4, which has better fonts and line breaking. Alternatively, render math at build time; that would remove the client-side JS entirely, but needs a JS runtime or a Rust TeX-to-MathML path.
