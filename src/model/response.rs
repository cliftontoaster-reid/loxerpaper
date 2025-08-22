/*
 * loxerpaper - Automatic wallpaper fetcher and desktop background manager
 * Copyright (C) 2025  Clifton Toaster Reid
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

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Response {
  pub api_key: String,
  pub r#type: String,
  pub text: String,
}

impl Response {
  pub fn new(
    api_key: impl Into<String>,
    r#type: impl Into<String>,
    text: impl Into<String>,
  ) -> Self {
    Response {
      api_key: api_key.into(),
      r#type: r#type.into(),
      text: text.into(),
    }
  }
}
