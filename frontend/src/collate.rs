/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2026 Andre Onuki
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Name ordering for display, using the browser's locale-aware comparison so accented names
//! sort next to their unaccented letters ("Émile" between "Eli" and "Fred", not after "Zoe").

use js_sys::{Array, JsString, Object};
use std::cmp::Ordering;

/// Compare two names the way the browser's `String.prototype.localeCompare` does, in the
/// user's default locale.
pub fn compare_names(a: &str, b: &str) -> Ordering {
    JsString::from(a)
        .locale_compare(b, &Array::new(), &Object::new())
        .cmp(&0)
}
