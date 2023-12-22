// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{SpaceArgs, SpaceCommands, SpaceNewArgs, Subg, SubgCommands};
use args::SubgArgs;
use cidr::IpCidr;
use clap;
use clap::Parser;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use subcommands::init;
use subnet_garden_core::memory;
use subnet_garden_core::SubnetGarden;

mod args;
mod subcommands;

fn load_garden(garden_path: &String) -> memory::MemorySubnetGarden {
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

fn store_space(garden_path: &str, garden: &memory::MemorySubnetGarden) {
    let path = Path::new(garden_path);
    let mut garden_file = File::create(path).unwrap();
    serde_json::to_writer_pretty(&mut garden_file, &garden).unwrap();
}

fn new_space(subg: &SubgArgs, args: &SpaceNewArgs) {
    let mut garden = load_garden(&subg.garden_path);
    let cidr = IpCidr::from_str(args.cidr.as_str()).unwrap();
    garden.new_space(args.name.as_str(), cidr).unwrap();
    store_space(&subg.garden_path, &garden);
}

fn space(subg: &SubgArgs, args: &SpaceArgs) {
    match &args.command {
        SpaceCommands::New(args) => {
            new_space(subg, args);
        }
    }
}

fn main() {
    let subg = Subg::parse();

    match subg.command {
        SubgCommands::Init(args) => {
            init::init(&subg.args, &args);
        }
        SubgCommands::Space(args) => {
            space(&subg.args, &args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{DEFAULT_STORAGE_PATH, SUBG_COMMAND};
    use assert_fs::fixture::ChildPath;
    use assert_fs::fixture::PathChild;

    const HELP_EXIT_CODE: i32 = 2;

    pub(crate) struct Test {
        pub subg: assert_cmd::Command,
        pub _dir: assert_fs::TempDir,
        pub subgarden_path: ChildPath,
    }

    impl Test {
        fn store(&self, garden: &memory::MemorySubnetGarden) {
            store_space(self.subgarden_path.to_str().unwrap(), garden);
        }

        fn load(&self) -> memory::MemorySubnetGarden {
            load_garden(&String::from(self.subgarden_path.to_str().unwrap()))
        }
    }

    pub(crate) fn new_test() -> Test {
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
            .stderr(predicates::str::contains(
                "\'subg\' requires a subcommand but one was not provided",
            ));
    }

    mod space {
        use super::*;

        fn new_space_test() -> Test {
            let mut test = new_test();
            let mut garden = memory::MemorySubnetGarden::new();
            garden
                .new_space("exists", IpCidr::from_str("10.20.0.0/24").unwrap())
                .unwrap();
            test.store(&garden);
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
                .stderr(predicates::str::contains("Usage: subg space <COMMAND>"));
        }
        mod new {
            use super::*;

            fn new_new_space_test() -> Test {
                let mut test = new_space_test();
                test.subg.arg("new");
                test
            }

            #[test]
            fn no_args() {
                let mut test = new_new_space_test();
                test.subg
                    .assert()
                    .failure()
                    .code(HELP_EXIT_CODE)
                    .stdout(predicates::str::contains(""))
                    .stderr(predicates::str::contains(
                        "the following required arguments were not provided:\n  <NAME>",
                    ));
            }

            #[test]
            fn new_space() {
                let mut test = new_new_space_test();
                test.subg
                    .arg("new")
                    .arg("10.10.0.0/24")
                    .assert()
                    .success()
                    .stdout(predicates::str::contains(""))
                    .stderr("");
                let stored = test.load();
                assert_eq!(stored.space_names(), vec!["exists", "new"]);
            }
        }
    }
}
