/*
 * Copyright © 2024 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// ----------------------------------------------------------------

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use dirs::home_dir;

use crate::constants::{JVR_CONFIGURE_FILE_NAME, JVR_HOME_DIR_NAME};

// ----------------------------------------------------------------

/// Returns the path to the jvr configuration JSON file
/// 
/// The configuration file is located at `~/.jvr/config.json` (Unix) or `%USERPROFILE%\.jvr\config.json` (Windows)
pub fn jvr_config_json_path() -> PathBuf {
    let mut config_path = home_dir().expect("Failed to get user home directory");
    config_path.push(JVR_HOME_DIR_NAME);

    fs::create_dir_all(&config_path).expect("Failed to create jvr home directory");
    config_path.push(JVR_CONFIGURE_FILE_NAME);

    config_path
}

/// Returns the path to the jvr home directory
/// 
/// The home directory is located at `~/.jvr` (Unix) or `%USERPROFILE%\.jvr` (Windows)
pub fn jvr_home_dir() -> PathBuf {
    let mut user_home = home_dir().expect("Failed to get user home directory");
    user_home.push(JVR_HOME_DIR_NAME);

    user_home
}

/// Creates a symlink from target to link path
/// 
/// On Windows, creates a directory junction or symlink depending on the target type.
/// On Unix, creates a symbolic link.
/// 
/// # Arguments
/// * `target` - The target path that the symlink will point to
/// * `link` - The path where the symlink will be created
pub fn create_symlink(target: &std::path::Path, link: &std::path::Path) -> std::io::Result<()> {
    // Remove existing link or directory if it exists
    if link.exists() {
        #[cfg(target_os = "windows")]
        {
            // On Windows, check if it's a directory or file
            if link.is_dir() {
                std::fs::remove_dir_all(link)?;
            } else {
                std::fs::remove_file(link)?;
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, check if it's a symlink or regular file/directory
            if link.is_symlink() {
                std::fs::remove_file(link)?;
            } else if link.is_dir() {
                std::fs::remove_dir_all(link)?;
            } else {
                std::fs::remove_file(link)?;
            }
        }
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create the symlink
    #[cfg(target_os = "windows")]
    {
        /*
        use std::os::windows::fs;
        // JAVA_HOME is always a directory, so we use symlink_dir
        if target.is_dir() {
            fs::symlink_dir(target, link)?;
        } else {
            fs::symlink_file(target, link)?;
        }
        */

        // @since 0.3.2
        // Use junction — no admin needed!
        junction::create(target, link)?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs;
        fs::symlink(target, link)?;
    }

    Ok(())
}

// ----------------------------------------------------------------

/// Opens the jvr home directory in the system's default file manager
/// 
/// On Windows, uses `explorer`. On macOS, uses `open`. On Linux, uses `xdg-open`.
pub fn open_jvr_directory() {
    let jvr_dir = jvr_home_dir();
    if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(jvr_dir)
            .spawn()
            .expect("Failed to open directory");
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(jvr_dir)
            .spawn()
            .expect("Failed to open directory");
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open")
            .arg(jvr_dir)
            .spawn()
            .expect("Failed to open directory");
    } else {
        println!("Unsupported OS");
    }
}
