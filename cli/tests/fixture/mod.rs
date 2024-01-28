// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use assert_fs::fixture::ChildPath;
use assert_fs::fixture::PathChild;
use subnet_garden_core::pool;

pub(crate) const HELP_EXIT_CODE: i32 = 2;

pub(crate) const TEST_CIDR: &str = "10.10.0.0/16";

pub(crate) struct Test {
    pub(crate) subg: assert_cmd::Command,
    pub(crate) _dir: assert_fs::TempDir,
    pub(crate) pool_path: ChildPath,
    pub(crate) pool: pool::SubnetPool,
}

impl Test {
    #[allow(dead_code)]
    pub(crate) fn store(&self) {
        subg::store_pool(self.pool_path.to_str().unwrap(), &self.pool);
    }

    pub(crate) fn load(&mut self) {
        self.pool = subg::load_pool(self.pool_path.to_str().unwrap());
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
        pool_path: pool_path,
        pool: pool::SubnetPool::new(TEST_CIDR.parse().unwrap()),
    }
}

pub(crate) fn new_test() -> Test {
    new_test_with_path(subg::DEFAULT_STORAGE_PATH)
}
