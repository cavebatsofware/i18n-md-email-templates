/*  This file is part of i18n-md-email-templates
 *  Copyright (C) 2026  Grant DeFayette
 *
 *  i18n-md-email-templates is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Lesser General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  i18n-md-email-templates is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public License
 *  along with i18n-md-email-templates.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::BTreeMap;

use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};

/// Runtime values interpolated into templates, keyed by `{{token}}` name.
pub type Vars<'a> = BTreeMap<&'a str, String>;

/// A fully rendered email: a subject line, a styled HTML body, and a plaintext body.
pub struct RenderedEmail {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

/// An optional call-to-action button rendered below the body.
pub struct Cta {
    pub label: String,
    pub url: String,
}

/// The trusted template pieces a caller assembles for one email.
///
/// `layout` is an HTML shell containing a `{{content}}` slot (where the rendered body and
/// CTA are inserted) and an optional `{{footer}}` slot, plus any `{{token}}` placeholders
/// drawn from [`Vars`]. `subject`, `body_md`, and `footer_md` are markdown/plain copy that
/// may also contain `{{token}}` placeholders.
pub struct EmailTemplate<'a> {
    pub layout: &'a str,
    pub subject: &'a str,
    pub body_md: &'a str,
    pub cta: Option<Cta>,
    pub footer_md: Option<&'a str>,
}

/// Escape the five HTML-significant characters for safe interpolation into markup.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Render trusted markdown (GFM tables, strikethrough, inline HTML passthrough) to HTML.
///
/// This does not sanitize: the input is assumed to be your own template copy, not user
/// input. Never feed untrusted content here.
pub fn markdown_to_html(trusted_md: &str) -> String {
    let parser = Parser::new_ext(trusted_md, gfm_options());
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

/// Render markdown to clean plaintext for the text/plain body: emphasis markers, code
/// fences, inline HTML, and link URLs are dropped (link text is kept); blocks are
/// separated by blank lines. `{{token}}` placeholders pass through untouched.
pub fn markdown_to_text(md: &str) -> String {
    let parser = Parser::new_ext(md, gfm_options());
    let mut out = String::new();
    for event in parser {
        match event {
            Event::Text(t) | Event::Code(t) => out.push_str(&t),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::Start(Tag::Item) => out.push_str("- "),
            Event::End(TagEnd::Item) => out.push('\n'),
            Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::CodeBlock) => out.push_str("\n\n"),
            _ => {}
        }
    }
    // Collapse runs of 3+ newlines to two and trim the ends.
    let mut collapsed = String::with_capacity(out.len());
    let mut newlines = 0u32;
    for ch in out.chars() {
        if ch == '\n' {
            newlines += 1;
            if newlines <= 2 {
                collapsed.push(ch);
            }
        } else {
            newlines = 0;
            collapsed.push(ch);
        }
    }
    collapsed.trim().to_string()
}

/// Inline a document's `<style>` rules into element `style=` attributes and drop the
/// `<style>` tag, so the rendered HTML keeps its styling in email clients that ignore
/// embedded stylesheets. No remote or filesystem stylesheet loading is performed.
pub fn inline_css(html: &str) -> Result<String, css_inline::InlineError> {
    let inliner = css_inline::CSSInliner::options()
        .load_remote_stylesheets(false)
        .keep_link_tags(false)
        .build();
    inliner.inline(html)
}

fn gfm_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_GFM);
    options
}

/// Replace `{{token}}` placeholders in a single left-to-right pass.
///
/// Inserted values are never rescanned, so a value that itself contains `{{...}}` cannot
/// trigger further substitution. Unknown tokens are left in place.
pub fn substitute(template: &str, vars: &Vars) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        if let Some(close_rel) = rest[open + 2..].find("}}") {
            let key = rest[open + 2..open + 2 + close_rel].trim();
            if let Some(val) = vars.get(key) {
                out.push_str(&rest[..open]);
                out.push_str(val);
                rest = &rest[open + 2 + close_rel + 2..];
                continue;
            }
        }
        // No closing `}}` or unknown token: emit through the `{{` and keep scanning.
        out.push_str(&rest[..open + 2]);
        rest = &rest[open + 2..];
    }
    out.push_str(rest);
    out
}

/// Render an email template against runtime values.
///
/// Subject and plaintext body use the raw values; the HTML body escapes them *after*
/// markdown rendering. The CTA (if any) is appended to both bodies, and `{{content}}` /
/// `{{footer}}` are filled into the layout last so values can never collide with layout
/// tokens.
pub fn render(tmpl: &EmailTemplate, vars: &Vars) -> RenderedEmail {
    let escaped: Vars = vars.iter().map(|(k, v)| (*k, html_escape(v))).collect();

    let subject = substitute(tmpl.subject, vars);

    let mut content_html = substitute(&markdown_to_html(tmpl.body_md), &escaped);
    if let Some(cta) = &tmpl.cta {
        content_html.push('\n');
        content_html.push_str(&cta_html(cta));
    }
    let footer_html = tmpl
        .footer_md
        .map(|f| substitute(&markdown_to_html(f), &escaped))
        .unwrap_or_default();

    // Fill the layout in one pass: escaped user vars plus the `content`/`footer`
    // slots. `substitute` never rescans inserted text, so a user value that happens
    // to contain `{{content}}`/`{{footer}}` cannot collide with the layout slots.
    let mut layout_vars = escaped;
    layout_vars.insert("content", content_html);
    layout_vars.insert("footer", footer_html);
    let html_body = substitute(tmpl.layout, &layout_vars);

    let mut text_body = substitute(&markdown_to_text(tmpl.body_md), vars);
    if let Some(cta) = &tmpl.cta {
        text_body.push_str("\n\n");
        text_body.push_str(&cta_text(cta));
    }
    if let Some(footer) = tmpl.footer_md {
        text_body.push_str("\n\n");
        text_body.push_str(&substitute(&markdown_to_text(footer), vars));
    }

    RenderedEmail {
        subject,
        html_body,
        text_body,
    }
}

fn cta_html(cta: &Cta) -> String {
    let url = html_escape(&cta.url);
    format!(
        "<div class=\"btn-wrap\"><a class=\"btn\" href=\"{url}\">{label}</a></div>\n\
         <p class=\"link-fallback\">{url}</p>",
        url = url,
        label = html_escape(&cta.label),
    )
}

fn cta_text(cta: &Cta) -> String {
    format!("{}: {}", cta.label, cta.url)
}
