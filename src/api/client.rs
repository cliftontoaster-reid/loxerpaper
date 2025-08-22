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

use crate::constants::{link_url, response_url, user_url_opt};
use crate::model::config::Config;
use crate::model::{link::Link, response::Response, user::User};

/// Simple API client that holds a base URL and a reusable reqwest client.
#[derive(Clone)]
pub struct ApiClient {
  client: reqwest::Client,
  pub config: Config,
}

impl ApiClient {
  /// Create a new client with an explicit base URL.
  pub fn new(config: Config) -> Self {
    ApiClient {
      config,
      client: reqwest::Client::new(),
    }
  }

  /// Create a client from the typed `Config` produced by `model::config`.
  pub fn from_config(cfg: &Config) -> Self {
    ApiClient::new(cfg.clone())
  }

  /// Get a link by id.
  pub async fn get_link(&self, id: i64) -> Result<Link, reqwest::Error> {
    let url = link_url(id);
    let resp = self.client.get(&url).send().await?.error_for_status()?;
    let link = resp.json::<Link>().await?;
    Ok(link)
  }

  /// Post a response for a given link.
  pub async fn post_response(
    &self,
    id: i64,
    response: &Response,
  ) -> Result<Link, Box<dyn std::error::Error>> {
    // if token == "your_token" or is None, error
    if self
      .config
      .feed
      .as_ref()
      .unwrap()
      .token
      .as_ref()
      .map(|s| s == "your_token")
      .unwrap_or(false)
    {
      return Err("Unauthorized: missing or placeholder token".into());
    }

    let url = response_url(id);
    let resp = self
      .client
      .post(&url)
      .json(response)
      .send()
      .await?
      .error_for_status()?;
    let link = resp.json::<Link>().await?;
    Ok(link)
  }

  /// Get user details; api_key is optional.
  pub async fn get_user(
    &self,
    username: &str,
    api_key: Option<&str>,
  ) -> Result<User, reqwest::Error> {
    let api_key_owned = api_key.map(|s| s.to_string());
    let url = user_url_opt(username, api_key_owned);
    let resp = self.client.get(&url).send().await?.error_for_status()?;
    let user = resp.json::<User>().await?;
    Ok(user)
  }

  /// Get the base URL of the API client.
  pub fn base_url(&self) -> &str {
    &self.config.base.as_ref().unwrap().base.as_ref().unwrap()
  }

  /// Get the link ID from the API client.
  pub fn link_id(&self) -> i64 {
    self.config.feed.as_ref().unwrap().feed.unwrap()
  }
}
