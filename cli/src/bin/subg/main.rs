// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{Subg, SubgCommands};
use clap;
use clap::Parser;
use exitcode::ExitCode;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use subcommands::init;
use subcommands::subnet;
use subnet_garden_core::garden;

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

fn load_garden(garden_path: &str) -> garden::SubnetGarden {
    let path = Path::new(garden_path);
    if !path.exists() {
        eprintln!("Garden file does not exist at {}", path.display());
        exit(exitcode::NOINPUT);
    }
    if !path.is_file() {
        eprintln!("Path is not a file at {}", path.display());
        exit(exitcode::NOINPUT);
    }
    let garden_file = File::open(path).unwrap();
    serde_json::from_reader(garden_file).unwrap()
}

fn store_space(garden_path: &str, garden: &garden::SubnetGarden) {
    let path = Path::new(garden_path);

    let mut garden_file = result(
        File::create(path),
        exitcode::CANTCREAT,
        &format!("Could not create garden file at {}", path.display()),
    );

    result(
        serde_json::to_writer_pretty(&mut garden_file, &garden),
        exitcode::CANTCREAT,
        &format!("Could not serialize garden file at {}", path.display()),
    );
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
        pub(crate) subgarden_path: ChildPath,
        pub(crate) garden: garden::SubnetGarden,
    }

    impl Test {
        pub(crate) fn store(&self) {
            store_space(self.subgarden_path.to_str().unwrap(), &self.garden);
        }

        pub(crate) fn load(&mut self) {
            let garden_path = self.subgarden_path.to_str().unwrap();
            self.garden = load_garden(garden_path);
        }
    }

    pub(crate) fn new_test_with_path(path: &str) -> Test {
        let mut test = assert_cmd::Command::cargo_bin(SUBG_COMMAND).unwrap();
        let dir = assert_fs::TempDir::new().unwrap();
        let subgarden_path = dir.child(path);
        test.args(&["--garden-path", subgarden_path.to_str().unwrap()]);
        Test {
            subg: test,
            _dir: dir,
            subgarden_path,
            garden: garden::SubnetGarden::new(TEST_CIDR.parse().unwrap()),
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
