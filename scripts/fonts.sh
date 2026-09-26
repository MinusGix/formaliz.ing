#!/usr/bin/env bash
# Fetches the theme's fonts from upstream and subsets them into theme/static/fonts/.
# The results are committed; rerun only to change fonts or coverage.
# Needs fonttools with brotli, e.g. on NixOS:
#   nix shell --impure --expr 'with import <nixpkgs> {}; python3.withPackages (p: [ p.fonttools p.brotli ])' --command scripts/fonts.sh
set -euo pipefail
cd "$(dirname "$0")/.."
dest=theme/static/fonts
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$dest"

raw=https://raw.githubusercontent.com
fetch() { curl -sSfL "$1" -o "$tmp/$2"; }

# Upstream EB Garamond (Octavio Pardo) keeps the OpenType features Google Fonts' copy lacks:
# true small caps (smcp, c2sc), old-style figures (onum), historical forms, swashes.
fetch "$raw/octaviopardo/EBGaramond12/master/fonts/variable/EBGaramond%5Bwght%5D.ttf" EBGaramond.ttf
fetch "$raw/octaviopardo/EBGaramond12/master/fonts/variable/EBGaramond-Italic%5Bwght%5D.ttf" EBGaramond-Italic.ttf
fetch "$raw/octaviopardo/EBGaramond12/master/OFL.txt" OFL-EBGaramond.txt
fetch "$raw/google/fonts/main/ofl/imfellenglish/IMFeENrm28P.ttf" IMFellEnglish.ttf
fetch "$raw/google/fonts/main/ofl/imfellenglish/IMFeENit28P.ttf" IMFellEnglish-Italic.ttf
fetch "$raw/google/fonts/main/ofl/imfellenglishsc/IMFeENsc28P.ttf" IMFellEnglishSC.ttf
fetch "$raw/google/fonts/main/ofl/imfellenglish/OFL.txt" OFL-IMFell.txt
fetch "$raw/JetBrains/JetBrainsMono/master/fonts/variable/JetBrainsMono%5Bwght%5D.ttf" JetBrainsMono.ttf
fetch "$raw/JetBrains/JetBrainsMono/master/fonts/variable/JetBrainsMono-Italic%5Bwght%5D.ttf" JetBrainsMono-Italic.ttf
fetch "$raw/JetBrains/JetBrainsMono/master/OFL.txt" OFL-JetBrainsMono.txt

# Latin, Greek (with polytonic), punctuation, arrows, maths, the ❦ ❧ fleurons, f-ligatures.
text_ranges="U+0000-024F,U+0300-036F,U+0370-03FF,U+1E00-1EFF,U+1F00-1FFF,U+2000-218F,U+2190-22FF,U+2766-2767,U+FB00-FB06"
# Code also wants box drawing, technical symbols, brackets, and maths alphanumerics.
mono_ranges="U+0000-024F,U+0370-03FF,U+1E00-1EFF,U+2000-23FF,U+2500-25FF,U+27E6-27EF,U+2A00-2AFF,U+1D400-1D7FF"

subset() { # font ranges out
  pyftsubset "$tmp/$1" --unicodes="$2" --layout-features='*' --flavor=woff2 \
    --output-file="$dest/$3" --no-hinting --desubroutinize
}
subset EBGaramond.ttf "$text_ranges" EBGaramond.woff2
subset EBGaramond-Italic.ttf "$text_ranges" EBGaramond-Italic.woff2
subset IMFellEnglish.ttf '*' IMFellEnglish.woff2
subset IMFellEnglish-Italic.ttf '*' IMFellEnglish-Italic.woff2
subset IMFellEnglishSC.ttf '*' IMFellEnglishSC.woff2
subset JetBrainsMono.ttf "$mono_ranges" JetBrainsMono.woff2
subset JetBrainsMono-Italic.ttf "$mono_ranges" JetBrainsMono-Italic.woff2
cp "$tmp"/OFL-*.txt "$dest/"
ls -l "$dest"
