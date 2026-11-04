#[derive(clap::Parser, Debug)]
#[command(
    name = "casrs",
    bin_name = "casrs",
    version = crate::consts::VERSION,
    about = "Operate casrs: manage CA families, issuer profiles and more.",
    propagate_version = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Args, Debug)]
pub struct GlobalArgs {
    #[arg(short = 'v', long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Noop,
}
