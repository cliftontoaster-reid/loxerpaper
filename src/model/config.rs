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

use dirs_next::{config_dir, picture_dir};
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use url::Url;

use crate::constants::DISCORD_CLIENT_ID;

/// Base section from the exported config. It's intentionally small because the
/// app itself knows the base endpoint; we still keep this to mirror the
/// original exported file.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseConfig {
  pub base: Option<String>,
}

/// Feed section: which link id to watch.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FeedConfig {
  pub feed: Option<i64>,
  #[serde(default)]
  pub token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ResizeMode {
  Fit,
  Crop,
}

/// Preferences section for various user settings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Preferences {
  pub interval: Option<u64>,
  pub mode: Option<ResizeMode>,
  #[serde(rename = "discordPresence")]
  pub discord_presence: Option<bool>,
  #[serde(rename = "discordClientId")]
  #[serde(default)]
  pub discord_client_id: Option<String>,
  #[serde(rename = "saveLocally")]
  pub save_locally: Option<bool>,
  pub notifications: Option<bool>,
}

/// Top-level typed configuration that mirrors the exported TOML layout.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
  #[serde(rename = "Base")]
  pub base: BaseConfig,
  #[serde(rename = "Feed")]
  pub feed: FeedConfig,
  #[serde(rename = "Preferences")]
  pub preferences: Preferences,
}

impl Config {
  /// Parse a TOML string into Config.
  pub fn from_str(toml: &str) -> Result<Self, toml::de::Error> {
    toml::from_str(toml)
  }

  // Get the path to the config file.
  pub fn path() -> PathBuf {
    if let Some(path) = config_dir() {
      path.join("loxerpaper/config.toml")
    } else {
      current_dir()
        .expect("Failed to get current directory")
        .join("loxerpaper/walltaker.toml")
    }
  }

  pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
    let path = Self::path();

