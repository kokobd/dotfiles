use age::{Identity, Recipient};
use anyhow::{anyhow, bail};
use std::io::{BufRead, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, iter};

pub fn decrypt(filepath: String) -> anyhow::Result<()> {
    let filepath = Path::new(&filepath);
    if filepath.extension().and_then(|x| x.to_str()) != Some("rage") {
        bail!(
            "encrypted file must have .age extension. actual patH: {:?}",
            filepath
        );
    }

    let encrypted_data = Cursor::new(fs::read(filepath)?);
    let identity_file_path = {
        let mut path = locate_ssh_dir()?;
        path.push("id_ed25519");
        path
    };
    let private_key = fs::read_to_string(identity_file_path.as_path())?;
    let identity = age::ssh::Identity::from_buffer(Cursor::new(private_key), None)
        .map_err(|err| anyhow!(err))?;
    let decrypted_data = decrypt_pure(&identity, encrypted_data)?;

    fs::write(filepath.with_extension(""), decrypted_data)?;
    fs::remove_file(filepath)?;

    Ok(())
}

fn locate_ssh_dir() -> anyhow::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(anyhow!("Could not find home directory"))?;
    let mut identity_file_path = home_dir;
    identity_file_path.push(".ssh");
    Ok(identity_file_path)
}

fn decrypt_pure<R: BufRead>(key: &dyn Identity, encrypted: R) -> anyhow::Result<Vec<u8>> {
    let decryptor = match age::Decryptor::new_buffered(encrypted)? {
        age::Decryptor::Recipients(d) => d,
        _ => unreachable!(),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor.decrypt(iter::once(key))?;
    reader.read_to_end(&mut decrypted)?;

    Ok(decrypted)
}

pub fn encrypt(filepath: String) -> anyhow::Result<()> {
    let filepath = Path::new(&filepath);
    let public_key = {
        let mut path = locate_ssh_dir()?;
        path.push("id_ed25519.pub");
        fs::read_to_string(path.as_path())?
    };
    let recipient: Box<dyn Recipient + Send> = Box::new(
        age::ssh::Recipient::from_str(&public_key)
            .map_err(|_| anyhow!("failed to parse public key"))?,
    );
    let plaintext = fs::read(filepath)?;
    let encrypted = encrypt_pure(recipient, &plaintext)?;
    fs::write(filepath.with_extension("rage"), encrypted)?;
    fs::remove_file(filepath)?;
    Ok(())
}

fn encrypt_pure(recipient: Box<dyn Recipient + Send>, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let encryptor =
        age::Encryptor::with_recipients(vec![recipient]).expect("we provided a recipient");

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(plaintext)?;
    writer.finish()?;

    Ok(encrypted)
}
