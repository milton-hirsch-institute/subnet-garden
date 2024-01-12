# Copyright 2024 The Milton Hirsch Institute, B.V.
# SPDX-License-Identifier: Apache-2.0

module "pool" {
  source = "./subgpool"
}

resource aws_vpc "institute" {
    cidr_block = module.pool.cidr
}

resource aws_subnet "tardigrade_lab" {
  vpc_id = aws_vpc.institute.id
  cidr_block = module.pool.subnets["tardigrade-lab"]
}

resource aws_subnet "rotifer_lab" {
  vpc_id = aws_vpc.institute.id
  cidr_block = module.pool.subnets["rotifer-lab"]
}
