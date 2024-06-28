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

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "jvr",
    version = "0.1.0",
    about = "A simple Java version manager(registry: jvr)",
    long_about = "A simple and easy-to-use Java version manager(registry: jvr), similar to Node.js's nvm,\n\
    but it does not follow nvm's naming convention. Otherwise, it would be\n\
    named 'jvm', which could cause command conflicts or ambiguity."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Add a JDK version, e.g.: >_ $ jvr add jdk8 \"${YOUR_HOME}/jdkws/jdk8\"")]
    Add {
        #[arg(help = "The alias for the JDK version")]
        name: String,
        #[arg(help = "The path to the JDK home directory")]
        path: String,
    },
    #[command(about = "List all JDK versions")]
    List,
    #[command(about = "Use a specific JDK version")]
    Use { name: String },
    #[command(about = "Show jvr version")]
    Version,
    #[command(about = "Open .jvr home directory")]
    Open,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Java {
    pub name: String,
    pub home: String,
}

impl Java {
    pub fn new(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            home: path.to_string(),
        }
    }
}

// ----------------------------------------------------------------
