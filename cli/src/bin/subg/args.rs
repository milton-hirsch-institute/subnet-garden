// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub const DEFAULT_STORAGE_PATH: &str = "subnet-garden.json";

pub const SUBG_COMMAND: &str = "subg";

#[derive(Debug, clap::Args)]
/// Subnet gardener command line interface
pub struct SubgArgs {
    #[arg(short, long, default_value = DEFAULT_STORAGE_PATH)]
    pub garden_path: String,
}

#[derive(Debug, clap::Args)]
/// Initialize the subnet garden file
pub struct InitArgs {
    #[arg(short, long, default_value_t)]
    /// Force initialization even if the garden file already exists
    pub force: bool,
}

#[derive(Debug, clap::Args)]
/// Manage spaces
pub struct SpaceArgs {
    #[command(subcommand)]
    pub command: SpaceCommands,
}

#[derive(Debug, clap::Args)]
/// Create a new space.
pub struct SpaceNewArgs {
    #[arg()]
    /// The name of the space
    pub name: String,

    #[arg()]
    /// The managed CIDR space
    pub cidr: String,
}

#[derive(Debug, clap::Subcommand)]
pub enum SpaceCommands {
    New(SpaceNewArgs),
}

#[derive(Debug, clap::Subcommand)]
pub enum SubgCommands {
    Init(InitArgs),
    Space(SpaceArgs),
}

#[derive(Debug, clap::Parser)]
#[command(
    name = SUBG_COMMAND,
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
)]
pub struct Subg {
    #[command(flatten)]
    pub args: SubgArgs,

    #[command(subcommand)]
    pub command: SubgCommands,
}
