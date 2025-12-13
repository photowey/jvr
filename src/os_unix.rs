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

use crate::constants::{JAVA_HOME_BIN, JVR_BEGIN, JVR_END, PATH_ENVIRONMENT_VARIABLE_UNIX};
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
            File::open(file_path)?.read_to_string(&mut content)?;
        }

        let quoted = self.quote_value(value);
        let export_assign = format!("export {}={}", name, quoted);
        let assign = format!("{}={}", name, quoted);

        let mut has_export = false;
        let mut has_export_assign = false;
        let mut has_assign = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('#') {
                continue;
            }

            let (exported, export_assigned) = self.export_line_contains(trimmed, name);
            if exported {
                has_export = true;
            }
            if export_assigned {
                has_export_assign = true;
            }

            if self.is_assign_line(trimmed, name) {
                has_assign = true;
            }
        }

        let mut new_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // 1: export NAME=xxx
            if has_export_assign {
                let (_, export_assigned) = self.export_line_contains(trimmed, name);
                if export_assigned {
                    new_lines.push(export_assign.clone());
                    continue;
                }
            }

            // 2: NAME=xxx + export NAME(...)
            if !has_export_assign && has_assign && has_export && self.is_assign_line(trimmed, name)
            {
                new_lines.push(assign.clone());
                continue;
            }

            new_lines.push(line.to_string());
        }

        if !has_export_assign && !(has_assign && has_export) {
            if !new_lines.is_empty() {
                new_lines.push(String::new());
            }
            new_lines.push(self.build_managed_block(&export_assign));
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

    /// Checks whether an `export ...` line contains the given variable
    /// Supports:
    ///   export NAME
    ///   export A B NAME C
    ///   export NAME=value
    ///   export A=value NAME=value
    fn export_line_contains(&self, line: &str, name: &str) -> (bool, bool) {
        let trimmed = line.trim();

        if !trimmed.starts_with("export ") {
            return (false, false);
        }

        let rest = trimmed.strip_prefix("export ").unwrap();

        let mut has_export = false;
        let mut has_export_assign = false;

        for token in rest.split_whitespace() {
            if token == name {
                has_export = true;
            } else if token.starts_with(&format!("{}=", name)) {
                has_export = true;
                has_export_assign = true;
            }
        }

        (has_export, has_export_assign)
    }

    /// Checks whether a line is a simple assignment: NAME=xxx
    fn is_assign_line(&self, line: &str, name: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with(&format!("{}=", name))
    }

    /// Builds the `jvr` managed environment block.
    ///
    /// This block is fully controlled by `jvr` and guarantees:
    /// 1. The environment variable (e.g. JAVA_HOME) is defined and exported.
    /// 2. `$<VAR>/bin` is appended to PATH *after* the variable is defined,
    ///    ensuring correct shell evaluation order.
    ///
    /// Example (when managing JAVA_HOME):
    ///
    /// ```sh
    /// # >>> jvr managed >>>
    /// export JAVA_HOME=/path/to/jdk21
    /// export PATH=$PATH:$JAVA_HOME/bin
    /// # <<< jvr managed <<<
    /// ```
    ///
    /// Notes:
    /// - The entire block is treated as an atomic unit:
    ///   if it already exists in a shell configuration file, it will be
    ///   replaced as a whole; otherwise, it will be appended.
    /// - Users may define or export JAVA_HOME elsewhere, but `jvr` only
    ///   guarantees correctness within this managed block.
    fn build_managed_block(&self, export_assign: &str) -> String {
        format!(
            "{}\n\
         {}\n\
         export PATH=$PATH:{}\n\
         {}",
            JVR_BEGIN, export_assign, JAVA_HOME_BIN, JVR_END
        )
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
