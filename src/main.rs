use dotfiles::bootstrap;

fn main() {
    if let Err(e) = bootstrap() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
