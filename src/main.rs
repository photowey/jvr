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

use clap::Parser;

use jvr::cli::{Cli, Commands};
use jvr::config::Configure;
use jvr::home;
use jvr::jdk::{add_jdk, list_jdks, use_jdk};

// ----------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    let mut configer = Configure::default();

    match &cli.command {
        Commands::Add { name, path } => {
            add_jdk(&mut configer, name, path);

            configer.store();
        }
        Commands::List => {
            list_jdks(&configer);
        }
        Commands::Use { name } => {
            use_jdk(&mut configer, name);
        }
        Commands::Version => {
            show_version();
        }
        Commands::Open => {
            home::open_jvr_directory();
        }
    }
}

fn show_version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("jvr version {}", version);
}

/*
cargo run -- add jdk8 "D:\software\jdkws\jdk8"
cargo run -- add jdk11 "D:\software\jdkws\jdk11"
cargo run -- add jdk17 "D:\software\jdkws\jdk17"
cargo run -- add jdk19 "D:\software\jdkws\jdk19"
cargo run -- add jdk21 "D:\software\jdkws\jdk21"

cargo run -- add graalvm17 "D:\software\jdkws\graalvm17"
cargo run -- add graalvm19 "D:\software\jdkws\graalvm19"

cargo run -- list
cargo run -- use jdk21
*/
