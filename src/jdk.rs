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
use crate::config::Configure;
use crate::constants::JAVA_HOME;
use crate::env::create_os_environment_handler;

// ----------------------------------------------------------------

pub fn add_jdk(configure: &mut Configure, name: &str, path: &str) {
    if configure.versions.iter().any(|v| v.name == name) {
        println!("JDK version ['{}'] already exists.", name);
    } else {
        configure.versions.push(Java::new(name, path));
        println!("Added JDK version: ['{}']", name);

        let accessor = create_os_environment_handler();
        accessor
            .set_user_environment_variable(JAVA_HOME, &path)
            .expect("Failed to set user environment variable.");
    }

    list_jdks(configure);
}

pub fn list_jdks(configure: &Configure) {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_DEFAULT);

    table.add_row(Row::new(vec![
        Cell::new("#").style_spec("c"),
        Cell::new("Alias").style_spec("c"),
        Cell::new("Path").style_spec("c"),
        Cell::new("Current").style_spec("c"),
    ]));

    for (index, version) in configure.versions.iter().enumerate() {
        let is_current = match &configure.current {
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

pub fn use_jdk(configure: &mut Configure, name: &str) {
    if let Some(version) = configure.versions.iter().find(|v| v.name == name) {
        configure.current = Some(name.to_string());
        let path = version.home.clone();
        println!("Switched to JDK version: {} ({})", name, path.clone());

        let accessor = create_os_environment_handler();
        accessor
            .set_user_environment_variable(JAVA_HOME, &path.clone())
            .expect("Failed to set user environment variable.");

        configure.store();
    } else {
        println!("JDK version not found: {}", name);
    }

    list_jdks(configure);
}
