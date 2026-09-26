use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const SUMMARY_WORDS: usize = 50;

#[derive(Debug, Clone, Serialize)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    /// Rendered HTML of the heading's contents (may contain math).
    pub title: String,
}

#[derive(Debug, Clone, Default)]
pub struct Rendered {
    pub html: String,
    pub toc: Vec<TocEntry>,
    pub has_math: bool,
    /// Plain-text opening of the first paragraph.
    pub summary: String,
    pub word_count: usize,
}

pub struct Markdown {
    syntaxes: SyntaxSet,
}

impl Markdown {
    /// `extra_syntaxes` may hold additional `.sublime-syntax` files (e.g. for Heather).
    pub fn new(extra_syntaxes: &Path) -> Result<Self> {
        let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
        if extra_syntaxes.is_dir() {
            builder.add_from_folder(extra_syntaxes, true)?;
        }
        Ok(Self {
            syntaxes: builder.build(),
        })
    }

    pub fn render(&self, src: &str) -> Rendered {
        let opts = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_MATH
            | Options::ENABLE_DEFINITION_LIST;
        let events: Vec<Event> = Parser::new_ext(src, opts).collect();
        let (word_count, mut has_math, summary) = stats(&events);

        let mut out: Vec<Event> = Vec::with_capacity(events.len());
        let mut footnotes: Vec<Event> = Vec::new();
        let mut in_footnote = false;
        let mut toc = Vec::new();
        let mut ids = HashSet::new();

        let mut i = 0;
        while i < events.len() {
            let emitted = match &events[i] {
                Event::Start(Tag::Heading {
                    level, id, classes, ..
                }) => {
                    let end = find_end(&events, i, |e| matches!(e, Event::End(TagEnd::Heading(_))));
                    let inner = &events[i + 1..end];
                    let id = match id {
                        Some(id) => id.to_string(),
                        None => unique_id(&mut ids, &slugify(&plain_text(inner))),
                    };
                    ids.insert(id.clone());
                    let mut inner_html = String::new();
                    html::push_html(&mut inner_html, inner.iter().cloned().map(inline));
                    let level = *level as u8;
                    let class_attr = if classes.is_empty() {
                        String::new()
                    } else {
                        let joined: Vec<&str> = classes.iter().map(|c| c.as_ref()).collect();
                        format!(r#" class="{}""#, escape(&joined.join(" ")))
                    };
                    if !in_footnote && (2..=4).contains(&level) {
                        toc.push(TocEntry {
                            level,
                            id: id.clone(),
                            title: inner_html.clone(),
                        });
                    }
                    i = end;
                    Event::Html(
                        format!(
                            r##"<h{level} id="{id}"{class_attr}><a class="anchor" href="#{id}" aria-hidden="true">§</a>{inner_html}</h{level}>"##,
                            id = escape(&id)
                        )
                        .into(),
                    )
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let end =
                        find_end(&events, i, |e| matches!(e, Event::End(TagEnd::CodeBlock)));
                    let code: String = events[i + 1..end]
                        .iter()
                        .filter_map(|e| match e {
                            Event::Text(t) => Some(t.as_ref()),
                            _ => None,
                        })
                        .collect();
                    let lang = match kind {
                        CodeBlockKind::Fenced(info) => {
                            info.split_whitespace().next().unwrap_or("").to_string()
                        }
                        CodeBlockKind::Indented => String::new(),
                    };
                    i = end;
                    if lang == "math" {
                        has_math = true;
                        Event::Html(
                            format!(
                                "<div class=\"math display tex2jax_process\">\\[{}\\]</div>\n",
                                escape(&code)
                            )
                            .into(),
                        )
                    } else {
                        Event::Html(self.highlight(&code, &lang).into())
                    }
                }
                // With newlines-as-breaks, a `<br>` beside display math adds a blank line.
                Event::SoftBreak
                    if matches!(events.get(i + 1), Some(Event::DisplayMath(_)))
                        || (i > 0 && matches!(events[i - 1], Event::DisplayMath(_))) =>
                {
                    Event::SoftBreak
                }
                Event::Start(Tag::FootnoteDefinition(_)) => {
                    in_footnote = true;
                    footnotes.push(events[i].clone());
                    i += 1;
                    continue;
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    in_footnote = false;
                    footnotes.push(events[i].clone());
                    i += 1;
                    continue;
                }
                other => inline(other.clone()),
            };
            if in_footnote {
                footnotes.push(emitted);
            } else {
                out.push(emitted);
            }
            i += 1;
        }

        // Gather footnote definitions at the foot of the entry, wherever they were written.
        if !footnotes.is_empty() {
            out.push(Event::Html(
                "<section class=\"footnotes\" role=\"doc-endnotes\">\n".into(),
            ));
            out.extend(footnotes);
            out.push(Event::Html("</section>\n".into()));
        }

        let mut html_out = String::new();
        html::push_html(&mut html_out, out.into_iter());
        Rendered {
            html: html_out,
            toc,
            has_math,
            summary,
            word_count,
        }
    }

    fn highlight(&self, code: &str, lang: &str) -> String {
        let lang_attr = if lang.is_empty() {
            String::new()
        } else {
            format!(r#" data-lang="{}""#, escape(lang))
        };
        let syntax = if lang.is_empty() {
            None
        } else {
            self.syntaxes.find_syntax_by_token(lang)
        };
        let body = syntax
            .and_then(|syntax| {
                let mut generator = ClassedHTMLGenerator::new_with_class_style(
                    syntax,
                    &self.syntaxes,
                    ClassStyle::SpacedPrefixed { prefix: "hl-" },
                );
                for line in LinesWithEndings::from(code) {
                    generator
                        .parse_html_for_line_which_includes_newline(line)
                        .ok()?;
                }
                Some(generator.finalize())
            })
            .unwrap_or_else(|| escape(code));
        format!("<pre class=\"code\"{lang_attr}><code>{body}</code></pre>\n")
    }
}

/// Per-event rewrites that apply everywhere, including inside headings.
fn inline(event: Event) -> Event {
    match event {
        // Newlines in the source are newlines on the page.
        Event::SoftBreak => Event::HardBreak,
        Event::InlineMath(m) => Event::InlineHtml(
            format!(
                r#"<span class="math inline tex2jax_process">\({}\)</span>"#,
                escape(&m)
            )
            .into(),
        ),
        Event::DisplayMath(m) => Event::InlineHtml(
            format!(
                r#"<span class="math display tex2jax_process">\[{}\]</span>"#,
                escape(&m)
            )
            .into(),
        ),
        e => e,
    }
}

fn find_end(events: &[Event], start: usize, is_end: impl Fn(&Event) -> bool) -> usize {
    (start + 1..events.len())
        .find(|&j| is_end(&events[j]))
        .unwrap_or(events.len() - 1)
}

fn plain_text(events: &[Event]) -> String {
    let mut s = String::new();
    for e in events {
        match e {
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) => s.push_str(t),
            Event::SoftBreak | Event::HardBreak => s.push(' '),
            _ => {}
        }
    }
    s
}

/// Word count, whether any math appears, and the first paragraph as a plain-text summary.
fn stats(events: &[Event]) -> (usize, bool, String) {
    let mut words = 0;
    let mut math = false;
    let mut in_code = false;
    let mut in_footnote = false;
    let mut summary = String::new();
    // 0: before the first paragraph, 1: inside it, 2: done.
    let mut state = 0u8;
    for e in events {
        match e {
            Event::Start(Tag::CodeBlock(_)) => in_code = true,
            Event::End(TagEnd::CodeBlock) => in_code = false,
            Event::Start(Tag::FootnoteDefinition(_)) => in_footnote = true,
            Event::End(TagEnd::FootnoteDefinition) => in_footnote = false,
            Event::Start(Tag::Paragraph) if state == 0 && !in_footnote => state = 1,
            Event::End(TagEnd::Paragraph) if state == 1 => state = 2,
            Event::Text(t) => {
                if !in_code {
                    words += t.split_whitespace().count();
                }
                if state == 1 {
                    summary.push_str(t);
                }
            }
            Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                words += 1;
                if !matches!(e, Event::Code(_)) {
                    math = true;
                }
                if state == 1 {
                    summary.push_str(t);
                }
            }
            Event::SoftBreak | Event::HardBreak if state == 1 => summary.push(' '),
            _ => {}
        }
    }
    let mut parts: Vec<&str> = summary.split_whitespace().collect();
    let truncated = parts.len() > SUMMARY_WORDS;
    parts.truncate(SUMMARY_WORDS);
    let mut summary = parts.join(" ");
    if truncated {
        summary.push('…');
    }
    (words, math, summary)
}

