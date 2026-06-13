# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-13

### Added

- `inline_css`: inline a document's `<style>` rules into element `style=` attributes
  (via `css-inline`, no remote/filesystem loading) and strip the `<style>` tag, so the
  rendered HTML survives email clients that ignore embedded stylesheets.
- `markdown_to_text`: render markdown to clean plaintext (drops emphasis markers, code
  fences, inline HTML, and link URLs; keeps link text), for readable text bodies.

### Changed

- `render` now builds `text_body` via `markdown_to_text` instead of emitting the raw
  markdown source, so plaintext parts no longer leak `**`, fences, or inline HTML tags.
- `render` fills the layout's `{{content}}` and `{{footer}}` slots within the single-pass
  `substitute` rather than a trailing `String::replace`, so an interpolated value
  containing the literal text `{{content}}`/`{{footer}}` can no longer collide with the
  layout slots.

## [0.1.0] - 2026-06-13

### Added

- `Catalog`: per-locale JSON string lookup by dotted key, with per-key fallback to a
  configured fallback locale.
- `render`: produces a `RenderedEmail` (subject, HTML body, plaintext body) from an
  `EmailTemplate` (layout shell, subject, markdown body, optional `Cta`, optional footer)
  and a map of runtime `Vars`. Values are HTML-escaped for the HTML outputs after markdown
  rendering and left raw for the plaintext output.
- `markdown_to_html`, `substitute`, and `html_escape` building blocks for callers that
  want finer control.
