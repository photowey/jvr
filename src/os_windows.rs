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

use crate::constants::{
    JAVA_HOME_BIN_WINDOWS, JVR_OS_WINDOWS_ENVIRONMENT_SYSTEM_SUB_KEY,
    JVR_OS_WINDOWS_ENVIRONMENT_USER_SUB_KEY, PATH_ENVIRONMENT_VARIABLE_WINDOWS,
};
use std::io;
use windows::core::PCSTR;
use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};
use windows_sys::Win32::System::Registry;
use winreg::enums::{RegType, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::types::FromRegValue;
use winreg::{RegKey, RegValue};

use crate::env::EnvironmentAccessor;

// ----------------------------------------------------------------

/// Windows environment variable accessor
///
/// Manages user and system environment variables through the Windows registry
pub struct WindowsEnvironmentAccessor {}

impl WindowsEnvironmentAccessor {
    /// Creates a new Windows environment variable accessor instance
    pub fn new() -> Self {
        Self {}
    }

    /// Opens an environment variable registry subkey (read-only)
    fn open_env_subkey(
        &self,
        h_key: winreg::HKEY,
        sub_key: &str,
        access: Registry::REG_SAM_FLAGS,
    ) -> io::Result<RegKey> {
        let root = RegKey::predef(h_key);
        root.open_subkey_with_flags(sub_key, access).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open registry subkey '{}': {}", sub_key, e),
            )
        })
    }

    /// Creates or opens an environment variable registry subkey (writable)
    fn create_or_open_env_subkey(&self, h_key: winreg::HKEY, sub_key: &str) -> io::Result<RegKey> {
        let root = RegKey::predef(h_key);
        let (key, _) = root
            .create_subkey_with_flags(sub_key, KEY_WRITE)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to create/open registry subkey '{}': {}", sub_key, e),
                )
            })?;

        Ok(key)
    }

    /// Reads an environment variable value from the registry
    fn get_environment_variable(
        &self,
        h_key: winreg::HKEY,
        sub_key: &str,
        name: &str,
    ) -> io::Result<String> {
        let env_key = self.open_env_subkey(h_key, sub_key, KEY_READ)?;
        env_key.get_value(name).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Environment variable '{}' not found in '{}'", name, sub_key),
                )
            } else {
                io::Error::new(io::ErrorKind::Other, e.to_string())
            }
        })
    }

    /// Sets an environment variable value in the registry
    fn set_environment_variable(
        &self,
        h_key: winreg::HKEY,
        sub_key: &str,
        name: &str,
        value: &str,
    ) -> io::Result<()> {
        let env_key = self.create_or_open_env_subkey(h_key, sub_key)?;
        env_key
            .set_value(name, &value)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        self.broadcast_environment_change()
    }

    // ----------------------------------------------------------------

    /// Ensures the PATH environment variable contains the specified entry
    ///
    /// # Arguments
    /// * `entry` - The entry to add to PATH
    /// * `user` - `true` for user-level, `false` for system-level
    fn ensure_path_contains(&self, entry: &str, user: bool) -> io::Result<()> {
        let (h_key, sub_key) = if user {
            (HKEY_CURRENT_USER, JVR_OS_WINDOWS_ENVIRONMENT_USER_SUB_KEY)
        } else {
            (
                HKEY_LOCAL_MACHINE,
                JVR_OS_WINDOWS_ENVIRONMENT_SYSTEM_SUB_KEY,
            )
        };

        let root = RegKey::predef(h_key);
        let env_key = root.open_subkey_with_flags(sub_key, KEY_READ | KEY_WRITE)?;

        let path_value: String = match env_key.get_raw_value(PATH_ENVIRONMENT_VARIABLE_WINDOWS) {
            Ok(raw) if raw.vtype == RegType::REG_EXPAND_SZ || raw.vtype == RegType::REG_SZ => {
                String::from_reg_value(&raw)?
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "PATH is not a string type",
                ))
            }
        };

        let paths: Vec<&str> = path_value.split(';').collect();
        let found = paths.iter().any(|p| p.trim().eq_ignore_ascii_case(entry));

        if !found {
            let new_path = if path_value.is_empty() {
                entry.to_string()
            } else if path_value.ends_with(';') {
                format!("{}{}", path_value, entry)
            } else {
                format!("{};{}", path_value, entry)
            };

            let reg_value = self.string_to_reg_expand_sz(&new_path);
            env_key.set_raw_value(PATH_ENVIRONMENT_VARIABLE_WINDOWS, &reg_value)?;
            self.broadcast_environment_change()?;
        }

        Ok(())
    }

    /// Converts a string to a Windows registry REG_EXPAND_SZ type value
    ///
    /// REG_EXPAND_SZ type supports environment variable expansion (e.g., %JAVA_HOME%)
    ///
    /// # Safety
    /// This method uses `unsafe` code to convert a UTF-16 byte array to a u8 byte array.
    /// This is safe because:
    /// 1. `wide` is a valid Vec<u16> with contiguous memory layout
    /// 2. We calculate the correct byte length (number of elements × u16 size)
    /// 3. The converted byte array is immediately copied to a new Vec<u8>, preventing dangling pointers
    fn string_to_reg_expand_sz(&self, v: &str) -> RegValue {
        // Encode UTF-8 string to UTF-16 (format used by Windows registry)
        let mut wide: Vec<u16> = v.encode_utf16().collect();
        wide.push(0); // Add null terminator

        // Convert u16 array to byte array
        // Note: unsafe is necessary here because we need to convert u16 memory representation to bytes
        let bytes: Vec<u8> = unsafe {
            std::slice::from_raw_parts(
                wide.as_ptr() as *const u8,
                wide.len() * std::mem::size_of::<u16>(),
            )
            .to_vec()
        };

        RegValue {
            vtype: RegType::REG_EXPAND_SZ,
            bytes,
        }
    }
}

