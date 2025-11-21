use anyhow::{Context, Result};
use log::trace;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct TargetSpec {
    pub to: PathBuf,
    pub is_symlink: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum FileTarget {
    Simple(PathBuf),
    WithSpec(TargetSpec),
}

pub type Files = HashMap<PathBuf, FileTarget>;
pub type Variables = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Package {
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub files: Files,
    #[serde(default)]
    pub variables: Variables,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InnerConfig {
    #[serde(flatten)]
    packages: HashMap<String, Package>,
    #[serde(default)]
    variables: Variables,
}

#[derive(Debug)]
pub struct Configuration {
    pub packages: HashMap<String, Package>,
    pub variables: Variables,
}

impl Configuration {
    pub fn ordered_by_dependencies(&self) -> Vec<(String, Package)> {
        let mut packages = self.packages.clone();
        let mut ordered = Vec::new();

        while !packages.is_empty() {
            let mut next = None;
            for (name, package) in &packages {
                if package
                    .depends
                    .iter()
                    .all(|dep| ordered.iter().any(|(n, _)| n == dep))
                {
                    next = Some((name.to_owned(), package.to_owned()));
                    break;
                }
            }
            let (name, package) = next.expect("circular dependency");
            packages.remove(&name);
            ordered.push((name, package));
        }

        ordered
    }
}

pub fn load_config(config_path: &Path) -> Result<Configuration> {
    let config: InnerConfig = load_file(config_path)
        .and_then(|c| c.ok_or_else(|| anyhow::anyhow!("config.yaml not found")))?;

    // expand paths
    let packages = config
        .packages
        .into_iter()
        .map(|(name, mut package)| -> Result<_, anyhow::Error> {
            package.files = expand_paths(package.files)?;
            Ok((name, package))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    // merge variables
    let package_variables = packages
        .values()
        .fold(HashMap::new(), |mut acc, p| {
            acc.extend(p.variables.to_owned());
            acc
        })
        .into_iter();

    let variables = merge_variables(config.variables.into_iter(), package_variables);

    trace!("variables: {:?}", variables);
    trace!("packages: {:?}", packages);

    let effective_config = Configuration {
        packages,
        variables,
    };

    Ok(effective_config)
}

pub fn load_file<T>(filename: &Path) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let mut buf = String::new();
    let mut f = match File::open(filename) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        e => e,
    }
    .context("open file")?;
    f.read_to_string(&mut buf).context("read file")?;
    let data = serde_yaml::from_str::<T>(&buf).context("deserialize file contents")?;
    Ok(Some(data))
}

fn expand_path(path: &Path) -> Result<PathBuf> {
    let expanded = shellexpand::full(&path.to_string_lossy())?.to_string();

    Ok(PathBuf::from(expanded))
}

fn expand_paths(files: Files) -> Result<Files> {
    files
        .into_iter()
        .map(|(k, v)| -> Result<_, anyhow::Error> {
            let updated_v = match v {
                FileTarget::Simple(path) => FileTarget::Simple(expand_path(&path)?),
                FileTarget::WithSpec(target) => {
                    let expanded_to = expand_path(&target.to)?;
                    FileTarget::WithSpec(TargetSpec {
                        to: expanded_to,
                        is_symlink: target.is_symlink,
                    })
                }
            };

            Ok((k, updated_v))
        })
        .collect()
}

fn merge_variables(
    variables: impl Iterator<Item = (String, String)>,
    package_variables: impl Iterator<Item = (String, String)>,
) -> Variables {
    variables.into_iter().chain(package_variables).collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::File, io::Write};
    use tempdir::TempDir;

    #[test]
    fn should_merge_variables() {
        let variables = vec![("a".to_string(), "1".to_string())]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let package_variables = vec![("b".to_string(), "2".to_string())]
            .into_iter()
            .collect::<HashMap<_, _>>();

        let merged = super::merge_variables(variables.into_iter(), package_variables.into_iter());

        let expected = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        assert_eq!(merged, expected);
    }

    #[test]
    fn should_load_config() -> anyhow::Result<()> {
        let config_content = r#"
        variables:
            a: "1"
            b: "2"
        
        shell:
            files:
                .bashrc: .bashrc

        "#
        .to_string();

        let dir = TempDir::new("config")?;
        let config_path = dir.path().join("config.toml");
        let mut config = File::create(&config_path)?;
        config.write(config_content.as_bytes())?;

        let config = super::load_config(&config_path).unwrap();

        let expected = super::Configuration {
            packages: vec![(
                "shell".to_string(),
                super::Package {
                    depends: vec![],
                    files: vec![(
                        ".bashrc".into(),
                        super::FileTarget::Simple(".bashrc".into()),
                    )]
                    .into_iter()
                    .collect(),
                    variables: HashMap::new(),
                },
            )]
            .into_iter()
            .collect(),
            variables: vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        assert_eq!(config.variables, expected.variables);
        assert_eq!(config.packages, expected.packages);

        Ok(())
    }
}
