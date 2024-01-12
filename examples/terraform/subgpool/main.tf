# Copyright 2024 The Milton Hirsch Institute, B.V.
# SPDX-License-Identifier: Apache-2.0

locals {
  cidr = local.parsed_pool["cidr"]
  named_subnets = { for subnet in local.subnets : subnet["name"] => subnet["cidr"] if subnet["name"] != null }
  parsed_pool   = endswith(var.pool_path, ".json") ? jsondecode(local.pool_file) : yamldecode(local.pool_file)
  pool_file     = file(var.pool_path)
  subnets       = local.parsed_pool["subnets"]
}
