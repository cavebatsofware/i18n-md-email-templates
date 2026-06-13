# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
