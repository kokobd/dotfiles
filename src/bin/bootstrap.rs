use clap::{builder::ValueParser, Parser};
use dotfiles::{bootstrap, Config, Region};
use thiserror::Error;

/// Bootstrap my dotfiles
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The list of targets to bootstrap. If not specified, will bootstrap everything.
    targets: Vec<dotfiles::Target>,
    #[arg(long, value_parser=region_parser())]
    region: Region,
    #[arg(long)]
    ssh_private_key: String,
}

fn region_parser() -> ValueParser {
    ValueParser::new(|input: &str| -> Result<Region, RegionParserError> {
        if input == "home" {
            Ok(Region::Home)
        } else {
            match input.strip_prefix("aws-") {
                None => Err(RegionParserError(input.to_string())),
                Some(aws_region) => Ok(Region::AWS {
                    region: aws_region.to_string(),
                }),
            }
        }
    })
}

#[derive(Error, Debug)]
#[error("Invalid region: {0}")]
struct RegionParserError(String);

fn main() {
    let args = Args::parse();
    match Config::new(args.region, args.ssh_private_key) {
        Ok(config) => match bootstrap(config, expand_default_targets(args.targets)) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn expand_default_targets(targets: Vec<dotfiles::Target>) -> Vec<dotfiles::Target> {
    if targets.is_empty() {
        dotfiles::all_targets()
    } else {
        targets
    }
}
