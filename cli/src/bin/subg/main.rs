// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{Subg, SubgCommands};
use clap;
use clap::Parser;
use exitcode::ExitCode;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;
use std::process::exit;
use subcommands::init;
use subcommands::subnet;
use subnet_garden_core::pool;

mod args;
mod subcommands;
fn show_error(err: impl Error, message: &str, exit_code: ExitCode) -> ! {
    eprintln!("{}", message);
    eprintln!("{}", err);
    exit(exit_code);
}

fn result<T, E>(result: Result<T, E>, exit_code: ExitCode, message: &str) -> T
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
pub(crate) enum PoolFormat {
    Json,
    Yaml,
}

impl Display for PoolFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s.to_lowercase())
    }
}

fn parse_pool_path(pool_path: &str) -> (&Path, PoolFormat) {
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

fn load_pool<'a>(pool_path: &str) -> pool::SubnetPool {
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

fn store_pool(pool_path: &str, pool: &pool::SubnetPool) {
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

fn main() {
    let subg = Subg::parse();

    match subg.command {
        SubgCommands::Init(args) => {
            init::init(&subg.args, &args);
        }
        SubgCommands::Allocate(args) => {
            subnet::allocate(&subg.args, &args);
        }
        SubgCommands::Free(args) => {
            subnet::free(&subg.args, &args);
        }
        SubgCommands::Cidrs(_) => {
            subnet::cidrs(&subg.args);
        }
        SubgCommands::Names(_) => {
            subnet::names(&subg.args);
        }
        SubgCommands::Claim(args) => {
            subnet::claim(&subg.args, &args);
        }
        SubgCommands::Rename(args) => {
            subnet::rename(&subg.args, &args);
        }
        SubgCommands::MaxAvailable(_) => {
            subnet::max_bits(&subg.args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{DEFAULT_STORAGE_PATH, SUBG_COMMAND};
    use assert_fs::fixture::ChildPath;
    use assert_fs::fixture::PathChild;

    pub(crate) const HELP_EXIT_CODE: i32 = 2;

    pub(crate) const TEST_CIDR: &str = "10.10.0.0/16";

    pub(crate) struct Test {
        pub(crate) subg: assert_cmd::Command,
        pub(crate) _dir: assert_fs::TempDir,
        pub(crate) pool_path: ChildPath,
        pub(crate) pool: pool::SubnetPool,
    }

    impl Test {
        pub(crate) fn store(&self) {
            store_pool(self.pool_path.to_str().unwrap(), &self.pool);
        }

        pub(crate) fn load(&mut self) {
            self.pool = load_pool(self.pool_path.to_str().unwrap());
        }
    }

    pub(crate) fn new_test_with_path(path: &str) -> Test {
        let mut test = assert_cmd::Command::cargo_bin(SUBG_COMMAND).unwrap();
        let dir = assert_fs::TempDir::new().unwrap();
        let pool_path = dir.child(path);
        test.args(&["--pool-path", pool_path.to_str().unwrap()]);
        Test {
            subg: test,
            _dir: dir,
            pool_path,
            pool: pool::SubnetPool::new(TEST_CIDR.parse().unwrap()),
        }
    }

    pub(crate) fn new_test() -> Test {
        return new_test_with_path(DEFAULT_STORAGE_PATH);
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Subg::command().debug_assert();
    }
    #[test]
    fn test_bin() {
        let mut test = new_test();
        test.subg
            .assert()
            .failure()
            .code(HELP_EXIT_CODE)
            .stderr(predicates::str::contains(
                "\'subg\' requires a subcommand but one was not provided",
            ));
    }
}
