#!/usr/bin/env bash
# Fetches MathJax and its New Computer Modern font into theme/vendor/mathjax/ (gitignored).
# The Press copies it to /theme/mathjax/. Run once after cloning; CI runs it before building.
set -euo pipefail
cd "$(dirname "$0")/.."
version=4.1.3
dest=theme/vendor/mathjax
if [[ -f "$dest/.version" && "$(cat "$dest/.version")" == "$version" ]]; then
  exit 0
fi
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
registry=https://registry.npmjs.org
curl -sSfL "$registry/mathjax/-/mathjax-$version.tgz" | tar xz -C "$tmp" --transform 's,^package,mathjax,'
curl -sSfL "$registry/@mathjax/mathjax-newcm-font/-/mathjax-newcm-font-$version.tgz" | tar xz -C "$tmp" --transform 's,^package,font,'

rm -rf "$dest"
mkdir -p "$dest/fonts/mathjax-newcm-font"
cp -r "$tmp"/mathjax/{tex-chtml.js,input,output,ui,a11y,sre,LICENSE} "$dest/"
cp -r "$tmp"/font/{chtml,chtml.js,LICENSE} "$dest/fonts/mathjax-newcm-font/" 2>/dev/null \
  || cp -r "$tmp"/font/{chtml,chtml.js} "$dest/fonts/mathjax-newcm-font/"
echo "$version" > "$dest/.version"
echo "MathJax $version → $dest ($(du -sh "$dest" | cut -f1))"
