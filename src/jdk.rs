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

use prettytable::{Cell, Row, Table};

use crate::cli::Java;
use crate::config::Configuration;
use crate::constants::JAVA_HOME;
use crate::env::create_os_environment_handler;

// ----------------------------------------------------------------

/// Adds a new JDK version to the configuration
/// 
/// # Arguments
/// * `config` - The configuration to update
/// * `name` - The alias name for the JDK version
/// * `path` - The home directory path of the JDK installation
pub fn add_jdk(config: &mut Configuration, name: &str, path: &str) {
    if config.versions.iter().any(|v| v.name == name) {
        println!("➕ JDK {} has already been added.", name);
    } else {
        config.versions.push(Java::new(name, path));
        println!("✅ Added JDK version: ['{}']", name);

        let accessor = create_os_environment_handler();
        accessor
            .set_user_environment_variable(JAVA_HOME, &path)
            .expect(
                "❌ Could not set `JAVA_HOME` in the user environment variables.\
                 Please ensure you have write permissions, or set it manually.",
            );

        accessor.ensure_java_home_bin_in_user_path().expect(
            "❌ Could not update the PATH environment variable automatically. \
            To fix this, please add `$JAVA_HOME/bin` (Unix) or `%JAVA_HOME%\\bin` (Windows) to your system's PATH manually.",
        );
    }

    list_jdks(config);
}

/// Lists all registered JDK versions in a formatted table
/// 
/// The table shows the index, alias name, path, and current status (marked with *)
pub fn list_jdks(config: &Configuration) {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_DEFAULT);

    table.add_row(Row::new(vec![
        Cell::new("#").style_spec("c"),
        Cell::new("Alias").style_spec("c"),
        Cell::new("Path").style_spec("c"),
        Cell::new("Current").style_spec("c"),
    ]));

    for (index, version) in config.versions.iter().enumerate() {
        let is_current = match &config.current {
            Some(current) if current == &version.name => "*",
            _ => "-",
        };
        table.add_row(Row::new(vec![
            Cell::new(&(index + 1).to_string()).style_spec("c"),
            Cell::new(&version.name).style_spec("c"),
            Cell::new(&version.home).style_spec("l"),
            Cell::new(is_current).style_spec("c"),
        ]));
    }

    table.printstd();
}

/// Switches to a specific JDK version
/// 
/// Updates the JAVA_HOME environment variable and marks the version as current
/// 
/// # Arguments
/// * `config` - The configuration to update
/// * `name` - The alias name of the JDK version to use
pub fn use_jdk(config: &mut Configuration, name: &str) {
    if let Some(version) = config.versions.iter().find(|v| v.name == name) {
        config.current = Some(name.to_string());
        let path = version.home.clone();
        println!("✅ Now using JDK {} at {}", name, path.clone());

        let accessor = create_os_environment_handler();
        accessor
            .set_user_environment_variable(JAVA_HOME, &path.clone())
            .expect(
                "❌ Could not set `JAVA_HOME` in the user environment variables.\
                 Please ensure you have write permissions, or set it manually.",
            );

        config.store();
    } else {
        eprintln!("❌ JDK version not found: {}", name);
    }

    list_jdks(config);
}
