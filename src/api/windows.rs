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

use crate::api::{DesktopApi, DesktopApiError, DesktopCapabilities, Icon, Notification};

#[cfg(windows)]
use {
  std::ffi::OsStr,
  std::os::windows::ffi::OsStrExt,
  windows::{
    core::PCWSTR,
    Win32::{
      Foundation::HWND,
      UI::{
        Shell::ShellExecuteW,
        WindowsAndMessaging::{
          SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
          SW_SHOWNORMAL,
        },
      },
    },
  },
  winrt_notification::{Duration, IconCrop, Sound, Toast},
};

/// Windows implementation of DesktopApi using Windows APIs for wallpaper and WinRT for notifications.
pub struct WindowsDesktopApi {}

impl WindowsDesktopApi {
  pub fn new() -> Self {
    WindowsDesktopApi {}
  }

  #[cfg(windows)]
  fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
  }

  #[cfg(windows)]
  fn path_to_wide_string(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
  }
}

impl DesktopApi for WindowsDesktopApi {
  fn change_background(&self, image: &Path) -> Result<(), DesktopApiError> {
    if !image.exists() {
      return Err(DesktopApiError::InvalidNotification(format!(
        "image path {image:?} does not exist"
      )));
    }

    #[cfg(windows)]
    {
      let wide_path = Self::path_to_wide_string(image);
      let pcwstr = PCWSTR(wide_path.as_ptr());

      unsafe {
        let result = SystemParametersInfoW(
          SPI_SETDESKWALLPAPER,
          0,
          Some(pcwstr.as_ptr() as *mut std::ffi::c_void),
          SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );

        match result {
          Ok(()) => {
            println!("Successfully changed wallpaper to {image:?}");
            Ok(())
          }
          Err(e) => Err(DesktopApiError::Backend(format!(
            "SystemParametersInfoW failed with error: {:?}",
            e
          ))),
        }
      }
    }

    #[cfg(not(windows))]
    {
      Err(DesktopApiError::Unsupported)
    }
  }

  fn capabilities(&self) -> DesktopCapabilities {
    #[cfg(windows)]
    {
      DesktopCapabilities {
        notifications: true,
        actions: false, // Limited action support in winrt-notification 0.5.1
        set_wallpaper: true,
        raw_icon_bytes: false, // WinRT notifications don't easily support raw bytes
        open_file: true,
      }
    }

    #[cfg(not(windows))]
    {
      DesktopCapabilities {
        notifications: false,
        actions: false,
        set_wallpaper: false,
        raw_icon_bytes: false,
        open_file: false,
      }
    }
  }

  fn send_notification(&self, notification: &Notification) -> Result<(), DesktopApiError> {
    #[cfg(windows)]
    {
      let mut toast = Toast::new(Toast::POWERSHELL_APP_ID);

      toast = toast.title(&notification.title);

      if let Some(body) = &notification.body {
        toast = toast.text1(body);
      }

      // Handle icon
      if let Some(icon) = &notification.icon {
        match icon {
          Icon::Path(p) => {
            if let Some(path_str) = p.to_str() {
              toast = toast.icon(p, IconCrop::Circular, path_str);
            }
          }
          Icon::Resource(name) => {
            if let Ok(path) = std::path::Path::new(name).canonicalize() {
              toast = toast.icon(&path, IconCrop::Circular, name);
            }
          }
          Icon::Raw(_) => {
            // Raw bytes not easily supported by winrt-notification
            // Could write to temp file as fallback, but skipping for now
          }
        }
      }

      // Set duration based on urgency and timeout
      toast = if let Some(timeout) = notification.timeout {
        if timeout.as_secs() > 25 {
          toast.duration(Duration::Long)
        } else {
          toast.duration(Duration::Short)
        }
      } else {
        match notification.urgency {
          crate::api::Urgency::Critical => toast.duration(Duration::Long),
          crate::api::Urgency::Normal => toast.duration(Duration::Short),
          crate::api::Urgency::Low => toast.duration(Duration::Short),
        }
      };

      // Set sound
      toast = toast.sound(Some(Sound::Default));

      toast
        .show()
        .map_err(|e| DesktopApiError::Backend(format!("winrt-notification error: {e}")))?;

      Ok(())
    }

    #[cfg(not(windows))]
    {
      Err(DesktopApiError::Unsupported)
    }
  }

  fn open_file(&self, file: &Path) -> Result<(), DesktopApiError> {
    if !file.exists() {
      return Err(DesktopApiError::InvalidNotification(format!(
        "file path {file:?} does not exist"
      )));
    }

    #[cfg(windows)]
    {
      let wide_file = Self::path_to_wide_string(file);
      let wide_open = Self::to_wide_string("open");

      unsafe {
        let result = ShellExecuteW(
          HWND(std::ptr::null_mut()),
          PCWSTR(wide_open.as_ptr()),
          PCWSTR(wide_file.as_ptr()),
          PCWSTR::null(),
          PCWSTR::null(),
          SW_SHOWNORMAL,
        );

        // ShellExecuteW returns HINSTANCE, where values > 32 indicate success
        let result_value = result.0 as isize;
        if result_value > 32 {
          println!("Successfully opened file {file:?}");
          Ok(())
        } else {
          let error_msg = match result_value {
            0 => "Out of memory or resources",
            2 => "File not found",
            3 => "Path not found",
            5 => "Access denied",
            8 => "Out of memory",
            26 => "Cannot share open file",
            27 => "File association incomplete or invalid",
            28 => "DDE timeout",
            29 => "DDE transaction failed",
            30 => "DDE busy",
            31 => "No file association",
            32 => "Invalid executable file",
            _ => "Unknown error",
          };

          Err(DesktopApiError::Backend(format!(
            "ShellExecuteW failed: {} (code: {})",
            error_msg, result_value
          )))
        }
      }
    }

    #[cfg(not(windows))]
    {
      Err(DesktopApiError::Unsupported)
    }
  }
}

impl Default for WindowsDesktopApi {
  fn default() -> Self {
    Self::new()
  }
}
