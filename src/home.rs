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

pub fn jvr_config_json_path() -> PathBuf {
    let mut config_path = home_dir().expect("Failed to get usr home directory");
    config_path.push(JVR_HOME_DIR_NAME);

    fs::create_dir_all(&config_path).expect("Failed to create jvr home directory");
    config_path.push(JVR_CONFIGURE_FILE_NAME);

    config_path
}

pub fn jvr_home_dir() -> PathBuf {
    let mut user_home = home_dir().expect("Failed to get usr home directory");
    user_home.push(JVR_HOME_DIR_NAME);

    user_home
}

// ----------------------------------------------------------------

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
