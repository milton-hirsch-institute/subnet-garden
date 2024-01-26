# Copyright 2024 The Milton Hirsch Institute, B.V.
# SPDX-License-Identifier: Apache-2.0

output "cidr" {
  value = local.cidr
}

output "subnets" {
  value = local.named_subnets
}
