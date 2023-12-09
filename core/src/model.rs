// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::CreateError;
use cidr::IpCidr;

pub type CreateResult<T> = std::result::Result<T, CreateError>;

pub trait SubnetGarden {
    fn space_count(&self) -> usize;
    fn new_space(&mut self, name: &str, cidr: &IpCidr) -> CreateResult<()>;

    fn space_cidr(&self, name: &str) -> Option<&IpCidr>;
}
