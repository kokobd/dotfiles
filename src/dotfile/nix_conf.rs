use std::{any::Any, convert::identity, error::Error, os::unix::fs::PermissionsExt};

use crate::dotfile::apply_utf8;

use super::{merge_same_type, ApplyError, Dotfile, MergeError};

/**
/etc/nix/nix.conf: https://nixos.org/manual/nix/stable/command-ref/conf-file
*/
#[derive(Debug, Default)]
pub struct NixConf(NixConfExt<()>);

#[derive(Debug, Default, Clone)]
pub struct NixConfExt<T> {
    substituters: (Vec<String>, T),
    trusted_public_keys: (Vec<String>, T),
    post_build_hook: (Option<String>, T),
    secret_key_files: (Vec<String>, T),
}

impl<T> NixConfExt<T> {
    fn new<F>(new_ext: F) -> Self
    where
        F: Fn() -> T,
    {
        Self {
            substituters: (Vec::new(), new_ext()),
            trusted_public_keys: (Vec::new(), new_ext()),
            post_build_hook: (None, new_ext()),
            secret_key_files: (Vec::new(), new_ext()),
        }
    }
}

impl NixConf {
    pub fn new() -> Self {
        Self(NixConfExt::new(|| ()))
    }

    pub fn with_substituters(mut self, substituters: Vec<String>) -> Self {
        self.0.substituters.0 = substituters;
        self
    }

    pub fn with_trusted_public_keys(mut self, trusted_public_keys: Vec<String>) -> Self {
        self.0.trusted_public_keys.0 = trusted_public_keys;
        self
    }

    pub fn with_post_build_hook(mut self, post_build_hook: String) -> Self {
        self.0.post_build_hook.0 = Some(post_build_hook);
        self
    }

    pub fn with_secret_key_files(mut self, secret_key_files: Vec<String>) -> Self {
        self.0.secret_key_files.0 = secret_key_files;
        self
    }
}

#[derive(Debug)]
struct Line {
    index: Option<usize>,
    content: String,
}

impl Dotfile for NixConf {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(*self)
    }

    fn file_permission(&self) -> std::fs::Permissions {
        std::fs::Permissions::from_mode(0o644)
    }

    fn apply(&self, old_content: &[u8]) -> Result<Option<Vec<u8>>, ApplyError> {
        apply_utf8::<Self, _>(
            old_content,
            |old_content: String| -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
                let lines: Vec<String> = old_content.lines().map(|s| String::from(s)).collect();

                let mut existing_config: NixConfExt<Option<Line>> = NixConfExt::new(|| None);

                for (index, content) in lines.iter().enumerate() {
                    parse_list(
                        "substituters",
                        index,
                        content,
                        &mut existing_config.substituters,
                    );
                    parse_list(
                        "trusted-public-keys",
                        index,
                        content,
                        &mut existing_config.trusted_public_keys,
                    );
                    parse_single_value(
                        "post-build-hook",
                        index,
                        content,
                        &mut existing_config.post_build_hook,
                    );
                    parse_list(
                        "secret-key-files",
                        index,
                        content,
                        &mut existing_config.secret_key_files,
                    );
                }
                let new_config = merge(self.0.clone(), existing_config)?;
                let new_lines: Vec<Line> = vec![
                    render_list("substituters", new_config.substituters),
                    render_list("trusted-public-keys", new_config.trusted_public_keys),
                    render_single_value("post-build-hook", new_config.post_build_hook),
                    render_list("secret-key-files", new_config.secret_key_files),
                ]
                .into_iter()
                .filter_map(identity)
                .collect();

                let lines = merge_lines(lines, new_lines);
                let new_content = lines.join("\n");
                if old_content == new_content {
                    Ok(None)
                } else {
                    Ok(Some(new_content))
                }
            },
        )
    }

    fn merge(&mut self, y: Box<dyn Dotfile>) -> Result<(), MergeError> {
        merge_same_type(self, y, |x, y| Ok(NixConf(merge(x.0, y.0)?)))
    }
}

fn merge<Ext>(x: NixConfExt<()>, y: NixConfExt<Ext>) -> Result<NixConfExt<Ext>, MergeError> {
    Ok(NixConfExt {
        substituters: merge_list(x.substituters, y.substituters),
        trusted_public_keys: merge_list(x.trusted_public_keys, y.trusted_public_keys),
        post_build_hook: merge_single_value(
            "post_build_hook",
            x.post_build_hook,
            y.post_build_hook,
        )?,
        secret_key_files: merge_list(x.secret_key_files, y.secret_key_files),
    })
}

