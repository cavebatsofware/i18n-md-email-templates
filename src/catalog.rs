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

use serde_json::Value;

/// A set of per-locale string catalogs with a fallback locale.
///
/// Each catalog is an arbitrary JSON object; strings are addressed by dotted key
/// (`"invite.subject"`) which walks nested objects. Lookup falls back to the configured
/// fallback locale per key, so a locale that is missing a single string still renders.
pub struct Catalog {
    locales: BTreeMap<String, Value>,
    fallback: String,
}

impl Catalog {
    /// Build a catalog from `(locale_code, parsed_json)` pairs and a fallback locale.
    pub fn new(locales: Vec<(String, Value)>, fallback: impl Into<String>) -> Self {
        Self {
            locales: locales.into_iter().collect(),
            fallback: fallback.into(),
        }
    }

    /// Resolve a dotted key for `locale`, falling back to the fallback locale's value
    /// for that key when the requested locale lacks it (or is unknown).
    pub fn get(&self, locale: &str, dotted_key: &str) -> Option<String> {
        self.lookup(locale, dotted_key)
            .or_else(|| self.lookup(&self.fallback, dotted_key))
    }

    fn lookup(&self, locale: &str, dotted_key: &str) -> Option<String> {
        let mut node = self.locales.get(locale)?;
        for part in dotted_key.split('.') {
            node = node.get(part)?;
        }
        node.as_str().map(str::to_string)
    }
}
