use clap::Parser;

/// Bootstrap my dotfiles
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The list of targets to bootstrap. If not specified, will bootstrap everything.
    targets: Vec<dotfiles::Target>,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args.targets);
}
