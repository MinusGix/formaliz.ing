# 004: Self-host fonts (and MathJax)

Status: done

Stop loading EB Garamond, IM Fell English (+SC), and JetBrains Mono from Google Fonts, and serve them from `/theme/fonts/`.

## Notes

- Why: privacy (no requests to Google), stability, and access to the full OpenType features. Google's subsets may strip `smcp` (true small caps) and `onum`, and the theme relies on both. If the self-hosted EB Garamond has real small caps, `font-variant: small-caps` stops being synthesized.
- Source the fonts from upstream (the EB Garamond repo by Octavio Pardo, Igino Marini's IM Fell fonts, JetBrains Mono releases). All are OFL, so include the licence files.
- Subset to Latin, Latin Extended, Greek, and math-adjacent symbols with `pyftsubset`, keeping the `smcp,c2sc,onum,lnum,tnum,liga,dlig` features, and ship woff2.
- Add `<link rel="preload">` for the body face.
- MathJax is also loaded from jsdelivr (v3). Consider vendoring it into `theme/static/` and moving to MathJax 4, which has better fonts and line breaking. Alternatively, render math at build time; that would remove the client-side JS entirely, but needs a JS runtime or a Rust TeX-to-MathML path.

## Resolution

- **Fonts:** `scripts/fonts.sh` fetches upstream EB Garamond (Octavio Pardo's variable fonts, which do include `smcp`, `c2sc` and `onum`), IM Fell English/SC, and JetBrains Mono. It subsets them with every layout feature kept and writes woff2 files to `theme/static/fonts/` (committed, with OFL licences). The body face is preloaded.
- EB Garamond lacks ✦ and ⁂, so the nav separator is now a middle dot, and `hr` draws its asterism from the face's own asterisks.
- **MathJax:** upgraded to 4.1.3. `scripts/mathjax.sh` fetches it and the New Computer Modern font from the npm registry into `theme/vendor/mathjax/` (gitignored, about 10 MB). The Press copies that directory to `/theme/mathjax/`, `loader.paths` points MathJax there, and CI runs the script before building.
- Verified that a page with math makes no requests to other origins.
- Not done: rendering math at build time.
