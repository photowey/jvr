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

use std::io;

#[cfg(target_os = "windows")]
use crate::os_windows;
#[cfg(not(target_os = "windows"))]
use crate::os_unix;

// ----------------------------------------------------------------

/// Trait for accessing and modifying environment variables on different operating systems
pub trait EnvironmentAccessor {
    fn set_user_environment_variable(&self, name: &str, value: &str) -> io::Result<()>;

    fn get_user_environment_variable(&self, name: &str) -> io::Result<String>;

    fn set_system_environment_variable(&self, name: &str, value: &str) -> io::Result<()>;

    fn get_system_environment_variable(&self, name: &str) -> io::Result<String>;

    fn broadcast_environment_change(&self) -> io::Result<()> {
        Ok(())
    }
    fn ensure_java_home_bin_in_user_path(&self) -> io::Result<()> {
        Ok(())
    }

    fn ensure_java_home_bin_in_system_path(&self) -> io::Result<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------

/// Creates an OS-specific environment variable accessor
/// 
/// Returns `WindowsEnvironmentAccessor` on Windows, `UnixEnvironmentAccessor` on Unix systems (Linux/macOS)
pub fn create_os_environment_handler() -> Box<dyn EnvironmentAccessor> {
    #[cfg(target_os = "windows")]
    {
        Box::new(os_windows::WindowsEnvironmentAccessor::default())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Box::new(os_unix::UnixEnvironmentAccessor::default())
    }
}
