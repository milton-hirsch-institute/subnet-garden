# Copyright 2024 The Milton Hirsch Institute, B.V.
# SPDX-License-Identifier: Apache-2.0

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
}
