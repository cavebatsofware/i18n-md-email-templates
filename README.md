# i18n-md-email-templates

Render localized transactional emails from markdown-authored copy.

You supply per-locale string catalogs (any JSON shape), an HTML layout shell, and a map
of runtime values. The crate resolves the locale, renders the markdown body to styled
HTML, and produces a subject, an HTML body, and a plaintext body. The consumer owns the
content (catalogs, layout, branding); the crate owns the mechanism.

## Security model

Template copy (layout, body, footer, subject) is treated as **trusted**: it is yours, not
user input, so it is not sanitized and may contain markdown and inline HTML. Interpolated
values (`Vars`) are treated as **untrusted**: they are HTML-escaped for the HTML outputs
and left raw for the plaintext output. Escaping happens *after* the markdown is rendered,
which prevents both double-escaping and value-driven markup injection.

Do not feed user-generated content through `markdown_to_html`; it does not sanitize. Pass
user-supplied data as `Vars` values instead, which are escaped.

## Usage

```rust
use std::collections::BTreeMap;
use i18n_md_email_templates::{render, Catalog, Cta, EmailTemplate};

// 1. Build a catalog from per-locale JSON (e.g. embedded with include_str!).
let catalog = Catalog::new(
    vec![
        ("en".into(), serde_json::from_str(EN_JSON)?),
        ("es".into(), serde_json::from_str(ES_JSON)?),
    ],
    "en", // fallback locale
);

// 2. Pull the localized strings for the recipient's locale.
let locale = "es";
let subject = catalog.get(locale, "invite.subject").unwrap();
let body = catalog.get(locale, "invite.body").unwrap();
let cta_label = catalog.get(locale, "invite.cta").unwrap();
let footer = catalog.get(locale, "invite.footer").unwrap();

// 3. Assemble the template and runtime values, then render.
let tmpl = EmailTemplate {
    layout: LAYOUT_HTML,         // shell with {{content}} / {{footer}} slots
    subject: &subject,
    body_md: &body,
    cta: Some(Cta { label: cta_label, url: invite_url.clone() }),
    footer_md: Some(&footer),
};
let mut vars: BTreeMap<&str, String> = BTreeMap::new();
vars.insert("site", site_name);
vars.insert("inviter", inviter_email);

let email = render(&tmpl, &vars);
// email.subject, email.html_body, email.text_body
```

## Tokens

- `{{token}}` placeholders are replaced in a single left-to-right pass; inserted values
  are never rescanned, and unknown tokens are left in place.
- The layout shell uses `{{content}}` (rendered body + CTA) and an optional `{{footer}}`
  slot, filled within the same single pass so a user value containing the literal text
  `{{content}}`/`{{footer}}` cannot collide with them.

## Email-client styling

`render` emits the body with the layout's `<style>` block intact. Many email clients
ignore embedded stylesheets, so call [`inline_css`] on the HTML body to rewrite those
rules into element `style=` attributes (and strip the `<style>` tag). No remote or
filesystem stylesheet loading is performed.

```rust
let email = render(&tmpl, &vars);
let html = i18n_md_email_templates::inline_css(&email.html_body)
    .unwrap_or(email.html_body);
```

The plaintext body (`text_body`) is produced with [`markdown_to_text`], which drops
emphasis markers, code fences, inline HTML, and link URLs (keeping link text), so the
text/plain part reads cleanly.

## License

LGPL-3.0-or-later. See [LICENSE.md](LICENSE.md).
