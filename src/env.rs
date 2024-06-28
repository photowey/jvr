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

use crate::os_windows;

// ----------------------------------------------------------------

pub trait EnvironmentAccessor {
    fn set_user_environment_variable(&self, name: &str, value: &str) -> io::Result<()>;

    fn get_user_environment_variable(&self, name: &str) -> io::Result<String>;

    fn set_system_environment_variable(&self, name: &str, value: &str) -> io::Result<()>;

    fn get_system_environment_variable(&self, name: &str) -> io::Result<String>;
}

// ----------------------------------------------------------------

pub fn create_os_environment_handler() -> Box<dyn EnvironmentAccessor> {
    let os = std::env::consts::OS;

    match os {
        "windows" => Box::new(os_windows::WindowsEnvironmentAccessor::default()),
        _ => panic!("Unsupported OS: [{}] now", os),
    }
}
