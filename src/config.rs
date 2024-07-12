use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
};

pub type Files = HashMap<PathBuf, PathBuf>;
pub type Variables = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
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
pub struct Config {
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
    let config: Config = load_file(config_path)
        .and_then(|c| c.ok_or_else(|| anyhow::anyhow!("config.toml not found")))?;

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

    Ok(Configuration {
        packages,
        variables,
    })
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
        .map(|(k, v)| -> Result<_, anyhow::Error> { Ok((k, expand_path(&v)?)) })
        .collect()
}

fn merge_variables(
    variables: impl Iterator<Item = (String, String)>,
    package_variables: impl Iterator<Item = (String, String)>,
) -> Variables {
    variables.into_iter().chain(package_variables).collect()
}