impl Default for WindowsEnvironmentAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentAccessor for WindowsEnvironmentAccessor {
    fn set_user_environment_variable(&self, name: &str, value: &str) -> io::Result<()> {
        self.set_environment_variable(
            HKEY_CURRENT_USER,
            JVR_OS_WINDOWS_ENVIRONMENT_USER_SUB_KEY,
            name,
            value,
        )
    }

    fn get_user_environment_variable(&self, name: &str) -> io::Result<String> {
        self.get_environment_variable(
            HKEY_CURRENT_USER,
            JVR_OS_WINDOWS_ENVIRONMENT_USER_SUB_KEY,
            name,
        )
    }

    fn set_system_environment_variable(&self, name: &str, value: &str) -> io::Result<()> {
        self.set_environment_variable(
            HKEY_LOCAL_MACHINE,
            JVR_OS_WINDOWS_ENVIRONMENT_SYSTEM_SUB_KEY,
            name,
            value,
        )
    }

    fn get_system_environment_variable(&self, name: &str) -> io::Result<String> {
        self.get_environment_variable(
            HKEY_LOCAL_MACHINE,
            JVR_OS_WINDOWS_ENVIRONMENT_SYSTEM_SUB_KEY,
            name,
        )
    }

    fn broadcast_environment_change(&self) -> io::Result<()> {
        // Broadcast environment variable change message to notify all windows that environment variables have changed
        // This allows running applications to detect environment variable changes
        const ENVIRONMENT_CSTR: PCSTR = PCSTR(b"Environment\0".as_ptr());
        unsafe {
            // Send message to all top-level windows to notify environment variable changes
            // Ignore return value as some windows may not respond, which is normal
            let _ = SendMessageTimeoutA(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(ENVIRONMENT_CSTR.0 as isize),
                SMTO_ABORTIFHUNG,
                100, // 100ms timeout
                None,
            );
        }

        Ok(())
    }

    fn ensure_java_home_bin_in_user_path(&self) -> io::Result<()> {
        self.ensure_path_contains(JAVA_HOME_BIN_WINDOWS, true)
    }

    fn ensure_java_home_bin_in_system_path(&self) -> io::Result<()> {
        self.ensure_path_contains(JAVA_HOME_BIN_WINDOWS, false)
    }
}
