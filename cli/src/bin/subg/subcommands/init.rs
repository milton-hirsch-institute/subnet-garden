// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::init::InitArgs;
use crate::args::SubgArgs;
use std::path::Path;
use std::process::exit;
use subnet_garden_core::pool;

pub(crate) fn init(subg: &SubgArgs, args: &InitArgs) {
    let path = Path::new(&subg.pool_path);
    if path.exists() {
        if !args.force {
            eprintln!("Pool file already exists at {}", path.display());
            exit(exitcode::CANTCREAT);
        }
        if !path.is_file() {
            eprintln!("Path is not a file at {}", path.display());
            exit(exitcode::CANTCREAT);
        }
    }
    crate::store_pool(&subg.pool_path, &pool::SubnetPool::new(args.cidr));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests;
    use crate::tests::{Test, TEST_CIDR};
    use assert_fs::prelude::*;
    fn new_init_test(cidr: &str) -> Test {
        let mut test = tests::new_test();
        test.subg.arg("init").arg(cidr);
        test
    }
    #[test]
    fn unforced() {
        let mut test = new_init_test(TEST_CIDR);
        test.subg.assert().success().stdout("").stderr("");

        let pool = pool::SubnetPool::new(TEST_CIDR.parse().unwrap());
        let expected_content = serde_json::to_string_pretty(&pool).unwrap();
        test.pool_path.assert(expected_content);
    }

    #[test]
    fn already_exists() {
        let mut test = new_init_test(TEST_CIDR);
        test.pool_path.touch().unwrap();
        test.subg
            .assert()
            .failure()
            .code(exitcode::CANTCREAT)
            .stdout("")
            .stderr(predicates::str::starts_with(format!(
                "Pool file already exists at {}",
                test.pool_path.display()
            )));

        test.pool_path.assert("");
    }

    #[test]
    fn forced() {
        let mut test = new_init_test(TEST_CIDR);
        test.subg.arg("--force");
        test.subg.assert().success().stdout("").stderr("");

        let pool = pool::SubnetPool::new(TEST_CIDR.parse().unwrap());
        let expected_content = serde_json::to_string_pretty(&pool).unwrap();
        test.pool_path.assert(expected_content);
    }

    #[test]
    fn not_a_file() {
        let mut test = new_init_test(TEST_CIDR);
        test.subg.arg("--force");
        test.pool_path.create_dir_all().unwrap();
        test.subg
            .assert()
            .failure()
            .code(exitcode::CANTCREAT)
            .stdout("")
            .stderr(predicates::str::starts_with(format!(
                "Path is not a file at {}",
                test.pool_path.display()
            )));

        test.pool_path.assert(predicates::path::is_dir());
    }

    #[test]
    fn bad_pool_file() {
        let mut test = tests::new_test_with_path("/bad/path");
        test.subg.arg("init").arg(TEST_CIDR);
        test.subg
            .assert()
            .failure()
            .code(exitcode::CANTCREAT)
            .stdout("")
            .stderr(format!(
                "Could not create pool file at {}\n\
                No such file or directory (os error 2)\n",
                test.pool_path.display()
            ));
    }
}
