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

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

use crate::constants::{
    JVR_OS_WINDOWS_ENVIRONMENT_SYSTEM_SUB_KEY, JVR_OS_WINDOWS_ENVIRONMENT_USER_SUB_KEY,
};
use crate::env::EnvironmentAccessor;

// ----------------------------------------------------------------

pub struct WindowsEnvironmentAccessor {}

impl WindowsEnvironmentAccessor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WindowsEnvironmentAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentAccessor for WindowsEnvironmentAccessor {
    fn set_user_environment_variable(&self, name: &str, value: &str) -> std::io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (env_key, _) = hkcu.create_subkey(JVR_OS_WINDOWS_ENVIRONMENT_USER_SUB_KEY)?;

        env_key.delete_value(name)?;
        env_key.set_value(name, &value)?;

        Ok(())
    }

    fn get_user_environment_variable(&self, name: &str) -> std::io::Result<String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu.open_subkey(JVR_OS_WINDOWS_ENVIRONMENT_USER_SUB_KEY)?;

        let value: String = env_key.get_value(name)?;

        Ok(value)
    }

    fn set_system_environment_variable(&self, name: &str, value: &str) -> std::io::Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (env_key, _) = hklm.create_subkey(JVR_OS_WINDOWS_ENVIRONMENT_SYSTEM_SUB_KEY)?;

        env_key.set_value(name, &value)?;

        Ok(())
    }

    fn get_system_environment_variable(&self, name: &str) -> std::io::Result<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env_key = hklm.open_subkey(JVR_OS_WINDOWS_ENVIRONMENT_SYSTEM_SUB_KEY)?;

        let value: String = env_key.get_value(name)?;

        Ok(value)
    }
}
