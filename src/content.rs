use crate::markdown::{Markdown, Rendered};
use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// TOML front matter, fenced by `+++` lines at the top of a file.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FrontMatter {
    pub title: String,
    pub date: Option<String>,
    pub updated: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub draft: bool,
    /// LessWrong-style epistemic status, shown beneath the title.
    pub epistemic: Option<String>,
    /// Force the table of contents on or off (default: on with 3+ headings).
    pub toc: Option<bool>,
    /// Force the drop cap on or off (default: on for section entries).
    pub dropcap: Option<bool>,
    pub slug: Option<String>,
    /// Section `_index.md` only: what one entry is called ("Note", "Thing").
    pub singular: Option<String>,
}

pub struct Page {
    pub fm: FrontMatter,
    pub title: String,
    pub url: String,
    pub date: Option<NaiveDate>,
    pub updated: Option<NaiveDate>,
    pub rendered: Rendered,
    /// Files beside a bundle's `index.md`: (source, destination relative to the page's dir).
    pub assets: Vec<(PathBuf, PathBuf)>,
    /// 1-based chronological position within its section.
    pub number: usize,
}

pub struct Section {
    pub name: String,
    pub title: String,
    pub singular: String,
    pub description: Option<String>,
    pub intro: Rendered,
    pub pages: Vec<Page>,
}

#[derive(Default)]
pub struct Content {
    pub home: Option<Page>,
    pub pages: Vec<Page>,
    pub sections: Vec<Section>,
    /// Non-markdown files and plain directories, copied as-is: (source, destination relative to site root).
    pub passthrough: Vec<(PathBuf, PathBuf)>,
}

/// Layout of a content directory:
/// - `index.md` is the home page's introduction; other top-level `*.md` are standalone pages.
/// - A directory with `_index.md` is a section; its `*.md` files (or `<slug>/index.md` bundles) are entries.
/// - Anything else is copied through untouched.
pub fn load(dir: &Path, md: &Markdown, drafts: bool) -> Result<Content> {
    let mut content = Content::default();
    for path in sorted_entries(dir)? {
        let name = file_name(&path);
        if path.is_dir() {
            if path.join("_index.md").exists() {
                let section = load_section(&path, &name, md, drafts, &mut content.passthrough)?;
                content.sections.push(section);
            } else {
                content.passthrough.push((path.clone(), PathBuf::from(&name)));
            }
        } else if let Some(stem) = name.strip_suffix(".md") {
            let mut page = load_page(&path, stem, "", md)?;
            if page.fm.draft && !drafts {
                continue;
            }
            if stem == "index" {
                page.url = "/".into();
                content.home = Some(page);
            } else {
                content.pages.push(page);
            }
        } else {
            content.passthrough.push((path.clone(), PathBuf::from(&name)));
        }
    }
    Ok(content)
}

fn load_section(
    dir: &Path,
    name: &str,
    md: &Markdown,
    drafts: bool,
    passthrough: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<Section> {
    let index = dir.join("_index.md");
    let src = fs::read_to_string(&index)?;
    let (fm, body) = split_front_matter(&src).with_context(|| index.display().to_string())?;
    let title = if fm.title.is_empty() {
        capitalize(name)
    } else {
        fm.title.clone()
    };
    let singular = fm
        .singular
        .clone()
        .unwrap_or_else(|| title.strip_suffix('s').unwrap_or(&title).to_string());

    let prefix = format!("/{name}");
    let mut pages = Vec::new();
    for path in sorted_entries(dir)? {
        let entry = file_name(&path);
        if entry == "_index.md" {
            continue;
        }
        let page = if path.is_dir() && path.join("index.md").exists() {
            let mut page = load_page(&path.join("index.md"), &entry, &prefix, md)?;
            for asset in walkdir::WalkDir::new(&path).min_depth(1) {
                let asset = asset?;
                let rel = asset.path().strip_prefix(&path)?.to_path_buf();
                if asset.file_type().is_file() && rel != Path::new("index.md") {
                    page.assets.push((asset.path().to_path_buf(), rel));
                }
            }
            page
        } else if let Some(stem) = entry.strip_suffix(".md") {
            load_page(&path, stem, &prefix, md)?
        } else {
            passthrough.push((path.clone(), Path::new(name).join(&entry)));
            continue;
        };
        if !page.fm.draft || drafts {
            pages.push(page);
        }
    }

    // Chronological, undated entries last.
    pages.sort_by(|a, b| {
        (a.date.is_none(), a.date, &a.title).cmp(&(b.date.is_none(), b.date, &b.title))
    });
    for (i, page) in pages.iter_mut().enumerate() {
        page.number = i + 1;
    }

    Ok(Section {
        name: name.to_string(),
        title,
        singular,
        description: fm.description.clone(),
        intro: md.render(body),
        pages,
    })
}

fn load_page(path: &Path, stem: &str, url_prefix: &str, md: &Markdown) -> Result<Page> {
    let src = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let (fm, body) = split_front_matter(&src).with_context(|| path.display().to_string())?;
    let (file_date, stem) = split_date_prefix(stem);
    let slug = fm.slug.clone().unwrap_or_else(|| stem.to_string());
    let date = match &fm.date {
        Some(d) => Some(parse_date(d).with_context(|| path.display().to_string())?),
        None => file_date,
    };
    let updated = fm
        .updated
        .as_deref()
        .map(parse_date)
        .transpose()
        .with_context(|| path.display().to_string())?;
    let title = if fm.title.is_empty() {
        capitalize(&slug.replace('-', " "))
    } else {
        fm.title.clone()
    };
    Ok(Page {
        title,
        url: format!("{url_prefix}/{slug}/"),
        date,
        updated,
        rendered: md.render(body),
        assets: Vec::new(),
        number: 0,
        fm,
    })
}

pub fn split_front_matter(src: &str) -> Result<(FrontMatter, &str)> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let Some(rest) = src.strip_prefix("+++") else {
        return Ok((FrontMatter::default(), src));
    };
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .context("the opening +++ must be on its own line")?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "+++" {
            let fm = parse_front_matter(&rest[..offset])?;
            return Ok((fm, &rest[offset + line.len()..]));
        }
        offset += line.len();
    }
    bail!("front matter has no closing +++")
}

fn parse_front_matter(src: &str) -> Result<FrontMatter> {
    let mut table: toml::Table = toml::from_str(src).context("parsing front matter")?;
    // Accept bare TOML dates (`date = 2026-09-26`) as well as strings.
    for key in ["date", "updated"] {
        let as_string = match table.get(key) {
            Some(toml::Value::Datetime(dt)) => Some(dt.to_string()),
            _ => None,
        };
        if let Some(s) = as_string {
            table.insert(key.into(), toml::Value::String(s));
        }
    }
    Ok(toml::Value::Table(table)
        .try_into()
        .context("reading front matter")?)
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    let day = s.get(..10).unwrap_or(s);
    NaiveDate::parse_from_str(day, "%Y-%m-%d").with_context(|| format!("bad date {s:?}"))
}

/// `2026-09-26-some-title` → (Some(2026-09-26), "some-title").
fn split_date_prefix(stem: &str) -> (Option<NaiveDate>, &str) {
    if let (Some(day), Some(rest)) = (stem.get(..10), stem.get(10..))
        && let Some(rest) = rest.strip_prefix('-')
        && let Ok(date) = NaiveDate::parse_from_str(day, "%Y-%m-%d")
        && !rest.is_empty()
    {
        return (Some(date), rest);
    }
    (None, stem)
}

fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    entries.retain(|p| !file_name(p).starts_with('.'));
    entries.sort();
    Ok(entries)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
