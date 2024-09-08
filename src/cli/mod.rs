// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::path::{Path, PathBuf};

use clap::Parser;

use crate::app::config::get_data_dir;

const LONG_ABOUT: &str = "
dft - DataFusion TUI

CLI and terminal UI data analysis tool using Apache DataFusion as query
execution engine.

dft provides a rich terminal UI as well as a broad array of pre-integrated
data sources, formats to make querying and analyzing data from
various sources.

Environment Variables
RUST_LOG { trace | debug | info | error }: Standard rust logging level.  Default is info.
";

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = LONG_ABOUT)]
pub struct DftCli {
    #[clap(
        short,
        long,
        num_args = 0..,
        help = "Execute commands from file(s), then exit",
        value_parser(parse_valid_file)
    )]
    pub file: Vec<PathBuf>,

    #[clap(short, long, help = "Path to the configuration file")]
    pub config: Option<String>,
}

fn get_config_path(cli_config_arg: Option<&String>) -> PathBuf {
    if let Some(config) = cli_config_arg {
        Path::new(config).to_path_buf()
    } else {
        let mut config = get_data_dir();
        config.push("config.toml");
        config
    }
}

impl DftCli {
    pub fn get_config(&self) -> PathBuf {
        get_config_path(self.config.as_ref())
    }
}

fn parse_valid_file(file: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(file);
    if !path.exists() {
        Err(format!("File does not exist: '{file}'"))
    } else if !path.is_file() {
        Err(format!("Exists but is not a file: '{file}'"))
    } else {
        Ok(path)
    }
}
