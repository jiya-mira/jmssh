use crate::app::AppContext;
use crate::cli::{PasswordArgs, PasswordCommand, PasswordLabelArgs};
use crate::error::{AppError, AppResult};
use crate::term::c_prefix;
use crate::usecase;

pub async fn handle_password(ctx: &AppContext, args: PasswordArgs) -> AppResult<()> {
    match args.cmd {
        PasswordCommand::Set(label_args) => handle_password_set(ctx, label_args).await,
        PasswordCommand::Show(label_args) => handle_password_show(ctx, label_args).await,
        PasswordCommand::Clear(label_args) => handle_password_clear(ctx, label_args).await,
    }
}

async fn handle_password_set(ctx: &AppContext, args: PasswordLabelArgs) -> AppResult<()> {
    let prompt = format!("password for '{}': ", args.label);
    let pwd = rpassword::prompt_password(prompt)
        .map_err(|e| AppError::IoError(format!("failed to read password: {e}")))?;

    usecase::password::set_profile_password_by_label(ctx, args.label, Some(pwd)).await?;
    eprintln!("{}", c_prefix("[jmssh] password stored in OS keyring"));
    Ok(())
}

async fn handle_password_show(ctx: &AppContext, args: PasswordLabelArgs) -> AppResult<()> {
    let pwd = usecase::password::get_profile_password_by_label(ctx, args.label.clone()).await?;
    match pwd {
        Some(p) => {
            println!("{p}")
        }
        None => {
            eprintln!(
                "{} {}",
                c_prefix("[jmssh] no password stored for profile"),
                args.label,
            );
        }
    }

    Ok(())
}

async fn handle_password_clear(ctx: &AppContext, args: PasswordLabelArgs) -> AppResult<()> {
    usecase::password::clear_profile_password_by_label(ctx, args.label.clone()).await?;
    eprintln!(
        "{} {}",
        c_prefix("[jmssh] password cleared for profile"),
        args.label
    );
    Ok(())
}
