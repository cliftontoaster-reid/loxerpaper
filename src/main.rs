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

mod api;
mod constants;
mod model;
use std::{
  fs,
  io::{self, BufRead, BufReader, Write as _},
  sync::Arc,
  thread,
};

use model::config::Config;

use crate::api::{ApiClient, spawn_review_notification};

fn print_gpl_notice() {
  println!("loxerpaper  Copyright (C) 2025  Clifton Toaster Reid");
  println!("This program comes with ABSOLUTELY NO WARRANTY; for details type 'show w'.");
  println!("This is free software, and you are welcome to redistribute it");
  println!("under certain conditions; type 'show c' for details.");
  println!();
}

fn show_warranty() {
  println!("THERE IS NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY");
  println!("APPLICABLE LAW. EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT");
  println!("HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM \"AS IS\" WITHOUT");
  println!("WARRANTY OF ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT");
  println!("LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR");
  println!("A PARTICULAR PURPOSE. THE ENTIRE RISK AS TO THE QUALITY AND");
  println!("PERFORMANCE OF THE PROGRAM IS WITH YOU. SHOULD THE PROGRAM PROVE");
  println!("DEFECTIVE, YOU ASSUME THE COST OF ALL NECESSARY SERVICING, REPAIR OR");
  println!("CORRECTION.");
  println!();
}

fn show_conditions() {
  println!("This program is free software: you can redistribute it and/or modify");
  println!("it under the terms of the GNU General Public License as published by");
  println!("the Free Software Foundation, either version 3 of the License, or");
  println!("(at your option) any later version.");
  println!();
  println!("This program is distributed in the hope that it will be useful,");
  println!("but WITHOUT ANY WARRANTY; without even the implied warranty of");
  println!("MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the");
  println!("GNU General Public License for more details.");
  println!();
  println!("You should have received a copy of the GNU General Public License");
  println!("along with this program. If not, see <https://www.gnu.org/licenses/>.");
  println!();
}

fn handle_stdin_commands() {
  let stdin = io::stdin();
  let reader = BufReader::new(stdin);

  for line in reader.lines() {
    if let Ok(input) = line {
      let trimmed = input.trim().to_lowercase();
      match trimmed.as_str() {
        "show w" => show_warranty(),
        "show c" => show_conditions(),
        "help" => {
          println!("Available commands:");
          println!("  show w - Show warranty information");
          println!("  show c - Show license conditions");
          println!("  help   - Show this help message");
          println!("  quit   - Exit the program");
          println!();
        }
        "quit" | "exit" => {
          println!("Goodbye!");
          std::process::exit(0);
        }
        "" => {} // Ignore empty lines
        _ => {
          println!(
            "Unknown command: '{}'. Type 'help' for available commands.",
            input.trim()
          );
        }
      }
    }
  }
}

fn hash_str(s: &str) -> i64 {
  // Polynomial rolling hash:
  // hash(s) = sum_{i=0..n-1} (s[i]+1) * base^{n-1-i}  (computed iteratively)
  const MOD: i64 = 1_000_000_007;
  const BASE: i64 = 257;
  let mut h: i64 = 0;
  for b in s.bytes() {
    // use (b as i64 + 1) so that '\0' and other zeros contribute
    h = (h.wrapping_mul(BASE) + (b as i64 + 1)) % MOD;
  }
  h
}

