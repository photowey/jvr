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

use serde::{Deserialize, Serialize};

use crate::cli::Java;
use crate::home::jvr_config_json_path;

// ----------------------------------------------------------------

/// Configuration structure for jvr
/// 
/// Stores the list of registered JDK versions and the currently active version
#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    /// The alias name of the currently active JDK version
    pub current: Option<String>,
    /// List of all registered JDK versions
    pub versions: Vec<Java>,
}

// ----------------------------------------------------------------

impl Configuration {
    /// Creates a new Configuration instance by loading from the config file
    /// 
    /// If the config file doesn't exist, returns a default empty configuration
    pub fn new() -> Self {
        let config_path = jvr_config_json_path();
        if config_path.exists() {
            let config_content =
                fs::read_to_string(config_path).expect("Failed to read config file");
            serde_json::from_str(&config_content).expect("Failed to parse config file")
        } else {
            Configuration {
                versions: Vec::new(),
                current: None,
            }
        }
    }

    /// Saves the configuration to the config file
    pub fn store(&self) {
        let config_path = jvr_config_json_path();
        let config_content =
            serde_json::to_string_pretty(self).expect("Failed to serialize config");
        fs::write(config_path, config_content).expect("Failed to write config file");
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------