    if path.exists() {
      let contents = fs::read_to_string(&path)?;
      Self::from_str(&contents).map_err(|e| e.into())
    } else {
      Err("Config file not found".into())
    }
  }

  // This looks for a file named 'walltaker.toml' as it is what the user would have downloaded.
  //
  // Therefore we look into the following folders in order :
  //
  // Downloads -> Desktop -> <user_home>
  //
  // If found we then move it to the new correct location, and indicate the new location of the config to the user.
  pub fn try_import() -> Result<bool, Box<dyn std::error::Error>> {
    let docs = dirs_next::document_dir().ok_or("Failed to get documents directory")?;
    let down = dirs_next::download_dir().ok_or("Failed to get downloads directory")?;
    let home = dirs_next::home_dir().ok_or("Failed to get home directory")?;

    fn copy(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
      let dest = Config::path();
      if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
      }
      fs::copy(path, &dest)?;
      Ok(())
    }

    let paths = vec![
      down.join("walltaker.toml"),
      docs.join("walltaker.toml"),
      home.join("walltaker.toml"),
    ];

    for path in paths {
      if path.exists() {
        copy(path)?;
        return Ok(true);
      } else {
        #[cfg(debug_assertions)]
        {
          println!("Checked {path:?}, not found");
        }
      }
    }

    // We didn't find it, look into all non hidden directories in the home folder. Recursively.
    let mut to_visit = vec![home];

    while let Some(dir) = to_visit.pop() {
      for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && !entry.file_name().to_string_lossy().starts_with('.') {
          let path = entry.path().join("walltaker.toml");
          if path.exists() {
            copy(path)?;
            return Ok(true);
          }
          to_visit.push(entry.path());
        }
      }
    }

    Ok(false)
  }

  pub fn query_config() -> Result<Self, Box<dyn std::error::Error>> {
    // We first ask the user his link url, as it contains both the base url and the link id.
    let link_url: Url = dialoguer::Input::<String>::new()
      .with_prompt("Enter the link URL")
      .validate_with(|input: &String| {
        // Validate the link URL format using `url::Url`
        Url::parse(input)
          .map(|_| ())
          .map_err(|_| "Invalid link URL")
      })
      .interact_text()?
      .parse::<Url>()?;

    // We then check if the last segment is a number
    let last_segment = link_url
      .path_segments()
      .and_then(|mut segments| segments.next_back())
      .ok_or("Invalid link URL: no path segments")?;

    // If it is a number, use that, if not, we ask for the link ID
    let link_id = if last_segment.parse::<i64>().is_ok() {
      last_segment.parse::<i64>().unwrap()
    } else {
      dialoguer::Input::<String>::new()
        .with_prompt("Enter your link ID")
        .validate_with(|input: &String| {
          input
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| "Invalid link ID")
        })
        .interact_text()?
        .parse::<i64>()
        .map_err(|_| Box::<dyn std::error::Error>::from("Failed to parse link id"))?
    };

    // We then ask the user to provide an api token, the user may choose to skip this step, if he does the value "your_token" will be used
    let api_token = dialoguer::Input::<String>::new()
      .with_prompt("Enter your API token (leave blank to use default)")
      .default("your_token".to_string())
      .interact_text()?;

    // We then ask how often it should update, how long to wait between pings
    let update_interval = dialoguer::Input::<String>::new()
      .with_prompt("Enter the update interval (in seconds)")
      .validate_with(|input: &String| {
        input
          .parse::<u64>()
          .map(|_| ())
          .map_err(|_| "Invalid update interval")
      })
      .default(10.to_string())
      .interact_text()?
      .parse::<u64>()
      .map_err(|_| Box::<dyn std::error::Error>::from("Failed to parse update interval"))?;

    // We then ask for the mode, either 'crop' or 'fit' by asking if the images should be resized
    let resize_mode = if dialoguer::Confirm::new()
      .with_prompt(
        "Would you like to resize images to fit your screen? (otherwise they will be cropped by your system)",
      )
      .default(true)
      .interact()?
    {
      ResizeMode::Fit
    } else {
      ResizeMode::Crop
    };

    // We then ask if the user wants to store the images or just store them temporarily
    let store_images = dialoguer::Confirm::new()
      .with_prompt("Would you like to store the images on disk for future 'review'?")
      .default(true)
      .interact()?;

    // If yes, we ask where, default being picture/WallTaker
    let image_path = if store_images {
      Some(
        dialoguer::Input::<String>::new()
          .with_prompt("Enter the path to the folder where images should be stored")
          .default(
            picture_dir()
              .ok_or("example")?
              .join("WallTaker")
              .to_string_lossy()
              .to_string(),
          )
          .interact_text()?,
      )
    } else {
      None
    };

    // We then ask wether or not discord rich presence should be enabled
    let discord_rich_presence = dialoguer::Confirm::new()
      .with_prompt("Would you like us to use Discord rich presence to advertise your activity?")
      .default(true)
      .interact()?;

    // If yes, we ask for the application ID, the default is the public Discord client ID
    let discord_app_id = if discord_rich_presence {
      Some(
        dialoguer::Input::<String>::new()
          .with_prompt("Enter your Discord application ID")
          .default(DISCORD_CLIENT_ID.to_string())
          .interact_text()?,
      )
    } else {
      None
    };

    // We then ask if the user wants to enable notifications
    let enable_notifications = dialoguer::Confirm::new()
      .with_prompt("Would you like to receive notifications when the background is changed?")
      .default(true)
      .interact()?;

    // We then build the config
    let config = Config {
      base: Some(BaseConfig {
        base: Some(link_url.to_string()),
      }),
      feed: Some(FeedConfig {
        feed: Some(link_id),
        token: Some(api_token),
      }),
      preferences: Some(Preferences {
        interval: Some(update_interval),
        mode: Some(resize_mode),
        discord_presence: Some(discord_rich_presence),
        discord_client_id: discord_app_id,
        save_locally: image_path.map(|p| !p.is_empty()),
        notifications: Some(enable_notifications),
      }),
    };

    Ok(config)
  }
}
