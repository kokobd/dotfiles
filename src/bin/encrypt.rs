use dotfiles::secret;

fn main() {
    match std::env::args().nth(1) {
        Some(filepath) => {
            if let Err(e) = secret::encrypt(filepath) {
                println!("encryption failed: {}", e);
            }
        }
        None => println!("You must specify the path to the file to decrypt"),
    }
}
