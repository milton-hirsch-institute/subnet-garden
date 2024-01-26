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
that covers the number of hosts-bits you need.

#### Allocate an anonymous subnet

For example, to allocate a subnet that can hold 8-bits worth of hosts (256
hosts), run:

```shell
subg allocate 8
```

This will allocate an 8-bit subnet from the pool. The location of the subnet
is determined by available space in the pool.

#### Allocate a named subnet

A subnets may be assigned names. Once assigned, the name may be referenced
in other commands. For example, to allocate a subnet with a name, run:

```shell
subg allocate 8 tardigrade-lab
```

This will create an 8-bit subnet with the name `tardigrade-lab`.

#### Allocate a set of subnets

In many cases one needs to allocate a set of subnets, such as when building
a system across multiple datacenters and availability zones. For example,
to allocate 2 8-bit subnets in each of two availability zones and two
regions, run:

```shell
subg allocate 8 tardigrade-project-{}-{}-{} us-east-1,eu-central-1 a,b %0..2
```

This will allocate 8 subnets in total:

```text
tardigrade-project-us-east-1-a-0
tardigrade-project-us-east-1-a-1
tardigrade-project-us-east-1-b-0
tardigrade-project-us-east-1-b-1
tardigrade-project-eu-central-1-a-0
tardigrade-project-eu-central-1-a-1
tardigrade-project-eu-central-1-b-0
tardigrade-project-eu-central-1-b-1
```

#### Claim a specific CIDR

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

# Subnet name template

When describing a set of subnets, the name parameter becomes a template
used for describing the name of each new subnet. The arguments after the
template are generated in every combination. The values are substituted
into each `{}` placeholder in the template.

## List parameters

List parameters are specified as a comma-separated list of values. For
example, to generate a list of subnets for each of the availability zones
`a` and `b`, run:

```shell
subg allocate 8 tardigrade-experiment-az-{} a,b
```

This will create two networks, `tardigrade-experiment-az-a` and
`tardigrade-experiment-az-b`.

## Range parameters

Range parameters are used to specify a range of numbers. For example, to
generate a list of 6 subnets, run:

```shell
subg allocate 8 rotifer-experiment-{} %0..6
```

Note that the range is exclusive of the last number. This will create
6 subnets

```text
rotifer-experiment-0
rotifer-experiment-1
rotifer-experiment-2
rotifer-experiment-3
rotifer-experiment-4
rotifer-experiment-5
```

# Subnet garden pool format

The subnet garden pool file is stored either as a YAML or JSON file.
Here is an example of a YAML pool file:

```yaml
cidr: 10.10.0.0/16
subnets:
- cidr: 10.10.0.0/24
- cidr: 10.10.1.0/24
  name: tardigrade-lab
- cidr: 10.10.110.0/24
```
