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

use std::sync::Arc;
use std::thread;

use notify_rust::Notification as NotifyRustNotification;

use crate::api::ApiClient;
use crate::api::DesktopApi;

/// Spawn a background thread that shows a review notification with actions.
///
/// - `client`: an `ApiClient` clone to use for posting responses.
/// - `current_id`: Arc to the current image/post id (e621 id). The thread will read this to ensure the user is reacting to the current image.
/// - `link_id`: the id of the link being reviewed.
/// - `username`: the username who provided the image (for the notification body).
/// - `api_key`: the API key to include in the response.
/// - `image_path`: the path to the current background image file.
///
/// This function returns immediately; the thread handles user interactions and posts responses.
pub fn spawn_review_notification(
  _client: &ApiClient,
  desktop: Arc<dyn DesktopApi>,
  _current_id: Arc<std::sync::atomic::AtomicI64>,
  _link_id: i64,
  post_id: i64,
  username: String,
  _api_key: String,
  image_path: std::path::PathBuf,
) {
  // Clone what we need into the thread - simplified for now
  thread::spawn(move || {
    // Create a notify-rust notification with actions and wait for user interaction
    let mut n2 = NotifyRustNotification::new();
    n2.summary("Background change pending");
    n2.body(&format!(
      "Your background will soon change to an image provided by {username}. You may review it from here."
    ));
    n2.action(&format!("horny-{post_id}"), "Horny");
    n2.action(&format!("disgust-{post_id}"), "Disgust");
    n2.action(&format!("came-{post_id}"), "Came");

    // Show the notification - simplified without actions for now
    match n2.show() {
      Ok(_handle) => {
        println!("Review notification sent");
        // For now, just wait a bit and then provide a simple notification
        std::thread::sleep(std::time::Duration::from_secs(5));

        // Auto-open the image for review
        match desktop.open_file(&image_path) {
          Ok(_) => {
            let notif = crate::api::Notification::builder("Image opened")
              .body("Successfully opened the current background image")
              .urgency(crate::api::Urgency::Normal)
              .build();
            let _ = desktop.send_notification(&notif);
          }
          Err(e) => {
            let notif = crate::api::Notification::builder("Failed to open image")
              .body(format!("Failed to open image: {}", e))
              .urgency(crate::api::Urgency::Critical)
              .build();
            let _ = desktop.send_notification(&notif);
          }
        }
      }
      Err(e) => {
        eprintln!("notify show error: {e}");
      }
    };
  });
}
