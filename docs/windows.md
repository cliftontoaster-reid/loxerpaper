# Windows Implementation

This document describes the Windows-specific implementation of the `DesktopApi` trait in loxerpaper.

## Overview

The `WindowsDesktopApi` provides Windows-native desktop integration using Windows APIs and WinRT for modern notification support.

## Features

### ✅ Supported Features

- **Desktop Wallpaper Management**: Uses Windows' `SystemParametersInfoW` API to set both light and dark mode wallpapers
- **Toast Notifications**: Rich notifications with actions using WinRT Toast Notifications
- **File Operations**: Open files with default applications using `ShellExecuteW`
- **Cross-platform Compatibility**: Automatically detected and used on Windows systems

### ❌ Limitations

- **Raw Icon Bytes**: Not supported by the winrt-notification library
- **Windows 10+ Required**: Toast notifications require Windows 10 or newer

## Technical Implementation

### Wallpaper Management

```rust
// Sets wallpaper using SystemParametersInfoW
let uri = format!("file://{}", image.display());
unsafe {
    SystemParametersInfoW(
        SPI_SETDESKWALLPAPER,
        0,
        Some(pcwstr.as_ptr() as *const std::ffi::c_void),
        SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
    )
}
```

### Notifications

```rust
// Creates toast notifications with WinRT
let mut toast = Toast::new(Toast::POWERSHELL_APP_ID);
toast.title(&notification.title);
toast.text1(body);
toast.duration(Duration::Short);
toast.action(&action.title, &action.id);
toast.show()?;
```

### File Operations

```rust
// Opens files with default application
unsafe {
    ShellExecuteW(
        HWND(0),
        PCWSTR(wide_open.as_ptr()),
        PCWSTR(wide_file.as_ptr()),
        PCWSTR::null(),
        PCWSTR::null(),
        SW_SHOWNORMAL,
    )
}
```

## Dependencies

The Windows implementation requires these Rust crates:

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_System_SystemServices", 
  "Win32_UI_Shell",
  "Win32_UI_WindowsAndMessaging",
  "Win32_Graphics_Gdi",
  "Data_Xml_Dom",
  "Foundation",
  "UI_Notifications",
  "ApplicationModel_Core",
  "implement"
] }
winrt-notification = "0.5.1"
```

## Usage Examples

### Basic Wallpaper Change

```rust
use loxerpaper::api::{WindowsDesktopApi, DesktopApi};
use std::path::Path;

let desktop = WindowsDesktopApi::new();
let wallpaper = Path::new("C:\\Users\\username\\Pictures\\wallpaper.jpg");
desktop.change_background(wallpaper)?;
```

### Rich Notifications

```rust
use loxerpaper::api::{Notification, Urgency};
use std::time::Duration;

let notification = Notification::builder("New Wallpaper")
    .body("Your desktop background has been updated!")
    .urgency(Urgency::Normal)
    .timeout(Duration::from_secs(5))
    .action("view", "👁️ View")
    .action("undo", "↩️ Undo")
    .build();

desktop.send_notification(&notification)?;
```

### File Operations

```rust
// Open image with default viewer
let image_path = Path::new("C:\\Users\\username\\Pictures\\photo.jpg");
desktop.open_file(image_path)?;
```

## Platform Detection

The application automatically detects Windows and uses the appropriate implementation:

```rust
let desktop: Arc<dyn DesktopApi> = {
    #[cfg(target_os = "windows")]
    {
        println!("Detected Windows, using Windows API.");
        Arc::new(WindowsDesktopApi::new())
    }
    #[cfg(target_os = "linux")]
    {
        println!("Detected GNOME desktop environment, using GNOME API.");
        Arc::new(GnomeDesktopApi::new())
    }
};
```

## Error Handling

The Windows implementation provides detailed error messages for common failure scenarios:

- **File Not Found**: When wallpaper or file paths don't exist
- **API Failures**: When Windows APIs return error codes
- **Notification Errors**: When toast notification creation fails
- **Permission Issues**: When access is denied to system functions

## Building for Windows

To build the project for Windows from a Linux system:

```bash
# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --target x86_64-pc-windows-gnu

# Run Windows example (on Windows only)
cargo run --example windows_desktop_api
```

## Testing

Run the Windows example to test all functionality:

```bash
# Windows only
cargo run --example windows_desktop_api
```

This will:

1. Display supported capabilities
2. Send test notifications
3. Attempt to set wallpaper (if image exists)
4. Try to open a file

## Compatibility

- **Windows 10**: Full support including toast notifications
- **Windows 11**: Full support with enhanced notifications
- **Windows 7/8**: Wallpaper and file operations only (no toast notifications)

## Future Enhancements

Potential improvements for the Windows implementation:

- [ ] Support for raw icon bytes via temporary files
- [ ] Windows 7/8 notification fallback using balloon tips
- [ ] Custom notification sounds
- [ ] Notification history integration
- [ ] Multi-monitor wallpaper support
- [ ] Windows accent color integration
