# Copyright 2024 The Milton Hirsch Institute, B.V.
# SPDX-License-Identifier: Apache-2.0

variable "pool_path" {
  type    = string
  default = "subnet-garden-pool.yaml"

  validation {
    condition     = endswith(var.pool_path, ".json") || endswith(var.pool_path, ".yaml") || endswith(var.pool_path, ".yml")
    error_message = "pool_path must be a .json, .yaml, or .yml file"
  }
}
