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

pub mod client;
#[cfg(target_os = "linux")]
pub mod gnome;
pub mod notify_helper;

#[cfg(windows)]
pub mod windows;

pub use client::ApiClient;
pub use notify_helper::spawn_review_notification;

#[cfg(target_os = "linux")]
pub use gnome::GnomeDesktopApi;

#[cfg(windows)]
pub use windows::WindowsDesktopApi;

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Creates a desktop API implementation appropriate for the current platform
pub fn create_desktop_api() -> Arc<dyn DesktopApi> {
  #[cfg(target_os = "windows")]
  {
    return Arc::new(WindowsDesktopApi::new());
  }
  #[cfg(target_os = "linux")]
  {
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP")
      .unwrap_or_default()
      .to_lowercase();

    return match desktop_env.as_str() {
      "gnome" => Arc::new(GnomeDesktopApi::new()),
      _ => {
        unimplemented!(
          "The desktop environment {} is not currently supported, please wait for future updates.",
          desktop_env
        );
      }
    };
  }

  // Not supported
  unimplemented!(
    "The operating system {} is not currently supported, please wait for future updates.",
    std::env::consts::OS
  );
}

#[derive(Debug, Clone)]
pub struct Notification {
  pub title: String,
  pub body: Option<String>,
  pub icon: Option<Icon>,
  pub urgency: Urgency,
  pub timeout: Option<Duration>,
  pub actions: Vec<Action>,
}

impl Notification {
  pub fn builder<T: Into<String>>(title: T) -> NotificationBuilder {
    NotificationBuilder {
      title: title.into(),
      body: None,
      icon: None,
      urgency: Urgency::Normal,
      timeout: None,
      actions: Vec::new(),
    }
  }
}

pub struct NotificationBuilder {
  title: String,
  body: Option<String>,
  icon: Option<Icon>,
  urgency: Urgency,
  timeout: Option<Duration>,
  actions: Vec<Action>,
}

impl NotificationBuilder {
  pub fn body<T: Into<String>>(mut self, body: T) -> Self {
    self.body = Some(body.into());
    self
  }
  pub fn icon(mut self, icon: Icon) -> Self {
    self.icon = Some(icon);
    self
  }
  pub fn urgency(mut self, urgency: Urgency) -> Self {
    self.urgency = urgency;
    self
  }
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = Some(timeout);
    self
  }
  pub fn action<ID: Into<String>, T: Into<String>>(mut self, id: ID, title: T) -> Self {
    self.actions.push(Action {
      id: id.into(),
      title: title.into(),
    });
    self
  }
  pub fn build(self) -> Notification {
    Notification {
      title: self.title,
      body: self.body,
      icon: self.icon,
      urgency: self.urgency,
      timeout: self.timeout,
      actions: self.actions,
    }
  }
}

#[derive(Debug, Clone)]
pub enum Icon {
  Path(PathBuf),
  Resource(String),
  Raw(Vec<u8>),
}

#[derive(Debug, Clone, Copy)]
pub enum Urgency {
  Low,
  Normal,
  Critical,
}

#[derive(Debug, Clone)]
pub struct Action {
  pub id: String,
  pub title: String,
}

#[derive(Debug)]
pub enum DesktopApiError {
  Unsupported,
  Io(std::io::Error),
  Backend(String),
  InvalidNotification(String),
}

impl fmt::Display for DesktopApiError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DesktopApiError::Unsupported => write!(f, "operation not supported on this platform"),
      DesktopApiError::Io(e) => write!(f, "io error: {e}"),
      DesktopApiError::Backend(msg) => write!(f, "backend error: {msg}"),
      DesktopApiError::InvalidNotification(msg) => write!(f, "invalid notification: {msg}"),
    }
  }
}

impl Error for DesktopApiError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      DesktopApiError::Io(e) => Some(e),
      _ => None,
    }
  }
}

impl From<std::io::Error> for DesktopApiError {
  fn from(e: std::io::Error) -> Self {
    DesktopApiError::Io(e)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct DesktopCapabilities {
  pub notifications: bool,
  pub actions: bool,
  pub set_wallpaper: bool,
  pub raw_icon_bytes: bool,
  pub open_file: bool,
}

pub trait DesktopApi: Send + Sync {
  fn change_background(&self, image: &Path) -> Result<(), DesktopApiError>;

  fn capabilities(&self) -> DesktopCapabilities;

  fn send_notification(&self, notification: &Notification) -> Result<(), DesktopApiError>;

  fn open_file(&self, file: &Path) -> Result<(), DesktopApiError>;
}
