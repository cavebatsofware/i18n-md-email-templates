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

use serde_json::json;

use crate::{html_escape, markdown_to_html, render, substitute, Catalog, Cta, EmailTemplate, Vars};

const LAYOUT: &str =
    "<html><body><h1>{{title}}</h1>{{content}}<footer>{{footer}}</footer></body></html>";

fn vars(pairs: &[(&'static str, &str)]) -> Vars<'static> {
    pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
}

#[test]
fn markdown_renders_to_html() {
    let html = markdown_to_html("**bold** and a [link](https://example.com)");
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("href=\"https://example.com\""));
}

#[test]
fn substitute_replaces_known_tokens_only() {
    let v = vars(&[("name", "Ada")]);
    assert_eq!(
        substitute("Hi {{name}}, {{unknown}}", &v),
        "Hi Ada, {{unknown}}"
    );
}

#[test]
fn substitute_does_not_rescan_inserted_values() {
    // A value containing another token must not trigger a second substitution.
    let v = vars(&[("a", "{{b}}"), ("b", "SECRET")]);
    assert_eq!(substitute("{{a}}", &v), "{{b}}");
}

#[test]
fn catalog_falls_back_per_key() {
    let cat = Catalog::new(
        vec![
            (
                "en".to_string(),
                json!({ "invite": { "subject": "Invited", "body": "Hello" } }),
            ),
            (
                "es".to_string(),
                json!({ "invite": { "subject": "Invitado" } }),
            ),
        ],
        "en",
    );
    // Present in es.
    assert_eq!(cat.get("es", "invite.subject").as_deref(), Some("Invitado"));
    // Missing in es -> falls back to en.
    assert_eq!(cat.get("es", "invite.body").as_deref(), Some("Hello"));
    // Unknown locale -> falls back to en entirely.
    assert_eq!(cat.get("zz", "invite.subject").as_deref(), Some("Invited"));
    // Unknown key -> None.
    assert_eq!(cat.get("en", "invite.missing"), None);
}

#[test]
fn render_escapes_values_in_html_but_not_in_text() {
    let tmpl = EmailTemplate {
        layout: LAYOUT,
        subject: "Hi {{name}}",
        body_md: "Welcome **{{name}}**",
        cta: None,
        footer_md: None,
    };
    let rendered = render(&tmpl, &vars(&[("name", "<script>"), ("title", "T")]));

    // Subject keeps the raw value.
    assert_eq!(rendered.subject, "Hi <script>");
    // HTML body escapes the interpolated value.
    assert!(rendered
        .html_body
        .contains("Welcome <strong>&lt;script&gt;</strong>"));
    assert!(!rendered.html_body.contains("<script>"));
    // Text body keeps the raw value.
    assert!(rendered.text_body.contains("Welcome **<script>**"));
}

#[test]
fn render_appends_cta_to_both_bodies() {
    let tmpl = EmailTemplate {
        layout: LAYOUT,
        subject: "S",
        body_md: "Body",
        cta: Some(Cta {
            label: "Accept Invite".to_string(),
            url: "https://example.com/i?t=a&b".to_string(),
        }),
        footer_md: Some("Expires soon"),
    };
    let rendered = render(&tmpl, &vars(&[("title", "T")]));

    // HTML CTA is a styled button with an escaped href, plus the layout/footer slots filled.
    assert!(rendered
        .html_body
        .contains("<a class=\"btn\" href=\"https://example.com/i?t=a&amp;b\">Accept Invite</a>"));
    assert!(rendered.html_body.contains("<footer><p>Expires soon</p>"));
    assert!(!rendered.html_body.contains("{{content}}"));
    assert!(!rendered.html_body.contains("{{footer}}"));

    // Text CTA is "label: url" with the raw url, and the footer is appended.
    assert!(rendered
        .text_body
        .contains("Accept Invite: https://example.com/i?t=a&b"));
    assert!(rendered.text_body.contains("Expires soon"));
}

#[test]
fn html_escape_covers_all_five() {
    assert_eq!(html_escape("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
}
