# Copyright 2024 The Milton Hirsch Institute, B.V.
# SPDX-License-Identifier: Apache-2.0

module "pool" {
  source = "./subgpool"
}

output "cdir" {
  value = module.pool.cidr
}

output "pool" {
  value = module.pool.subnets
}
