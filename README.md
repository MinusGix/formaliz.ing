# formaliz.ing

Source for [formaliz.ing](https://formaliz.ing) and [heather.formaliz.ing](https://heather.formaliz.ing), and the Press: the small Rust static-site generator that builds them.

## Usage

```sh
cargo run -- serve                    # main site at http://localhost:8000, drafts shown, live reload
cargo run -- serve --site heather --port 8001
cargo run -- new main notes "On Univalence"   # new draft in sites/main/content/notes/
cargo run -- build                    # every site into public/<site>/
```

## Layout

```
sites/<site>/site.toml      title, base_url, nav, MathJax macros, ...
sites/<site>/static/        copied to the site root (CNAME lives here)
sites/<site>/content/
  index.md                  front-page introduction
  about.md                  standalone page → /about/
  notes/_index.md           a directory with _index.md is a section → /notes/
  notes/some-entry.md       → /notes/some-entry/
  notes/2026-09-26-foo.md   date prefix is stripped from the slug and used as the date
  notes/bundle/index.md     → /notes/bundle/, with its sibling files (images, etc.) copied alongside
  anything-else/            copied through untouched
theme/templates/            minijinja templates, shared by all sites
theme/static/               served at /theme/
theme/syntaxes/             extra .sublime-syntax files for code highlighting
src/                        the Press
issues/                     Press feature ideas and bugs (not content)
```

## Writing

Front matter is TOML between `+++` lines:

```toml
+++
title = "On Univalence"
date = 2026-09-26
updated = 2026-10-01        # optional
description = "One line for listings and the feed."
tags = ["type theory", "hott"]
epistemic = "Exploratory; I expect to revise this."
draft = true                # hidden from `build`, shown by `serve`
toc = false                 # default: contents shown with 3+ headings
dropcap = false             # default: on for section entries
+++
```

Markdown is CommonMark with GitHub tables, strikethrough, task lists, footnotes, definition lists, smart punctuation, and `{#id .class}` heading attributes.
**A single newline is a line break.** A blank line starts a new paragraph.

Mathematics is set by MathJax, loaded only on pages that use it: `$inline$`, `$$display$$`, or a ```` ```math ```` block (which also takes `equation`/`align` environments and numbering). Per-site macros live in `site.toml` under `[mathjax_macros]`.

Footnotes may be defined anywhere; they are collected at the foot of the entry.

## Deployment

Pushing to `main` runs `.github/workflows/deploy.yml`, which builds every site and publishes `public/main` to this repository's GitHub Pages.

One-time setup:

1. Repository settings → Pages → Source: **GitHub Actions**. Custom domain: `formaliz.ing`.
2. DNS for `formaliz.ing`: apex `A` records to `185.199.108.153`, `185.199.109.153`, `185.199.110.153`, `185.199.111.153` (and `AAAA` to `2606:50c0:8000::153` … `8003::153`); optionally `www` `CNAME` → `minusgix.github.io`.
3. Verify the domain under GitHub account settings → Pages, then tick "Enforce HTTPS".

### heather.formaliz.ing

GitHub Pages serves one custom domain per repository, so Heather's site is pushed to a second repository:

1. Create an empty repository, e.g. `MinusGix/heather.formaliz.ing`.
2. Generate a key pair (`ssh-keygen -t ed25519 -f heather-deploy -N ""`). Add the public key to that repository as a deploy key with write access, and the private key to this repository as the secret `HEATHER_DEPLOY_KEY`.
3. In this repository, set the Actions variable `HEATHER_REPO` to `MinusGix/heather.formaliz.ing`.
4. After the first deploy, set that repository's Pages source to the `gh-pages` branch, custom domain `heather.formaliz.ing`.
5. DNS: `heather` `CNAME` → `minusgix.github.io`.