fn merge_list<T: Ord, U>(mut x: (Vec<T>, ()), mut y: (Vec<T>, U)) -> (Vec<T>, U) {
    x.0.append(&mut y.0);
    x.0.sort();
    x.0.dedup();
    (x.0, y.1)
}

fn merge_single_value<T, U>(
    field_name: &str,
    x: (Option<T>, ()),
    y: (Option<T>, U),
) -> Result<(Option<T>, U), MergeError>
where
    T: Eq,
{
    match (x.0, y.0) {
        (None, None) => Ok((None, y.1)),
        (Some(xv), None) => Ok((Some(xv), y.1)),
        (None, Some(yv)) => Ok((Some(yv), y.1)),
        (Some(xv), Some(yv)) => {
            if xv == yv {
                Ok((Some(xv), y.1))
            } else {
                Err(MergeError::MergeConflict {
                    reason: format!("Conflicting values set for {}", field_name),
                })
            }
        }
    }
}

fn merge_lines(mut old_lines: Vec<String>, new_lines: Vec<Line>) -> Vec<String> {
    for line in new_lines {
        match line.index {
            Some(index) => old_lines[index] = line.content,
            None => old_lines.push(line.content),
        }
    }
    old_lines
}

fn new_line_from_content(content: Option<String>, line: Option<Line>) -> Option<Line> {
    content.map(|content| {
        let mut line = line.unwrap_or_else(|| {
            let line = Line {
                content: String::new(),
                index: None,
            };
            line
        });
        line.content = content;
        line
    })
}

fn render_list(field_name: &str, conf_entry_ext: (Vec<String>, Option<Line>)) -> Option<Line> {
    let (values, line) = conf_entry_ext;
    let content = if values.is_empty() {
        None
    } else {
        Some(format!("{} = {}", field_name, values.join(" ")))
    };

    new_line_from_content(content, line)
}

fn render_single_value(
    field_name: &str,
    conf_entry_ext: (Option<String>, Option<Line>),
) -> Option<Line> {
    let (value, line) = conf_entry_ext;
    let content = value.map(|value| format!("{} = {}", field_name, value));

    new_line_from_content(content, line)
}

fn parse_field<T, F>(
    field_name: &str,
    index: usize,
    content: &str,
    parse_value: F,
    result: &mut (T, Option<Line>),
) where
    F: FnOnce(&str) -> T,
{
    if let Some(value_str) = content.strip_prefix(format!("{} = ", field_name).as_str()) {
        let value = parse_value(value_str);
        result.0 = value;
        result.1 = Some(Line {
            index: Some(index),
            content: content.to_string(),
        });
    }
}

fn parse_list(
    field_name: &str,
    index: usize,
    content: &str,
    result: &mut (Vec<String>, Option<Line>),
) {
    parse_field(
        field_name,
        index,
        content,
        |value_str| {
            value_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        },
        result,
    )
}

fn parse_single_value(
    field_name: &str,
    index: usize,
    content: &str,
    result: &mut (Option<String>, Option<Line>),
) {
    parse_field(
        field_name,
        index,
        content,
        |value_str| Some(value_str.to_string()),
        result,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        let old_content = r#"
experimental-features = nix-command flakes
build-users-group = nixbld
trusted-public-keys = cache.zelinf.net:poESahuRAXqYC2QPevSId+pTtoq0P1XfTxaSHRgfvVI=
post-build-hook = /etc/nix/post-build-hook
secret-key-files = /etc/nix/secret-key"#
            .as_bytes();
        let nix_conf = NixConf::new()
            .with_substituters(vec!["s3://nix-cache?endpoint=http://192.168.31.2:9091&profile=minio&compression=zstd&priority=0&trusted=true&want-mass-query=true".to_string()])
            .with_secret_key_files(vec!["a".to_string(), "b".to_string()]);
        let result = nix_conf
            .apply(old_content)
            .map(|x| x.map(|x| String::from_utf8(x).unwrap()));
        assert_eq!(
        result.unwrap().unwrap(),
                r#"
experimental-features = nix-command flakes
build-users-group = nixbld
trusted-public-keys = cache.zelinf.net:poESahuRAXqYC2QPevSId+pTtoq0P1XfTxaSHRgfvVI=
post-build-hook = /etc/nix/post-build-hook
secret-key-files = a b /etc/nix/secret-key
substituters = s3://nix-cache?endpoint=http://192.168.31.2:9091&profile=minio&compression=zstd&priority=0&trusted=true&want-mass-query=true"#
                    .to_string()
        )
    }
}
