use crate::config::SiteConfig;
use crate::content::{self, Page, Section};
use crate::markdown::{Markdown, TocEntry, slugify};
use anyhow::{Context, Result, anyhow};
use chrono::{Datelike, Local, NaiveDate};
use minijinja::{Environment, Value, context};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const WORDS_PER_MINUTE: usize = 230;
const FEED_ENTRIES: usize = 20;

#[derive(Clone)]
pub struct BuildOptions {
    pub drafts: bool,
    /// Replaces the site's `base_url` (the dev server points it at localhost).
    pub base_url: Option<String>,
    pub live_reload: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Link {
    title: String,
    url: String,
}

#[derive(Debug, Clone, Serialize)]
struct PageView {
    title: String,
    url: String,
    section: Option<String>,
    kind: Option<String>,
    number: usize,
    number_padded: String,
    date: Option<String>,
    date_long: Option<String>,
    date_short: Option<String>,
    updated: Option<String>,
    updated_long: Option<String>,
    atom_updated: String,
    description: Option<String>,
    summary: String,
    tags: Vec<Link>,
    epistemic: Option<String>,
    content: String,
    toc: Vec<TocEntry>,
    show_toc: bool,
    has_math: bool,
    reading_minutes: usize,
    draft: bool,
    dropcap: bool,
    prev: Option<Link>,
    next: Option<Link>,
}

#[derive(Serialize)]
struct SectionView {
    name: String,
    title: String,
    description: Option<String>,
    intro: String,
    url: String,
}

#[derive(Serialize)]
struct TagView {
    title: String,
    url: String,
    pages: Vec<PageView>,
}

/// Builds `sites/<site>` into `out`. Returns the number of entries written.
pub fn build(root: &Path, site: &str, out: &Path, opts: &BuildOptions) -> Result<usize> {
    let site_dir = root.join("sites").join(site);
    let theme = root.join("theme");
    let mut config = SiteConfig::load(&site_dir.join("site.toml"))?;
    if let Some(base) = &opts.base_url {
        config.base_url = base.trim_end_matches('/').to_string();
    }
    let md = Markdown::new(&theme.join("syntaxes"))?;
    let content = content::load(&site_dir.join("content"), &md, opts.drafts)?;

    if out.exists() {
        fs::remove_dir_all(out).with_context(|| format!("clearing {}", out.display()))?;
    }
    fs::create_dir_all(out)?;
    copy_tree(&theme.join("static"), &out.join("theme"))?;
    // Third-party assets fetched by scripts/ (MathJax), served beside the theme.
    let vendor = theme.join("vendor");
    if !vendor.join("mathjax").exists() {
        eprintln!("warning: theme/vendor/mathjax is missing; run scripts/mathjax.sh");
    }
    copy_tree(&vendor, &out.join("theme"))?;
    copy_tree(&site_dir.join("static"), out)?;
    for (src, rel) in &content.passthrough {
        copy_tree(src, &out.join(rel))?;
    }

    let today = Local::now().date_naive();
    let entry_count: usize = content.sections.iter().map(|s| s.pages.len()).sum();
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader(theme.join("templates")));
    env.add_filter("roman", roman);
    env.add_global("site", Value::from_serialize(&config));
    env.add_global(
        "volume",
        (today.year() - config.established.unwrap_or(today.year()) + 1).max(1),
    );
    env.add_global("entry_count", entry_count);
    env.add_global("build_date", long_date(today));
    env.add_global("build_id", Local::now().timestamp());
    env.add_global("live_reload", opts.live_reload);

    // Section entries.
    let mut all: Vec<PageView> = Vec::new();
    for section in &content.sections {
        let views: Vec<PageView> = section
            .pages
            .iter()
            .enumerate()
            .map(|(i, page)| {
                let prev = i.checked_sub(1).map(|j| link(&section.pages[j]));
                let next = section.pages.get(i + 1).map(link);
                view(page, Some(section), prev, next, today)
            })
            .collect();
        for (page, v) in section.pages.iter().zip(&views) {
            let html = render(&env, "post.html", context! { page => v, has_math => v.has_math })?;
            write_page(out, &v.url, &html)?;
            copy_assets(out, page)?;
        }
        let section_view = SectionView {
            name: section.name.clone(),
            title: section.title.clone(),
            description: section.description.clone(),
            intro: section.intro.html.clone(),
            url: format!("/{}/", section.name),
        };
        let newest_first: Vec<&PageView> = views.iter().rev().collect();
        let html = render(
            &env,
            "section.html",
            context! { section => section_view, pages => newest_first, has_math => section.intro.has_math },
        )?;
        write_page(out, &format!("/{}/", section.name), &html)?;
        all.extend(views);
    }
    all.sort_by(|a, b| b.date.cmp(&a.date));

    // Standalone pages.
    for page in &content.pages {
        let v = view(page, None, None, None, today);
        let html = render(&env, "page.html", context! { page => v, has_math => v.has_math })?;
        write_page(out, &v.url, &html)?;
    }

