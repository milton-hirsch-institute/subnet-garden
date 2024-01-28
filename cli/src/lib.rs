// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use exitcode::ExitCode;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;
use std::process::exit;
use subnet_garden_core::pool;

pub const DEFAULT_STORAGE_PATH: &str = "subnet-garden-pool.yaml";

pub const SUBG_COMMAND: &str = "subg";

fn show_error(err: impl Error, message: &str, exit_code: ExitCode) -> ! {
    eprintln!("{}", message);
    eprintln!("{}", err);
    exit(exit_code);
}

pub fn result<T, E>(result: Result<T, E>, exit_code: ExitCode, message: &str) -> T
where
    E: Error,
{
    match result {
        Ok(value) => value,
        Err(err) => {
            show_error(err, message, exit_code);
        }
    }
}

#[derive(Debug)]
enum PoolFormat {
    Json,
    Yaml,
}

impl Display for PoolFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s.to_lowercase())
    }
}

fn parse_pool_path(pool_path: &str) -> (&Path, crate::PoolFormat) {
    let path = Path::new(pool_path);
    let format = match path.extension() {
        Some(ext) => match ext.to_str().unwrap() {
            "json" => PoolFormat::Json,
            "yaml" | "yml" => PoolFormat::Yaml,
            _ => {
                eprintln!("Unknown pool file extension: {}", ext.to_str().unwrap());
                exit(exitcode::USAGE);
            }
        },
        None => {
            eprintln!("Pool file has no extension: {}", path.display());
            exit(exitcode::USAGE);
        }
    };
    (path, format)
}

pub fn load_pool(pool_path: &str) -> pool::SubnetPool {
    let (path, pool_format) = parse_pool_path(pool_path);
    if !path.exists() {
        eprintln!("Subnet pool file does not exist at {}", path.display());
        exit(exitcode::NOINPUT);
    }
    if !path.is_file() {
        eprintln!("Path is not a file at {}", path.display());
        exit(exitcode::NOINPUT);
    }
    let pool_file = File::open(path).unwrap();
    fn from_reader<'a, E: Error>(
        reader: &'a File,
        from_reader: fn(&'a File) -> Result<pool::SubnetPool, E>,
    ) -> pool::SubnetPool {
        result(
            from_reader(reader),
            exitcode::DATAERR,
            "Unable to load subnet pool file",
        )
    }

    match pool_format {
        PoolFormat::Json => from_reader(&pool_file, serde_json::from_reader),
        PoolFormat::Yaml => from_reader(&pool_file, serde_yaml::from_reader),
    }
}

pub fn store_pool(pool_path: &str, pool: &pool::SubnetPool) {
    let (path, pool_format) = parse_pool_path(pool_path);

    let pool_file = result(
        File::create(path),
        exitcode::CANTCREAT,
        &format!("Could not create pool file at {}", path.display()),
    );
    fn to_writer<'a, E: Error>(
        writer: &'a File,
        to_writer: fn(&'a File, &pool::SubnetPool) -> Result<(), E>,
        subnet_pool: &pool::SubnetPool,
    ) {
        result(
            to_writer(writer, subnet_pool),
            exitcode::CANTCREAT,
            "Could not store pool file",
        );
    }

    match pool_format {
        PoolFormat::Json => to_writer(&pool_file, serde_json::to_writer_pretty, pool),
        PoolFormat::Yaml => to_writer(&pool_file, serde_yaml::to_writer, pool),
    }
}
