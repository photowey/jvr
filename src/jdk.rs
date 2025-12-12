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

use std::path::Path;

use prettytable::{Cell, Row, Table};

use crate::cli::Java;
use crate::config::Configuration;
use crate::constants::JAVA_HOME;
use crate::env::create_os_environment_handler;
use crate::home::create_symlink;

// ----------------------------------------------------------------

/// Initializes jvr with a JAVA_HOME path
/// 
/// Creates the JAVA_HOME directory if it doesn't exist, sets the JAVA_HOME environment variable,
/// and ensures JAVA_HOME/bin is in the PATH environment variable.
/// 
/// # Arguments
/// * `config` - The configuration to update
/// * `java_home_path` - The path for the JAVA_HOME directory
pub fn init_jdk(config: &mut Configuration, java_home_path: &str) {
    let java_home = Path::new(java_home_path);

    // Create the directory if it doesn't exist
    if !java_home.exists() {
        std::fs::create_dir_all(java_home).unwrap_or_else(|e| {
            eprintln!("❌ Failed to create JAVA_HOME directory: {}", java_home_path);
            eprintln!("   Error: {}", e);
            std::process::exit(1);
        });
        println!("✅ Created JAVA_HOME directory: {}", java_home_path);
    } else {
        println!("✅ JAVA_HOME directory already exists: {}", java_home_path);
    }

    // Set JAVA_HOME environment variable
    let accessor = create_os_environment_handler();
    accessor
        .set_user_environment_variable(JAVA_HOME, java_home_path)
        .expect(
            "❌ Could not set `JAVA_HOME` in the user environment variables.\
             Please ensure you have write permissions, or set it manually.",
        );

    // Ensure JAVA_HOME/bin is in PATH
    accessor.ensure_java_home_bin_in_user_path().expect(
        "❌ Could not update the PATH environment variable automatically. \
        To fix this, please add `$JAVA_HOME/bin` (Unix) or `%JAVA_HOME%\\bin` (Windows) to your system's PATH manually.",
    );

    // Save the JAVA_HOME path to configuration
    config.java_home_path = Some(java_home_path.to_string());
    config.store();

    println!("✅ jvr initialized successfully!");
    println!("   JAVA_HOME: {}", java_home_path);
    println!("   Use 'jvr use <alias>' to switch between JDK versions.");
}

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
        // Verify the JDK path exists
        if !Path::new(path).exists() {
            eprintln!("❌ JDK path does not exist: {}", path);
            return;
        }

        config.versions.push(Java::new(name, path));
        println!("✅ Added JDK version: ['{}']", name);
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
/// Creates a symlink from the JAVA_HOME path (set by init) to the specified JDK path.
/// This allows switching JDK versions without updating the JAVA_HOME environment variable.
/// 
/// # Arguments
/// * `config` - The configuration to update
/// * `name` - The alias name of the JDK version to use
pub fn use_jdk(config: &mut Configuration, name: &str) {
    // Check if jvr has been initialized
    let java_home_path = match &config.java_home_path {
        Some(path) => path,
        None => {
            eprintln!("❌ jvr has not been initialized. Please run 'jvr init <JAVA_HOME_PATH>' first.");
            return;
        }
    };

    if let Some(version) = config.versions.iter().find(|v| v.name == name) {
        let jdk_path = Path::new(&version.home);

        // Verify the JDK path exists
        if !jdk_path.exists() {
            eprintln!("❌ JDK path does not exist: {}", version.home);
            return;
        }

        // Create symlink from JAVA_HOME to JDK path
        let java_home = Path::new(java_home_path);
        match create_symlink(jdk_path, java_home) {
            Ok(_) => {
                config.current = Some(name.to_string());
                config.store();
                println!("✅ Now using JDK {} at {}", name, version.home);
                println!("   JAVA_HOME symlink: {} -> {}", java_home_path, version.home);
            }
            Err(e) => {
                eprintln!("❌ Failed to create symlink: {}", e);
                eprintln!("   Please ensure you have write permissions to: {}", java_home_path);
            }
        }
    } else {
        eprintln!("❌ JDK version not found: {}", name);
    }

    list_jdks(config);
}
