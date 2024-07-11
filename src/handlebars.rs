use anyhow::Result;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason,
};
use std::process::{Command, Stdio};

pub fn create_new_handlebars<'b>() -> Result<Handlebars<'b>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(str::to_string);
    handlebars.set_strict_mode(true);
    register_helpers(&mut handlebars);

    Ok(handlebars)
}
fn register_helpers(handlebars: &mut Handlebars<'_>) {
    handlebars_misc_helpers::register(handlebars);
    handlebars.register_helper("math", Box::new(math_helper));
    handlebars.register_helper("include_template", Box::new(include_template_helper));
    handlebars.register_helper("is_executable", Box::new(is_executable_helper));
    handlebars.register_helper("command_success", Box::new(command_success_helper));
    handlebars.register_helper("command_output", Box::new(command_output_helper));
}

fn math_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let params = h
        .params()
        .iter()
        .map(|p| p.render())
        .collect::<Vec<String>>();
    let expression = params.join(" ");

    out.write(
        &evalexpr::eval(&expression)
            .map_err(|e| {
                RenderErrorReason::Other(format!(
                    "Cannot evaluate expression {expression} because {e}"
                ))
            })?
            .to_string(),
    )?;
    Ok(())
}

fn include_template_helper(
    h: &Helper<'_>,
    handlebars: &Handlebars<'_>,
    ctx: &Context,
    rc: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let mut params = h.params().iter();
    let path = params
        .next()
        .ok_or(RenderErrorReason::ParamNotFoundForIndex(
            "include_template",
            0,
        ))?
        .render();
    if params.next().is_some() {
        return Err(RenderErrorReason::Other(
            "include_template: More than one parameter given".to_owned(),
        )
        .into());
    }

    let included_file =
        std::fs::read_to_string(path).map_err(|e| RenderErrorReason::NestedError(Box::new(e)))?;
    let rendered_file = handlebars
        .render_template_with_context(&included_file, rc.context().as_deref().unwrap_or(ctx))
        .map_err(|e| RenderErrorReason::NestedError(Box::new(e)))?;

    out.write(&rendered_file)?;

    Ok(())
}

fn is_executable_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let mut params = h.params().iter();
    let executable = params
        .next()
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("is_executable", 0))?
        .render();
    if params.next().is_some() {
        return Err(RenderErrorReason::Other(
            "is_executable: More than one parameter given".to_owned(),
        )
        .into());
    }

    let status =
        is_executable(&executable).map_err(|e| RenderErrorReason::NestedError(Box::new(e)))?;
    if status {
        out.write("true")?;
    }

    Ok(())
}

fn command_success_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let mut params = h.params().iter();
    let command = params
        .next()
        .ok_or(RenderErrorReason::ParamNotFoundForIndex(
            "command_success",
            0,
        ))?
        .render();
    if params.next().is_some() {
        return Err(RenderErrorReason::Other(
            "command_success: More than one parameter given".to_owned(),
        )
        .into());
    }

    let status = os_shell()
        .arg(&command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success();
    if status {
        out.write("true")?;
    }

    Ok(())
}

fn command_output_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let mut params = h.params().iter();
    let command = params
        .next()
        .ok_or(RenderErrorReason::ParamNotFoundForIndex(
            "command_success",
            0,
        ))?
        .render();
    if params.next().is_some() {
        return Err(RenderErrorReason::Other(
            "command_success: More than one parameter given".to_owned(),
        )
        .into());
    }

    let output = os_shell()
        .arg(&command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .output()?;
    out.write(&String::from_utf8_lossy(&output.stdout))?;

    Ok(())
}
fn is_executable(name: &str) -> Result<bool, std::io::Error> {
    Command::new("which")
        .arg(name)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
}

fn os_shell() -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c");
    cmd
}
