// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use cidr::IpCidr;
use clap;
use clap::Parser;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use subnet_garden_core::memory;
use subnet_garden_core::model::SubnetGarden;

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
pub struct SpaceArgs {
    #[command(subcommand)]
    pub command: SpaceCommands,
}

#[derive(Debug, clap::Args)]
/// Create a new space.
pub struct SpaceNewArgs {
    #[arg()]
    /// The name of the space
    pub name: String,

    #[arg()]
    /// The managed CIDR space
    pub cidr: String,
}

#[derive(Debug, clap::Subcommand)]
pub enum SpaceCommands {
    New(SpaceNewArgs),
}

#[derive(Debug, clap::Subcommand)]
pub enum SubgCommands {
    Init(InitArgs),
    Space(SpaceArgs),
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
            init(&subg.args, &args);
        }
        SubgCommands::Space(args) => {
            space(&subg.args, &args);
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

    impl Test {
        fn store(&self, garden: &memory::MemorySubnetGarden) {
            store_space(self.subgarden_path.to_str().unwrap(), garden);
        }

        fn load(&self) -> memory::MemorySubnetGarden {
            load_garden(&String::from(self.subgarden_path.to_str().unwrap()))
        }
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
            .stderr(predicates::str::contains(
                "\'subg\' requires a subcommand but one was not provided",
            ));
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
