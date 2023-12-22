// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{InitArgs, SubgArgs};
use std::fs::File;
use std::path::Path;
use std::process::exit;
use subnet_garden_core::memory;

pub(crate) fn init(subg: &SubgArgs, args: &InitArgs) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests;
    use crate::tests::Test;
    use assert_fs::prelude::*;
    fn new_init_test() -> Test {
        let mut test = tests::new_test();
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
