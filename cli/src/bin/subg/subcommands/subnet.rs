// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod listing;

use crate::args::{AllocateArgs, ClaimArgs, FreeArgs, RenameArgs, SubgArgs};
use crate::param_str;
use cidr::IpCidr;
use std::process::exit;

pub(crate) fn allocate(subg: &SubgArgs, args: &AllocateArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    match &args.param {
        None => {
            subg::result(
                pool.allocate(args.bits, args.name_format.as_deref()),
                exitcode::SOFTWARE,
                "Could not allocate subnet",
            );
        }
        Some(params) => {
            let format = args.name_format.as_deref().unwrap();
            let param_strs: param_str::format::Args = params.iter().map(|s| s.as_str()).collect();
            let names = subg::result(
                param_str::format::format_strings(format, &param_strs),
                exitcode::SOFTWARE,
                "Could not format subnet names",
            );
            for name in names {
                subg::result(
                    pool.allocate(args.bits, Some(name.to_string().as_str())),
                    exitcode::SOFTWARE,
                    format!("Could not allocate subnet {}", name).as_str(),
                );
            }
        }
    };
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn free(subg: &SubgArgs, args: &FreeArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    let identifier_list = match args.param {
        None => vec![args.identifier_format.clone()],
        Some(ref params) => {
            let format = args.identifier_format.as_str();
            let param_strs: param_str::format::Args = params.iter().map(|s| s.as_str()).collect();
            subg::result(
                param_str::format::format_strings(format, &param_strs),
                exitcode::SOFTWARE,
                "Could not format subnet names",
            )
        }
    };
    for identifier in identifier_list {
        let cidr = match pool.find_by_name(identifier.as_str()) {
            Some(cidr) => cidr,
            None => {
                let parse_result = identifier.parse::<IpCidr>();
                if args.ignore_missing {
                    continue;
                }
                subg::result(
                    parse_result,
                    exitcode::USAGE,
                    format!("Could not parse arg IDENTIFIER: {}", identifier).as_str(),
                )
            }
        };
        if !pool.free(&cidr) && !args.ignore_missing {
            eprintln!("Could not free subnet {}", cidr);
            exit(exitcode::SOFTWARE);
        }
    }
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn claim(subg: &SubgArgs, args: &ClaimArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    subg::result(
        pool.claim(&args.cidr, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not claim subnet",
    );
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn rename(subg: &SubgArgs, args: &RenameArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    let cidr = match pool.find_by_name(args.identifier.as_str()) {
        Some(cidr) => cidr,
        None => subg::result(
            args.identifier.parse::<IpCidr>(),
            exitcode::USAGE,
            "Could not parse arg IDENTIFIER",
        ),
    };
    subg::result(
        pool.rename(&cidr, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not rename subnet",
    );
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn max_bits(subg: &SubgArgs) {
    let pool = subg::load_pool(&subg.pool_path);
    let largest = pool.max_available_bits();
    println!("{}", largest);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    mod allocate {
        use super::*;
        use subnet_garden_core::CidrRecord;
        fn new_allocate_test(bits: &str, name: Option<&str>) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("allocate").arg(bits);
            if let Some(name) = name {
                test.subg.arg(name);
            }
            test
        }

        #[test]
        fn allocate_single_failure() {
            let mut test = new_allocate_test("8", Some("test"));
            test.pool.allocate(16, None).unwrap();
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr("Could not allocate subnet\nNo space available\n");
        }

        #[test]
        fn allocate_multi_failure() {
            let mut test = new_allocate_test("8", Some("name-{}"));
            test.subg.arg("%0..129");
            test.pool.allocate(15, None).unwrap();
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr("Could not allocate subnet name-128\nNo space available\n");
        }

        #[test]
        fn allocate_format_failure() {
            let mut test = new_allocate_test("8", Some("name-{}"));
            test.subg.arg("%0..,");
            test.pool.allocate(15, None).unwrap();
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr(
                    "Could not format subnet names\n\
                        arg: %0..,\n\
                        if range: InvalidValue: Expected digit, found ,\n\
                        if list: InvalidValue: Unexpected end of list\n",
                );
        }

        #[test]
        fn allocate_with_name() {
            let mut test = new_allocate_test("8", Some("test"));
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }

        #[test]
        fn allocate_without_name() {
            let mut test = new_allocate_test("8", None);
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name, None);
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }

        #[test]
        fn allocate_multiple() {
            let mut test = new_allocate_test("8", Some("name-{}-{}"));
            test.subg.arg("%0..2");
            test.subg.arg("a,b");
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 4);
            assert_eq!(subnets[0].name.clone().unwrap(), "name-0-a");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
            assert_eq!(subnets[1].name.clone().unwrap(), "name-0-b");
            assert_eq!(subnets[1].cidr.to_string(), "10.10.1.0/24");
            assert_eq!(subnets[2].name.clone().unwrap(), "name-1-a");
            assert_eq!(subnets[2].cidr.to_string(), "10.10.2.0/24");
            assert_eq!(subnets[3].name.clone().unwrap(), "name-1-b");
            assert_eq!(subnets[3].cidr.to_string(), "10.10.3.0/24");
        }
    }

    mod free {
        use super::*;
        fn new_free_test(identifier: &str) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("free").arg(identifier);
            test
        }

        #[test]
        fn name_not_found() {
            let mut test = new_free_test("test");
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::USAGE)
                .stdout("")
                .stderr(
                    "Could not parse arg IDENTIFIER: test\n\
                couldn\'t parse address in network: invalid IP address syntax\n",
                );
        }

        #[test]
        fn free_failure() {
            let mut test = new_free_test("20.20.0.0/24");
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr("Could not free subnet 20.20.0.0/24\n");
        }

        #[test]
        fn free_success_with_name() {
            let mut test = new_free_test("test");
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.pool.find_by_name("test"), None);
        }

        #[test]
        fn free_success_with_cidr() {
            let mut test = new_free_test("10.10.0.0/28");
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.pool.find_by_name("test"), None);
        }

        #[test]
        fn free_success_multiple() {
            let mut test = new_free_test("test{}");
            test.subg.arg("%1..3");
            test.pool.allocate(4, Some("test1")).unwrap();
            test.pool.allocate(4, Some("test2")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.pool.find_by_name("test1"), None);
            assert_eq!(test.pool.find_by_name("test2"), None);
        }

        #[test]
        fn ignore_missing_name() {
            let mut test = new_free_test("test{}");
            test.subg.arg("%1..3");
            test.subg.arg("--ignore-missing");
            test.pool.allocate(4, Some("test1")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.pool.find_by_name("test1"), None);
            assert_eq!(test.pool.find_by_name("test2"), None);
        }

        #[test]
        fn ignore_missing_cidr() {
            let mut test = new_free_test("10.10.0.0/28");
            test.subg.arg("--ignore-missing");
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.pool.find_by_name("test"), None);
        }
    }

    mod claim {
        use super::*;
        use subnet_garden_core::CidrRecord;
        fn new_claim_test(cidr: &str, name: Option<&str>) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("claim").arg(cidr);
            if let Some(name) = name {
                test.subg.arg(name);
            }
            test
        }

        #[test]
        fn claim_failed() {
            let mut test = new_claim_test("20.20.0.0/24", Some("does-not-exist"));
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr(
                    "Could not claim subnet\n\
                No space available\n",
                );
        }

        #[test]
        fn unnamed() {
            let mut test = new_claim_test("10.10.0.0/24", None);
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name, None);
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }

        #[test]
        fn named() {
            let mut test = new_claim_test("10.10.0.0/24", Some("test"));
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }
    }

    mod rename {
        use super::*;
        use subnet_garden_core::CidrRecord;
        fn new_rename_test(identifier: &str, name: Option<&str>) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("rename").arg(identifier);
            if let Some(name) = name {
                test.subg.arg(name);
            }
            test
        }

        #[test]
        fn unknown() {
            let mut test = new_rename_test("bad-cidr", None);
            test.subg
                .assert()
                .failure()
                .code(exitcode::USAGE)
                .stdout("")
                .stderr(
                    "Could not parse arg IDENTIFIER\n\
                couldn\'t parse address in network: invalid IP address syntax\n",
                );
        }

        #[test]
        fn rename_failure() {
            let mut test = new_rename_test("10.10.0.0/24", Some("test"));
            test.pool.allocate(4, Some("test")).unwrap();
            test.pool.allocate(4, None).unwrap();
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr("Could not rename subnet\nDuplicate name\n");
        }

        #[test]
        fn success_with_name() {
            let mut test = new_rename_test("test", Some("test2"));
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test2");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/28");
        }

        #[test]
        fn success_with_cidr() {
            let mut test = new_rename_test("10.10.0.0/28", Some("test2"));
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test2");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/28");
        }
    }

    mod max_available {
        use super::*;
        fn new_max_available_test() -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("max-available");
            test
        }

        #[test]
        fn no_subnets() {
            let mut test = new_max_available_test();
            test.subg.assert().success().stdout("16\n").stderr("");
        }

        #[test]
        fn has_subnets() {
            let mut test = new_max_available_test();
            test.pool.allocate(4, Some("test1")).unwrap();
            test.pool.allocate(6, Some("test2")).unwrap();
            test.store();
            test.subg.assert().success().stdout("15\n").stderr("");

            test.pool.allocate(14, Some("test3")).unwrap();
            test.pool.allocate(15, Some("test4")).unwrap();
            test.store();
            test.subg.assert().success().stdout("13\n").stderr("");
        }
    }
}
