// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{Subg, SubgCommands};

use clap::Parser;
use subcommands::init;
use subcommands::subnet;
use subcommands::subnet::listing;

mod args;
mod param_str;
mod subcommands;
mod util;

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
        SubgCommands::Cidrs(args) => {
            listing::cidrs(&subg.args, &args);
        }
        SubgCommands::Names(args) => {
            listing::names(&subg.args, &args);
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
    use assert_fs::fixture::ChildPath;
    use assert_fs::fixture::PathChild;
    use subnet_garden_core as subg_core;

    pub(crate) const TEST_CIDR: &str = "10.10.0.0/16";

    pub(crate) struct Test {
        pub(crate) subg: assert_cmd::Command,
        pub(crate) _dir: assert_fs::TempDir,
        pub(crate) pool_path: ChildPath,
        pub(crate) pool: subg_core::pool::SubnetPool,
    }

    impl Test {
        pub(crate) fn store(&self) {
            subg::store_pool(self.pool_path.to_str().unwrap(), &self.pool);
        }
    }

    pub(crate) fn new_test_with_path(path: &str) -> Test {
        let mut test = assert_cmd::Command::cargo_bin(subg::SUBG_COMMAND).unwrap();
        let dir = assert_fs::TempDir::new().unwrap();
        let pool_path = dir.child(path);
        test.args(["--pool-path", pool_path.to_str().unwrap()]);
        Test {
            subg: test,
            _dir: dir,
            pool_path,
            pool: subg_core::pool::SubnetPool::new(TEST_CIDR.parse().unwrap()),
        }
    }

    pub(crate) fn new_test() -> Test {
        new_test_with_path(subg::DEFAULT_STORAGE_PATH)
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Subg::command().debug_assert();
    }
}