fn unique_id(ids: &mut HashSet<String>, base: &str) -> String {
    if !ids.contains(base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !ids.contains(candidate))
        .unwrap()
}

pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for c in s.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(c);
        } else {
            pending_dash = true;
        }
    }
    if out.is_empty() { "section".into() } else { out }
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md() -> Markdown {
        Markdown::new(Path::new("/nonexistent")).unwrap()
    }

    #[test]
    fn newlines_are_breaks() {
        let r = md().render("one\ntwo");
        assert!(r.html.contains("one<br />\ntwo"), "{}", r.html);
    }

    #[test]
    fn math_is_protected() {
        let r = md().render("Let $a < b_1 * c_2$ hold.\n\n$$\\sum_{i} x_i$$");
        assert!(r.has_math);
        assert!(r.html.contains(r"\(a &lt; b_1 * c_2\)"), "{}", r.html);
        assert!(r.html.contains(r"\[\sum_{i} x_i\]"), "{}", r.html);
    }

    #[test]
    fn no_break_around_display_math() {
        let r = md().render("before\n$$x$$\nafter");
        assert!(!r.html.contains("<br />"), "{}", r.html);
    }

    #[test]
    fn headings_get_ids_and_toc() {
        let r = md().render("## Hello, World\n\n## Hello, World\n");
        assert_eq!(r.toc.len(), 2);
        assert_eq!(r.toc[0].id, "hello-world");
        assert_eq!(r.toc[1].id, "hello-world-2");
    }

    #[test]
    fn footnotes_move_to_end() {
        let r = md().render("A[^1].\n\n[^1]: Note.\n\nB.");
        let b = r.html.find("<p>B.</p>").unwrap();
        let notes = r.html.find("class=\"footnotes\"").unwrap();
        assert!(notes > b);
    }
}
