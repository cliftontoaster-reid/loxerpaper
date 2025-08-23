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

use std::sync::{
  Arc,
  atomic::{AtomicI64, Ordering},
};
use std::thread;

use notify_rust::Notification as NotifyRustNotification;

use crate::api::ApiClient;
use crate::api::DesktopApi;
use crate::model::response::Response;

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
  client: &ApiClient,
  desktop: Arc<dyn DesktopApi>,
  current_id: Arc<AtomicI64>,
  link_id: i64,
  post_id: i64,
  username: String,
  api_key: String,
  image_path: std::path::PathBuf,
) {
  // Clone what we need into the thread
  let client_thread = client.clone();
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

    // Show and get handle
    let handle = match n2.show() {
      Ok(h) => h,
      Err(e) => {
        eprintln!("notify show error: {e}");
        return;
      }
    };

    // notify-rust's wait_for_action expects a closure invoked when an action occurs.
    // We'll use a channel to receive the action from that closure.
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    handle.wait_for_action(move |action_id| {
      // send action id back to the thread, ignoring errors
      let _ = tx.send(action_id.to_string());
    });

    // Wait up to 5 minutes for an action
    if let Ok(action_id) = rx.recv_timeout(std::time::Duration::from_secs(300)) {
      if action_id == "__closed" {
        // Try using the provided DesktopApi to open the image. If it fails, fall back to a notify-rust
        // notification to inform the user.
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
        return;
      }
      let parts: Vec<&str> = action_id.splitn(2, '-').collect();
      if parts.len() != 2 {
        eprintln!("unexpected action id: {action_id}");
        return;
      }
      let reaction = parts[0].to_string();
      let reacted_post_id = parts[1].parse::<i64>().unwrap_or(-1);

      // Check that the post id matches current image id
      let current = current_id.load(Ordering::SeqCst);
      if current != reacted_post_id {
        // Send critical notification: image changed
        let notif = crate::api::Notification::builder("Image changed")
          .body(
            "The image you reacted to is no longer the current background and cannot be reviewed.",
          )
          .urgency(crate::api::Urgency::Critical)
          .build();
        let _ = desktop.send_notification(&notif);
        return;
      }

      // Build a Response and post it using a small tokio runtime so it doesn't block
      // api_key is owned by the thread now
      let resp = Response::new(api_key.clone(), reaction.clone(), "".to_string());

      // We need to run the async post_response; create a temporary runtime
      let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

      let post_fut = client_thread.post_response(link_id, &resp);
      if let Err(e) = rt.block_on(post_fut) {
        // Send critical notification: failed to post response
        let notif = crate::api::Notification::builder("Failed to post response")
          .body(format!("Failed to post response: {}", e))
          .urgency(crate::api::Urgency::Critical)
          .build();
        let _ = desktop.send_notification(&notif);
      } else {
        // Send a notification for the successful response
        let notif = crate::api::Notification::builder("Response posted")
          .body(format!("Successfully posted response: {}", resp.r#type))
          .urgency(crate::api::Urgency::Normal)
          .build();
        let _ = desktop.send_notification(&notif);
      }
    }
  });
}
