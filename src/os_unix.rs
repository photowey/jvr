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

use crate::constants::{JAVA_HOME_BIN, PATH_ENVIRONMENT_VARIABLE_UNIX};
use crate::env::EnvironmentAccessor;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

// ----------------------------------------------------------------

/// Unix environment variable accessor
///
/// Manages user and system environment variables through shell configuration files
pub struct UnixEnvironmentAccessor {}

impl UnixEnvironmentAccessor {
    /// Creates a new Unix environment variable accessor instance
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the user's home directory
    fn get_home_dir(&self) -> io::Result<PathBuf> {
        dirs::home_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Cannot determine home directory")
        })
    }

    /// Detects the user's default shell
    fn detect_shell(&self) -> String {
        std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/bash".to_string())
            .to_lowercase()
    }

    /// Gets the list of user shell configuration files based on the detected shell
    fn get_user_shell_config_files(&self) -> Vec<PathBuf> {
        let home = match self.get_home_dir() {
            Ok(h) => h,
            Err(_) => return vec![],
        };

        let shell = self.detect_shell();
        let mut config_files = Vec::new();

        if shell.contains("zsh") {
            config_files.push(home.join(".zshrc"));
            config_files.push(home.join(".zshenv"));
        } else if shell.contains("bash") {
            config_files.push(home.join(".bashrc"));
            config_files.push(home.join(".bash_profile"));
            config_files.push(home.join(".profile"));
        } else {
            config_files.push(home.join(".profile"));
            config_files.push(home.join(".bashrc"));
        }

        config_files
    }

    /// Gets the list of system-wide configuration files
    fn get_system_config_files(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from("/etc/profile"),
            PathBuf::from("/etc/environment"),
            PathBuf::from("/etc/bash.bashrc"),
        ]
    }

    /// Reads an environment variable value from a shell configuration file
    fn read_env_from_file(&self, file_path: &Path, name: &str) -> io::Result<Option<String>> {
        if !file_path.exists() {
            return Ok(None);
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.starts_with("export") {
                if let Some(rest) = line.strip_prefix("export") {
                    if let Some(value) = self.parse_env_line(rest.trim(), name) {
                        return Ok(Some(value));
                    }
                }
            } else if let Some(value) = self.parse_env_line(line, name) {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    /// Parses an environment variable line from a shell configuration file
    ///
    /// Supports formats: `VAR=value`, `VAR="value"`, `VAR='value'`, and `export VAR=value`
    fn parse_env_line(&self, line: &str, name: &str) -> Option<String> {
        if line.starts_with('#') {
            return None;
        }

        if let Some(equals_pos) = line.find('=') {
            let var_name = line[..equals_pos].trim();
            if var_name == name {
                let value = line[equals_pos + 1..].trim();
                let value = value
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                    .unwrap_or(value);
                return Some(value.to_string());
            }
        }

        None
    }

    /// Writes an environment variable to a shell configuration file
    ///
    /// If the variable already exists, it will be replaced. Otherwise, it will be appended.
    fn write_env_to_file(&self, file_path: &Path, name: &str, value: &str) -> io::Result<()> {
        let mut content = String::new();

        if file_path.exists() {
            let mut file = File::open(file_path)?;
            file.read_to_string(&mut content)?;
        }

        let export_line = format!("export {}={}", name, self.quote_value(value));
        let pattern = format!("export {}", name);

        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();
        let mut found = false;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with(&pattern) && trimmed.contains('=') {
                new_lines.push(export_line.clone());
                found = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !found {
            new_lines.push(export_line);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)?;

        for line in new_lines {
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    /// Quotes a value if it contains spaces, dollar signs, or backticks
    fn quote_value(&self, value: &str) -> String {
        if value.contains(' ') || value.contains('$') || value.contains('`') {
            format!("\"{}\"", value.replace('"', "\\\""))
        } else {
            value.to_string()
        }
    }

    /// Searches for an environment variable in a list of configuration files
    ///
    /// Returns the first matching value found, or an error if not found in any file
    fn find_env_in_files(&self, files: &[PathBuf], name: &str) -> io::Result<String> {
        for file in files {
            if let Ok(Some(value)) = self.read_env_from_file(file, name) {
                return Ok(value);
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Environment variable '{}' not found", name),
        ))
    }

    /// Writes an environment variable to the first available configuration file
    ///
    /// Tries to write to existing files first, or creates the first file if none exist
    fn write_env_to_first_available(
        &self,
        files: &[PathBuf],
        name: &str,
        value: &str,
    ) -> io::Result<()> {
        for file in files {
            if file.exists() || file.parent().map(|p| p.exists()).unwrap_or(false) {
                return self.write_env_to_file(file, name, value);
            }
        }

        if let Some(first) = files.first() {
            if let Some(parent) = first.parent() {
                std::fs::create_dir_all(parent)?;
            }
            self.write_env_to_file(first, name, value)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No configuration file available",
            ))
        }
    }

    /// Ensures the PATH environment variable contains the specified entry
    ///
    /// # Arguments
    /// * `entry` - The entry to add to PATH
    /// * `user` - `true` for user-level, `false` for system-level
    fn ensure_path_contains(&self, entry: &str, user: bool) -> io::Result<()> {
        let files = if user {
            self.get_user_shell_config_files()
        } else {
            self.get_system_config_files()
        };

        let current_path = self
            .find_env_in_files(&files, PATH_ENVIRONMENT_VARIABLE_UNIX)
            .unwrap_or_else(|_| std::env::var(PATH_ENVIRONMENT_VARIABLE_UNIX).unwrap_or_default());

        let paths: Vec<&str> = current_path.split(':').collect();
        let found = paths.iter().any(|p| p.trim() == entry.trim());

        if !found {
            let new_path = if current_path.is_empty() {
                entry.to_string()
            } else {
                format!("{}:{}", current_path, entry)
            };

            self.write_env_to_first_available(&files, PATH_ENVIRONMENT_VARIABLE_UNIX, &new_path)?;
        }

        Ok(())
    }
}

impl Default for UnixEnvironmentAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentAccessor for UnixEnvironmentAccessor {
    fn set_user_environment_variable(&self, name: &str, value: &str) -> io::Result<()> {
        let files = self.get_user_shell_config_files();
        self.write_env_to_first_available(&files, name, value)
    }

    fn get_user_environment_variable(&self, name: &str) -> io::Result<String> {
        let files = self.get_user_shell_config_files();
        self.find_env_in_files(&files, name)
    }

    fn set_system_environment_variable(&self, name: &str, value: &str) -> io::Result<()> {
        let files = self.get_system_config_files();
        self.write_env_to_first_available(&files, name, value)
    }

    fn get_system_environment_variable(&self, name: &str) -> io::Result<String> {
        let files = self.get_system_config_files();
        self.find_env_in_files(&files, name)
    }

    fn broadcast_environment_change(&self) -> io::Result<()> {
        Ok(())
    }

    fn ensure_java_home_bin_in_user_path(&self) -> io::Result<()> {
        // true = user scope
        self.ensure_path_contains(JAVA_HOME_BIN, true)
    }

    fn ensure_java_home_bin_in_system_path(&self) -> io::Result<()> {
        // false = system scope
        self.ensure_path_contains(JAVA_HOME_BIN, false)
    }
}