#[tokio::main]
async fn main() {
  // Print GPL notice
  print_gpl_notice();

  // Spawn stdin handler in background thread
  thread::spawn(|| {
    handle_stdin_commands();
  });

  // Try to load a local `config.toml` in the cwd; fall back to defaults.
  let cfg = Config::load();

  if let Err(e) = cfg {
    #[cfg(debug_assertions)]
    {
      eprintln!("Config load error: {}", e);
    }
    // We try and find the config file in the default locations.
    if let Err(e2) = Config::try_import() {
      eprintln!("Failed to import config file: {}", e2);
      // We then start the query and write the config file.
      let new_cfg = Config::query_config().unwrap();

      // We then write the new config to the config file.
      fs::write(Config::path(), toml::to_string(&new_cfg).unwrap()).unwrap();
      println!(
        "Config file created at {}, please restart the application.",
        Config::path().display()
      );
      return;
    } else {
      println!("Config file imported, please restart the application.");
    }
    return;
  }

  let cfg_data = cfg.unwrap();

  let client = ApiClient::from_config(&cfg_data);

  // Then the tool should loop, pinging the API for updates (link) and apply changes if a needed, sending a notification
  // and then waiting for the user defined period of time to restart the loop.

  let should_keep = client
    .config
    .preferences
    .as_ref()
    .unwrap()
    .save_locally
    .unwrap_or(false);
  let link_id = client.config.feed.as_ref().unwrap().feed.unwrap();
  let sleep_time = tokio::time::Duration::from_secs(
    cfg_data
      .preferences
      .as_ref()
      .unwrap()
      .interval
      .unwrap_or(60),
  );
  let api_key = cfg_data
    .feed
    .unwrap()
    .token
    .unwrap_or("your_token".to_string());

  let current_id = Arc::new(std::sync::atomic::AtomicI64::new(-1));

  loop {
    // Ping the API for updates (link)
    let updates = client.get_link(link_id).await;
    match updates {
      Ok(link) => {
        // We first check if this is a new url with the post id.
        let post_url = link.post_url.unwrap();

        // Try to parse the URL and extract the final path segment (the filename).
        // Fallback to a safe replacement when parsing fails.
        let filename = match url::Url::parse(&post_url).ok().and_then(|u| {
          u.path_segments()
            .and_then(|s| s.last().map(|s| s.to_string()))
        }) {
          Some(f) => f,
          None => {
            // Fallback: use the whole URL but sanitize characters so we don't produce
            // a filename that contains ':' or '/'.
            post_url.replace(
              |c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'),
              "_",
            )
          }
        };

        // Derive a stem and extension from the filename.
        let path_like = std::path::Path::new(&filename);
        let stem = path_like
          .file_stem()
          .and_then(|s| s.to_str())
          .unwrap_or("image");
        let ext = path_like
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("png");

        // Sanitize stem to remove any unexpected characters.
        let sanitize: String = stem
          .chars()
          .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
              c
            } else {
              '_'
            }
          })
          .collect();

        let hashed_id = hash_str(&sanitize);

        if current_id.load(std::sync::atomic::Ordering::SeqCst) == hashed_id {
          // We have the same image, we print a debug message, and return.
          println!("No new image, current is still id {}", hashed_id);
          // Wait before next poll
          tokio::time::sleep(sleep_time).await;
          continue;
        }

        // We have a new image, we download it and set it as the background.
        // Build the target filename from the sanitized stem and extension.
        let target_filename = format!("{}.{}", sanitize, ext);

        // Build the target path for the downloaded image. Don't canonicalize the full
        // file path (the file won't exist yet) and avoid using a TempDir that is
        // immediately dropped (which would delete the directory). Instead use the
        // system temp directory for transient files.
        let path = if should_keep {
          let mut dir = dirs_next::picture_dir().unwrap_or(std::path::PathBuf::from("."));
          dir.push("WallTaker");
          dir.push(&target_filename);
          dir
        } else {
          let mut dir = std::env::temp_dir();
          dir.push(&target_filename);
          dir
        };

        // Ensure the parent directory exists before creating the file.
        let parent = path
          .parent()
          .map(|p| p.to_path_buf())
          .unwrap_or(std::env::current_dir().unwrap());
        fs::create_dir_all(&parent).unwrap();

        // We now download the image using reqwest
        let response = reqwest::get(&post_url).await;
        match response {
          Ok(resp) => {
            let mut file = fs::File::create(&path).unwrap();
            let content = resp.bytes().await.unwrap();
            file.write_all(&content).unwrap();
          }
          Err(e) => {
            eprintln!("Failed to download image: {}", e);
            // Wait before next poll
            tokio::time::sleep(sleep_time).await;
            continue;
          }
        }

        // We now send the notification and edit the current ID
        current_id.store(hashed_id, std::sync::atomic::Ordering::SeqCst);
        spawn_review_notification(
          &client,
          current_id.clone(),
          link_id,
          hashed_id,
          link.set_by.unwrap_or("unknown".to_string()),
          api_key.clone(),
          path.clone(),
        );

        // We now set the background.
        #[cfg(target_os = "linux")]
        {
          let desktop_env = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
          if desktop_env.contains("GNOME") {
            use crate::api::{DesktopApi, GnomeDesktopApi};

            let _ = GnomeDesktopApi::new().change_background(&path);
          }
        }
        #[cfg(target_os = "macos")]
        {
          unimplemented!("macOS support is not planned");
        }
      }
      Err(e) => {
        eprintln!("Failed to fetch link: {}", e);
        // Wait before next poll on error
        tokio::time::sleep(sleep_time).await;
        continue;
      }
    }

    // Wait for the user defined period of time before next iteration
    tokio::time::sleep(sleep_time).await;
  }
}
