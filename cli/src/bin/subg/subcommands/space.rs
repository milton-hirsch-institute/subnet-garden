// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::space::{SpaceArgs, SpaceCommands, SpaceDeleteArgs, SpaceNewArgs};
use crate::args::SubgArgs;
use cidr::IpCidr;
use std::str::FromStr;
use subnet_garden_core::SubnetGarden;

fn new_space(subg: &SubgArgs, args: &SpaceNewArgs) {
    let mut garden = crate::load_garden(&subg.garden_path);
    let cidr = crate::result(
        IpCidr::from_str(args.cidr.as_str()),
        exitcode::USAGE,
        "Invalid CIDR parameter",
    );
    crate::result(
        garden.new_space(args.name.as_str(), cidr),
        exitcode::CANTCREAT,
        "Could not create space",
    );
    crate::store_space(&subg.garden_path, &garden);
}

fn delete_space(subg: &SubgArgs, args: &SpaceDeleteArgs) {
    let mut garden = crate::load_garden(&subg.garden_path);
    crate::result(
        garden.delete_space(args.name.as_str()),
        exitcode::NOINPUT,
        "Could not delete space",
    );
    crate::store_space(&subg.garden_path, &garden);
}

fn list_spaces(subg: &SubgArgs) {
    let garden = crate::load_garden(&subg.garden_path);
    for space in garden.space_names() {
        println!("{}", space);
    }
}

pub fn space(subg: &SubgArgs, args: &SpaceArgs) {
    match &args.command {
        SpaceCommands::New(args) => {
            new_space(subg, args);
        }
        SpaceCommands::Delete(args) => {
            delete_space(subg, args);
        }
        SpaceCommands::List(_) => {
            list_spaces(subg);
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
        fn cannot_create_space() {
            let mut test = new_new_space_test();
            test.subg.arg("exists").arg("10.10.0.0/24");
            test.subg
                .assert()
                .failure()
                .code(exitcode::CANTCREAT)
                .stderr("Could not create space\nDuplicate object\n");
        }

        #[test]
        fn invalid_cidr() {
            let mut test = new_new_space_test();
            test.subg.arg("exists").arg("bad-cidr");
            test.subg.assert().failure().code(exitcode::USAGE).stderr(
                "Invalid CIDR parameter\n\
                couldn\'t parse address in network: invalid IP address syntax\n",
            );
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

    mod delete {
        use super::*;

        fn new_delete_space_test() -> Test {
            let mut test = new_space_test();
            test.subg.arg("delete");
            test
        }
        #[test]
        fn cannot_delete_space() {
            let mut test = new_delete_space_test();
            test.subg.arg("does-not-exist");
            test.subg
                .assert()
                .failure()
                .code(exitcode::NOINPUT)
                .stderr("Could not delete space\nNo such object\n");
        }

        #[test]
        fn delete_space() {
            let mut test = new_delete_space_test();
            test.subg.arg("exists");
            test.subg
                .assert()
                .success()
                .stdout(predicates::str::contains(""))
                .stderr("");
            let stored = test.load();
            assert_eq!(stored.space_names(), Vec::<String>::new());
        }
    }

    mod list {
        use super::*;

        fn new_list_spaces_test() -> Test {
            let mut test = new_space_test();
            test.subg.arg("list");
            test
        }

        #[test]
        fn list_spaces() {
            let mut test = new_list_spaces_test();
            test.subg
                .assert()
                .success()
                .stdout(predicates::str::contains("exists\n"))
                .stderr("");
        }
    }
}
