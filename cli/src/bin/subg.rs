// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use clap;
use clap::Parser;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use subnet_garden_core::memory;

const SUBG_COMMAND: &str = "subg";

const DEFAULT_STORAGE_PATH: &str = "subnet-garden.json";

#[derive(Debug, clap::Args)]
/// Subnet gardener command line interface
pub struct SubgArgs {
    #[arg(short, long, default_value = DEFAULT_STORAGE_PATH)]
    pub garden_path: String,
}

#[derive(Debug, clap::Args)]
/// Initialize the subnet garden file
pub struct InitArgs {
    #[arg(short, long, default_value_t)]
    /// Force initialization even if the garden file already exists
    pub force: bool,
}

#[derive(Debug, clap::Args)]
/// Manage spaces
pub struct SpaceArgs {}

#[derive(Debug, clap::Subcommand)]
pub enum SubgCommands {
    Init(InitArgs),
}

#[derive(Debug, clap::Parser)]
#[command(
    name = SUBG_COMMAND,
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
)]
pub struct Subg {
    #[command(flatten)]
    pub args: SubgArgs,

    #[command(subcommand)]
    pub command: SubgCommands,
}

fn init(subg: &SubgArgs, args: &InitArgs) {
    let path = Path::new(&subg.garden_path);
    if path.exists() {
        if !args.force {
            eprintln!("Garden file already exists at {}", path.display());
            exit(exitcode::CANTCREAT);
        }
        if !path.is_file() {
            eprintln!("Path is not a file at {}", path.display());
            exit(exitcode::CANTCREAT);
        }
    }
    let new_garden = memory::MemorySubnetGarden::new();
    let mut garden_file = File::create(path).unwrap();
    serde_json::to_writer_pretty(&mut garden_file, &new_garden).unwrap();
}

fn main() {
    let subg = Subg::parse();

    match subg.command {
        SubgCommands::Init(args) => {
            init(&subg.args, &args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::fixture::ChildPath;
    use assert_fs::fixture::PathChild;
    use assert_fs::prelude::*;

    const HELP_EXIT_CODE: i32 = 2;

    struct Test {
        subg: assert_cmd::Command,
        _dir: assert_fs::TempDir,
        subgarden_path: ChildPath,
    }

    fn new_test() -> Test {
        let mut subg = assert_cmd::Command::cargo_bin(SUBG_COMMAND).unwrap();
        let dir = assert_fs::TempDir::new().unwrap();
        let subgarden_path = dir.child(DEFAULT_STORAGE_PATH);
        subg.args(&["--garden-path", subgarden_path.to_str().unwrap()]);
        Test {
            subg,
            _dir: dir,
            subgarden_path,
        }
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
            .stdout(predicates::str::contains("Usage: subg"))
            .stderr("");
    }

    mod init {
        use super::*;

        fn new_init_test() -> Test {
            let mut test = new_test();
            test.subg.arg("init");
            test
        }
        #[test]
        fn unforced() {
            let mut test = new_init_test();
            test.subg.assert().success().stdout("").stderr("");

            let garden = memory::MemorySubnetGarden::new();
            let expected_content = serde_json::to_string_pretty(&garden).unwrap();
            test.subgarden_path.assert(expected_content);
        }

        #[test]
        fn already_exists() {
            let mut test = new_init_test();
            test.subgarden_path.touch().unwrap();
            test.subg
                .assert()
                .failure()
                .code(exitcode::CANTCREAT)
                .stdout("")
                .stderr(predicates::str::starts_with(format!(
                    "Garden file already exists at {}",
                    test.subgarden_path.display()
                )));

            test.subgarden_path.assert("");
        }

        #[test]
        fn forced() {
            let mut test = new_init_test();
            test.subg.arg("--force");
            test.subg.assert().success().stdout("").stderr("");

            let garden = memory::MemorySubnetGarden::new();
            let expected_content = serde_json::to_string_pretty(&garden).unwrap();
            test.subgarden_path.assert(expected_content);
        }

        #[test]
        fn not_a_file() {
            let mut test = new_init_test();
            test.subg.arg("--force");
            test.subgarden_path.create_dir_all().unwrap();
            test.subg
                .assert()
                .failure()
                .code(exitcode::CANTCREAT)
                .stdout("")
                .stderr(predicates::str::starts_with(format!(
                    "Path is not a file at {}",
                    test.subgarden_path.display()
                )));

            test.subgarden_path.assert(predicates::path::is_dir());
        }
    }

    mod space {
        use super::*;

        fn new_space_test() -> Test {
            let mut test = new_test();
            test.subg.arg("space");
            test
        }

        #[test]
        fn no_args() {
            let mut test = new_space_test();
            test.subg
                .assert()
                .failure()
                .code(HELP_EXIT_CODE)
                .stdout(predicates::str::contains("Usage: space"))
                .stderr("");
        }
    }
}
