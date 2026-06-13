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

//! Render localized transactional emails from markdown-authored copy.
//!
//! The crate owns the *mechanism* and the consumer owns the *content*: you supply
//! per-locale string catalogs (any JSON shape), an HTML layout shell, and a map of
//! runtime values; the crate resolves the locale, renders the markdown body to styled
//! HTML, and produces a [`RenderedEmail`] with a subject, an HTML body, and a plaintext
//! body.
//!
//! Template copy is treated as trusted (it is yours, not user input) and is not
//! sanitized, so it may contain markdown and inline HTML. Interpolated [`Vars`] values
//! are treated as untrusted: they are HTML-escaped for the HTML outputs and left raw for
//! the plaintext output. Escaping happens *after* the markdown is rendered, which avoids
//! both double-escaping and letting a value inject markup.

mod catalog;
mod render;

#[cfg(test)]
mod tests;

pub use catalog::Catalog;
pub use render::{
    html_escape, inline_css, markdown_to_html, markdown_to_text, render, substitute, Cta,
    EmailTemplate, RenderedEmail, Vars,
};