    // Home.
    let home = content.home.as_ref().map(|p| view(p, None, None, None, today));
    let recent: Vec<&PageView> = all.iter().take(config.home_recent).collect();
    let home_math = home.as_ref().is_some_and(|h| h.has_math);
    let html = render(
        &env,
        "index.html",
        context! { page => home, recent => recent, has_math => home_math },
    )?;
    write_page(out, "/", &html)?;

    // Tags.
    let mut tags: BTreeMap<String, TagView> = BTreeMap::new();
    for v in &all {
        for tag in &v.tags {
            tags.entry(tag.url.clone())
                .or_insert_with(|| TagView {
                    title: tag.title.clone(),
                    url: tag.url.clone(),
                    pages: Vec::new(),
                })
                .pages
                .push(v.clone());
        }
    }
    for tag in tags.values() {
        let html = render(&env, "tag.html", context! { tag => tag })?;
        write_page(out, &tag.url, &html)?;
    }
    let tag_list: Vec<&TagView> = tags.values().collect();
    let html = render(&env, "tags.html", context! { tags => tag_list })?;
    write_page(out, "/tags/", &html)?;

    // Feed and 404.
    let feed_updated = all
        .iter()
        .map(|v| v.atom_updated.clone())
        .max()
        .unwrap_or_else(|| atom_date(today));
    let feed_pages: Vec<&PageView> = all.iter().filter(|v| !v.draft).take(FEED_ENTRIES).collect();
    let feed = render(
        &env,
        "feed.xml",
        context! { pages => feed_pages, updated => feed_updated },
    )?;
    fs::write(out.join("feed.xml"), feed)?;
    let not_found = render(&env, "404.html", context! {})?;
    fs::write(out.join("404.html"), not_found)?;

    Ok(entry_count)
}

fn view(
    page: &Page,
    section: Option<&Section>,
    prev: Option<Link>,
    next: Option<Link>,
    today: NaiveDate,
) -> PageView {
    let r = &page.rendered;
    PageView {
        title: page.title.clone(),
        url: page.url.clone(),
        section: section.map(|s| s.name.clone()),
        kind: section.map(|s| s.singular.clone()),
        number: page.number,
        number_padded: format!("{:03}", page.number),
        date: page.date.map(|d| d.to_string()),
        date_long: page.date.map(long_date),
        date_short: page.date.map(|d| d.format("%-d %b %Y").to_string()),
        updated: page.updated.map(|d| d.to_string()),
        updated_long: page.updated.map(long_date),
        atom_updated: atom_date(page.updated.or(page.date).unwrap_or(today)),
        description: page.fm.description.clone(),
        summary: r.summary.clone(),
        tags: page
            .fm
            .tags
            .iter()
            .map(|t| Link {
                title: t.clone(),
                url: format!("/tags/{}/", slugify(t)),
            })
            .collect(),
        epistemic: page.fm.epistemic.clone(),
        content: r.html.clone(),
        toc: r.toc.clone(),
        show_toc: page.fm.toc.unwrap_or(r.toc.len() >= 3),
        has_math: r.has_math,
        reading_minutes: r.word_count.div_ceil(WORDS_PER_MINUTE).max(1),
        draft: page.fm.draft,
        dropcap: page.fm.dropcap.unwrap_or(section.is_some()),
        prev,
        next,
    }
}

fn link(page: &Page) -> Link {
    Link {
        title: page.title.clone(),
        url: page.url.clone(),
    }
}

fn render(env: &Environment, template: &str, ctx: Value) -> Result<String> {
    env.get_template(template)
        .and_then(|t| t.render(ctx))
        .map_err(|e| anyhow!("rendering {template}: {e:#}"))
}

fn write_page(out: &Path, url: &str, html: &str) -> Result<()> {
    let dir = out.join(url.trim_matches('/'));
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("index.html"), html)?;
    Ok(())
}

fn copy_assets(out: &Path, page: &Page) -> Result<()> {
    let dir = out.join(page.url.trim_matches('/'));
    for (src, rel) in &page.assets {
        let dest = dir.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, &dest).with_context(|| format!("copying {}", src.display()))?;
    }
    Ok(())
}

/// Copies a file, or a directory's contents, to `dest`. Missing sources are skipped.
fn copy_tree(src: &Path, dest: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    if src.is_file() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dest).with_context(|| format!("copying {}", src.display()))?;
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let target = dest.join(entry.path().strip_prefix(src)?);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            fs::copy(entry.path(), &target)
                .with_context(|| format!("copying {}", entry.path().display()))?;
        }
    }
    Ok(())
}

/// "26th September, 2026"
fn long_date(d: NaiveDate) -> String {
    let day = d.day();
    let suffix = match (day % 10, day % 100) {
        (1, n) if n != 11 => "st",
        (2, n) if n != 12 => "nd",
        (3, n) if n != 13 => "rd",
        _ => "th",
    };
    format!("{day}{suffix} {}", d.format("%B, %Y"))
}

fn atom_date(d: NaiveDate) -> String {
    format!("{d}T00:00:00Z")
}

fn roman(n: u32) -> String {
    if n == 0 {
        return "N".into();
    }
    const NUMERALS: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut n = n;
    let mut out = String::new();
    for (value, numeral) in NUMERALS {
        while n >= value {
            out.push_str(numeral);
            n -= value;
        }
    }
    out
}
