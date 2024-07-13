use crate::config::Variables;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use log::{debug, info, trace};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Command};

#[macro_export]
macro_rules! cwd {
    () => {{
        use anyhow::Context;
        std::env::current_dir().context("current working directory")?
    }};
}

pub trait Hook {
    fn run(location: &Path, handlebars: &Handlebars<'_>, variables: &Variables) -> Result<()> {
        if !location.exists() {
            debug!("No hook at {:?}", location);
            return Ok(());
        }
        info!("Running hook at {:?}", location);

        let script_location = cwd!().join(location);
        render_template(&script_location, handlebars, variables)?;
        let script_location = script_location.with_extension("templated");
        let mut child = run_script_file(&script_location)?;

        anyhow::ensure!(
            child.wait().context("wait for child shell")?.success(),
            "subshell returned error"
        );

        Ok(())
    }
}

pub struct Pre;
pub struct Post;

impl Hook for Pre {}
impl Hook for Post {}

fn run_script_file(script: &Path) -> Result<Child> {
    let permissions = script.metadata()?.permissions();
    if !script.is_dir() && permissions.mode() & 0o111 != 0 {
        Command::new(script).spawn().context("spawn script file")
    } else {
        Command::new("sh")
            .arg(script)
            .spawn()
            .context("spawn shell")
    }
}

fn render_template(
    source: &Path,
    handlebars: &Handlebars<'_>,
    variables: &Variables,
) -> Result<()> {
    let file_contents = std::fs::read_to_string(source).context("read template source file")?;
    let rendered = handlebars
        .render_template(&file_contents, variables)
        .context("render template")?;

    let templated_source = source.with_extension("templated");
    fs::write(templated_source, rendered)?;

    Ok(())
}

pub fn remove_templated_scripts() -> Result<()> {
    let templated = fs::read_dir(cwd!())?
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            path.extension().map_or(false, |ext| ext == "templated")
        });
    for entry in templated {
        trace!("removing templated script: {:?}", entry.path());
        fs::remove_file(entry.path())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Variables;
    use handlebars::Handlebars;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn should_run_hook() -> Result<()> {
        let dir = TempDir::new("hook")?;

        let script = dir.path().join("script.sh");
        File::create(&script)?.write_all(b"echo 'Hello, world!'")?;

        let handlebars = Handlebars::new();
        let variables = Variables::new();

        Pre::run(&script, &handlebars, &variables)?;

        Ok(())
    }

    #[test]
    fn should_remove_templated_scripts() -> Result<()> {
        let dir = TempDir::new("hook")?;
        // override current dir
        std::env::set_current_dir(dir.path())?;

        let script = dir.path().join("script.sh");
        File::create(&script)?.write_all(b"echo 'Hello, {{name}}!'")?;

        let templated = dir.path().join("script.templated");
        let variables = vec![("name".to_string(), "world".to_string())]
            .into_iter()
            .collect::<Variables>();

        assert!(!templated.exists());

        Pre::run(&script, &Handlebars::new(), &variables)?;

        assert!(templated.exists());

        remove_templated_scripts()?;

        assert!(!templated.exists());

        Ok(())
    }

    #[test]
    fn should_render_script_template() -> Result<()> {
        let dir = TempDir::new("hook")?;

        let script = dir.path().join("script.sh");
        File::create(&script)?.write_all(b"echo 'Hello, {{name}}!'")?;

        let desired_templated_script = dir.path().join("script.templated");
        let variables = vec![("name".to_string(), "world".to_string())]
            .into_iter()
            .collect::<Variables>();

        assert!(!desired_templated_script.exists());

        render_template(&script, &Handlebars::new(), &variables)?;

        assert!(desired_templated_script.exists());
        let templated_contents = fs::read_to_string(&desired_templated_script)?;
        assert_eq!(templated_contents, "echo 'Hello, world!'");

        Ok(())
    }
}
