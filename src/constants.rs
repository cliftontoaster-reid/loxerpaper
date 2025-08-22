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

/// Construct the full URL for a link JSON by id using the `BASE_URL` constant.
///
/// Examples:
///
/// ```rust
/// use loxerpaper::constants::{link_url, response_url, user_url};
/// assert_eq!(link_url(42), "https://walltaker.joi.how/api/links/42.json");
/// assert_eq!(link_url("abc"), "https://walltaker.joi.how/api/links/abc.json");
/// assert_eq!(response_url("abc"), "https://walltaker.joi.how/api/links/abc/response.json");
/// assert_eq!(user_url("me", "key"), "https://walltaker.joi.how/api/users/me.json?api_key=key");
/// ```
pub fn link_url(id: impl ToString) -> String {
  format!("{}links/{}.json", BASE_URL, id.to_string())
}

/// Public base URL constant for callers who want a default base.
pub const BASE_URL: &str = "https://walltaker.joi.how/api/";

/// Construct the URL for the responses of a link id.
pub fn response_url(id: impl ToString) -> String {
  format!("{}links/{}/response.json", BASE_URL, id.to_string())
}

/// Construct the URL for a user with an API key.
pub fn user_url(username: impl ToString, api_key: impl ToString) -> String {
  format!(
    "{}users/{}.json?api_key={}",
    BASE_URL,
    username.to_string(),
    api_key.to_string()
  )
}

/// Construct the URL for a user, allowing an optional API key.
pub fn user_url_opt(username: impl ToString, api_key: Option<impl ToString>) -> String {
  match api_key {
    Some(k) => user_url(username, k),
    None => format!("{}users/{}.json", BASE_URL, username.to_string()),
  }
}

pub const DISCORD_CLIENT_ID: &str = "123456789012345678";

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builds_url_from_integer() {
    assert_eq!(link_url(42), "https://walltaker.joi.how/api/links/42.json");
  }

  #[test]
  fn builds_url_from_str() {
    assert_eq!(
      link_url("abc"),
      "https://walltaker.joi.how/api/links/abc.json"
    );
  }

  #[test]
  fn builds_response_url() {
    assert_eq!(
      response_url("abc"),
      "https://walltaker.joi.how/api/links/abc/response.json"
    );
  }

  #[test]
  fn builds_user_url_with_api_key() {
    assert_eq!(
      user_url("me", "key"),
      "https://walltaker.joi.how/api/users/me.json?api_key=key"
    );
  }

  #[test]
  fn empty_id_is_allowed() {
    assert_eq!(link_url(""), "https://walltaker.joi.how/api/links/.json");
  }
}
