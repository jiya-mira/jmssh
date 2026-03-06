use crate::app::AppContext;
use crate::cli::{PasswordArgs, PasswordCommand, PasswordLabelArgs};
use crate::error::{AppError, AppResult};
use crate::term::{c_accent, c_error, c_warning, log_error, log_info, log_warn};
use crate::usecase;

pub async fn handle_password(ctx: &AppContext, args: PasswordArgs) -> AppResult<()> {
    match args.cmd {
        PasswordCommand::Set(label_args) => handle_password_set(ctx, label_args).await,
        PasswordCommand::Show(label_args) => handle_password_show(ctx, label_args).await,
        PasswordCommand::Clear(label_args) => handle_password_clear(ctx, label_args).await,
    }
}

async fn handle_password_set(ctx: &AppContext, args: PasswordLabelArgs) -> AppResult<()> {
    let profile = usecase::profile::get_profile_by_label(ctx, args.label.clone()).await?;

    let profile_info = format_args!(
        "{label}({user}@{host}:{port})",
        label = args.label.clone(),
        user = profile.user,
        host = profile.host,
        port = profile.port,
    );

    let prompt = format!("password for {}: ", profile_info);
    let pwd = rpassword::prompt_password(prompt)
        .map_err(|e| AppError::IoError(format!("failed to read password: {e}")))?;

    usecase::password::set_profile_password_by_label(ctx, args.label, Some(pwd)).await?;

    log_info(c_accent(
        format!(
            "{} password stored in OS keyring",
            c_warning(profile_info.to_string().as_str())
        )
        .as_str(),
    ));

    Ok(())
}

async fn handle_password_show(ctx: &AppContext, args: PasswordLabelArgs) -> AppResult<()> {
    let pwd = usecase::password::get_profile_password_by_label(ctx, args.label.clone()).await?;
    match pwd {
        Some(p) => {
            log_warn(format!(
                "showing password for profile {} (plaintext below)",
                c_accent(&args.label),
            ));

            // stdout：直接输出密码，本身用醒目颜色
            // 终端里一眼能看出是敏感内容；管道场景下只是多了 ANSI，但功能不受影响
            println!("{}", c_error(&p));
        }
        None => {
            log_error(format!(
                "no password stored for profile {}",
                c_accent(&args.label),
            ));
        }
    }

    Ok(())
}

async fn handle_password_clear(ctx: &AppContext, args: PasswordLabelArgs) -> AppResult<()> {
    usecase::password::clear_profile_password_by_label(ctx, args.label.clone()).await?;
    log_info(format!(
        "password cleared for profile {}",
        c_accent(&args.label),
    ));
    Ok(())
}
