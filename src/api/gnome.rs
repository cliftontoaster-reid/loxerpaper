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

use std::path::Path;
use std::process::Command;
// std::time::Duration not needed here

use notify_rust::Notification as NotifyRustNotification;

use crate::api::DesktopApi;
use crate::api::{DesktopApiError, DesktopCapabilities, Icon, Notification};

/// GNOME implementation of DesktopApi using notify-rust for notifications and gsettings for wallpaper.
pub struct GnomeDesktopApi {}

impl GnomeDesktopApi {
  pub fn new() -> Self {
    GnomeDesktopApi {}
  }
}

impl DesktopApi for GnomeDesktopApi {
  fn change_background(&self, image: &Path) -> Result<(), DesktopApiError> {
    if !image.exists() {
      return Err(DesktopApiError::InvalidNotification(format!(
        "image path {image:?} does not exist"
      )));
    }

    // Try to set via gsettings for org.gnome.desktop.background picture-uri
    // Use file:// URI
    let uri = format!("file://{}", image.display());

    // Set both light and dark mode wallpapers to ensure it works regardless of color scheme
    let status_light = Command::new("gsettings")
      .args(["set", "org.gnome.desktop.background", "picture-uri", &uri])
      .status()
      .map_err(DesktopApiError::Io)?;

    let status_dark = Command::new("gsettings")
      .args([
        "set",
        "org.gnome.desktop.background",
        "picture-uri-dark",
        &uri,
      ])
      .status()
      .map_err(DesktopApiError::Io)?;

    if status_light.success() && status_dark.success() {
      println!("Successfully changed wallpaper to {image:?}");
      Ok(())
    } else {
      Err(DesktopApiError::Backend(format!(
        "gsettings failed - light: {status_light}, dark: {status_dark}"
      )))
    }
  }

  fn capabilities(&self) -> DesktopCapabilities {
    DesktopCapabilities {
      notifications: true,
      actions: true,
      set_wallpaper: true,
      raw_icon_bytes: true,
      open_file: true,
    }
  }

  fn send_notification(&self, notification: &Notification) -> Result<(), DesktopApiError> {
    let mut n = NotifyRustNotification::new();
    n.summary(&notification.title);
    if let Some(body) = &notification.body {
      n.body(body);
    }

    if let Some(icon) = &notification.icon {
      match icon {
        Icon::Path(p) => {
          n.icon(p.to_string_lossy().as_ref());
        }
        Icon::Resource(name) => {
          n.icon(name);
        }
        Icon::Raw(bytes) => {
          // notify-rust doesn't accept raw bytes; write a temp file fallback
          if let Ok(mut tmp) = tempfile::Builder::new().suffix(".png").tempfile() {
            use std::io::Write;
            if tmp.write_all(bytes).is_ok()
              && let Ok(path) = tmp.into_temp_path().keep()
            {
              n.icon(path.to_string_lossy().as_ref());
            }
          }
        }
      }
    }

    // Map urgency
    match notification.urgency {
      crate::api::Urgency::Low => {
        n.hint(notify_rust::Hint::Urgency(notify_rust::Urgency::Low));
      }
      crate::api::Urgency::Normal => {
        n.hint(notify_rust::Hint::Urgency(notify_rust::Urgency::Normal));
      }
      crate::api::Urgency::Critical => {
        n.hint(notify_rust::Hint::Urgency(notify_rust::Urgency::Critical));
      }
    }

    // timeout
    if let Some(t) = notification.timeout {
      n.timeout(t.as_millis() as i32);
    }

    for action in &notification.actions {
      n.action(&action.id, &action.title);
    }

    n.show()
      .map_err(|e| DesktopApiError::Backend(format!("notify-rust error: {e}")))?;
    Ok(())
  }

  fn open_file(&self, file: &Path) -> Result<(), DesktopApiError> {
    if !file.exists() {
      return Err(DesktopApiError::InvalidNotification(format!(
        "file path {file:?} does not exist"
      )));
    }

    // Use xdg-open to open the file with the default application
    let status = Command::new("xdg-open")
      .arg(file)
      .status()
      .map_err(DesktopApiError::Io)?;

    if status.success() {
      println!("Successfully opened file {file:?}");
      Ok(())
    } else {
      Err(DesktopApiError::Backend(format!(
        "xdg-open failed with exit code: {status}"
      )))
    }
  }
}
