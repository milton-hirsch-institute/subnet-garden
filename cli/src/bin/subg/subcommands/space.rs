// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{SpaceArgs, SpaceCommands, SpaceNewArgs, SubgArgs};
use cidr::IpCidr;
use std::str::FromStr;
use subnet_garden_core::SubnetGarden;

fn new_space(subg: &SubgArgs, args: &SpaceNewArgs) {
    let mut garden = crate::load_garden(&subg.garden_path);
    let cidr = IpCidr::from_str(args.cidr.as_str()).unwrap();
    garden.new_space(args.name.as_str(), cidr).unwrap();
    crate::store_space(&subg.garden_path, &garden);
}

pub fn space(subg: &SubgArgs, args: &SpaceArgs) {
    match &args.command {
        SpaceCommands::New(args) => {
            new_space(subg, args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use subnet_garden_core::memory;

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
