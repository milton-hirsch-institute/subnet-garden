<!--
 Copyright 2024 The Milton Hirsch Institute, B.V.
 SPDX-License-Identifier: Apache-2.0
 -->

# Subnet Garden

## Description

Subnet Garden is a source-control based approach to managing network infrastructure
similar to [IPAM](https://en.wikipedia.org/wiki/IP_address_management) without the
need to set up a dedicated IPAM server. The database is a human-readable text file
that can be edited with any text editor and is stored in a git repository.

## Features

* Human-readable text file
* Git-based version control
* Automatic subnet assignment

# How to use it

## Installation

Using cargo:

```shell
cargo install subg
```

This will install the `subg` binary in `~/.cargo/bin`. Make sure that this directory
is in your `PATH` environment variable.

## Initialization

Subnet garden stores its data in a pool file. This file is a source-control friendly
text file that can be edited with any text editor and easily placed under version
control. Each pool file is responsible for managing subnets under a single parent
subnet.

For example, To create a new pool file that will managing the `10.10.0.0/16` subnet,
run:

```shell
subg init 10.10.0.0/16
```

This will createa new pool file in the current directory called `subnet-garden-pool.yaml`.

You may also manage IPv6 subnets. For example, to manage the `fc00::/112` subnet,
run:

```shell
subg init fc00::/112
```

You can specify alternate names for the pool file using the `--pool-path` option:

```shell
subg init --pool-path institute-network.yaml 10.10.0.0/16
```

You can also set the `SUBG_POOL_PATH` environment variable to specify the pool file.
This is useful to avoid having to specify the `--pool-path` option every time you
run a command:

```shell
# Using an environment variable
export SUBG_POOL_PATH=institute-network.yaml
subg init 10.10.0.0/16
```

It is also possible to store the pool file as JSON instead of YAML:

```shell
export SUBG_POOL_PATH=subnet-garden-pool.json
subg init 10.10.0.0/16
```

## Managing subnets

Once you have initialized a pool file, you can start allocate, deallocate, and
see information about managed subnets.

### Subnet allocation

When you need to allocate a new subnet, you can do so by requesting a subnet
that covers the number of hosts-bits you need. For example, to allocate a subnet
that can hold 8-bits worth of hosts (256 hosts), run:

```shell
subg allocate 8
```

This will allocate an 8-bit subnet from the pool. The location of the subnet
is determined by available space in the pool. Subnets may be assigned names.
Once assigned, the name may be referenced in other commands. For example, to
allocate a subnet with a name, run:

```shell
subg allocate 8 tardigrade-lab
```

This will create an 8-bit subnet with the name `tardigrade-lab`.

In some cases you may want to allocate a subnet with a specific address. For
example, when applying a subnet pool to an existing network. To claim
a specific subnet, run:

```shell
subg claim 10.10.110.0/24
```

### Seeing allocated subnets

To see the subnets that have been allocated, run:

```shell
subg cidrs
```

To see a list of named subnets use:

```shell
subg names
```

### Subnet naming

It is possible to add, change or remove the name of a subnet. Examples:

Add a name to an unnamed subnet:

```shell
subg rename 10.10.0.0/24 rotifer-lab
```

Change the name of a subnet:

```shell
subg rename rotifer-lab other-microbe-lab
```

To remove the name of a subnet, omit the name:

```shell
subg rename other-microbe-lab
```

### Freeing subnets

Subnets are freed by name or CIDR. For example, to free the previously
claimed subnet, run:

```shell
subg free 10.10.110.0/24
```

# Subnet garden pool format

The subnet garden pool file is stored either as a YAML or JSON file.
Here is an example of a YAML pool file:

```yaml
cidr: 10.10.0.0/16
subnets:
- cidr: 10.10.0.0/24
  name: null
- cidr: 10.10.1.0/24
  name: tardigrade-lab
- cidr: 10.10.110.0/24
  name: null
```
