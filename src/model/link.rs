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

use serde::Deserialize;

/// Known response types from the API. Unknown values are captured as `Other(String)`.
#[derive(Debug, PartialEq)]
pub enum ResponseType {
  Horny,
  Disgust,
  Came,
  Other(String),
}

impl<'de> Deserialize<'de> for ResponseType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
      "horny" => Ok(ResponseType::Horny),
      "disgust" => Ok(ResponseType::Disgust),
      "came" => Ok(ResponseType::Came),
      other => Ok(ResponseType::Other(other.to_string())),
    }
  }
}

/// Representation of a link returned by the API.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Link {
  pub id: i64,
  /// ISO-8601 timestamp when the link expires (may be null)
  pub expires: Option<String>,
  pub username: String,
  pub terms: Option<String>,
  pub blacklist: Option<String>,
  pub post_url: Option<String>,
  pub post_thumbnail_url: Option<String>,
  pub post_description: Option<String>,
  pub created_at: Option<String>,
  pub updated_at: Option<String>,
  pub set_by: Option<String>,
  /// Can be 'horny', 'disgust', 'came' or null
  pub response_type: Option<ResponseType>,
  pub response_text: Option<String>,
  pub online: Option<bool>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn deserialize_sample_link() {
    let json = r#"
		{
		  "id": 1,
		  "expires": "2025-03-05T00:00:00.000Z",
		  "username": "gray",
		  "terms": "I'm trying out something new, break this please! :)",
		  "blacklist": "feet blood",
		  "post_url": "https://static1.e621.net/data/5d/87/5d87428c4839b0dc7d585b87a25af61a.png",
		  "post_thumbnail_url": "https://static1.e621.net/data/preview/5d/87/5d87428c4839b0dc7d585b87a25af61a.jpg",
		  "post_description": "",
		  "created_at": "2022-03-08T01:01:50.142Z",
		  "updated_at": "2022-03-13T21:39:01.828Z",
		  "set_by": "name",
		  "response_type": "horny",
		  "response_text": "HUFF wow",
		  "online": true
		}
		"#;

    let link: Link = serde_json::from_str(json).expect("deserialization failed");

    assert_eq!(link.id, 1);
    assert_eq!(link.username, "gray");
    assert_eq!(link.response_type, Some(ResponseType::Horny));
    assert_eq!(link.response_text.as_deref(), Some("HUFF wow"));
    assert_eq!(link.online, Some(true));
  }
}
