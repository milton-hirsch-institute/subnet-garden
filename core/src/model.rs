// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::CreateError;
use cidr::IpCidr;

pub type CreateResult<T> = std::result::Result<T, CreateError>;

pub trait Space {
    fn cidr(&self) -> &IpCidr;
}
pub trait SubnetGarden {
    fn space_count(&self) -> usize;
    fn new_space(&mut self, name: &str, cidr: &IpCidr) -> CreateResult<&dyn Space>;

    fn space(&self, name: &str) -> Option<&dyn Space>;
}
