use crate::app::AppContext;
use crate::cli::{EditProfileArgs, ProfileArgs, ProfileCommand, ProfileWithoutArgs};
use crate::error::{AppError, AppResult};
use crate::term::{c_accent, c_prefix};
use crate::usecase;
use rpassword::prompt_password;

pub async fn handle_profile(ctx: &AppContext, args: ProfileArgs) -> AppResult<()> {
    match args.cmd {
        ProfileCommand::Add(args) => profile_add(ctx, args).await?,
        ProfileCommand::Set(args) => profile_set(ctx, args).await?,
        _ => profile_unknown(ctx).await?,
    };

    Ok(())
}

async fn profile_add(ctx: &AppContext, args: EditProfileArgs) -> AppResult<()> {
    usecase::profile::add_profile(
        ctx,
        usecase::EditProfileInput {
            label: args.label,
            host: args.host,
            user: args.user,
            port: args.port,
            mode: args.mode,
            tags: args.tags,
            notes: args.note,
            jumps: args.jumps,
        },
    )
    .await?;

    Ok(())
}

async fn profile_set(ctx: &AppContext, args: EditProfileArgs) -> AppResult<()> {
    usecase::profile::set_profile(
        ctx,
        usecase::EditProfileInput {
            label: args.label,
            host: args.host,
            user: args.user,
            port: args.port,
            mode: args.mode,
            tags: args.tags,
            notes: args.note,
            jumps: args.jumps,
        },
    )
    .await?;

    Ok(())
}

pub async fn password_set(ctx: &AppContext, args: ProfileWithoutArgs) -> AppResult<()> {
    // 1. 读取密码（不回显）
    let prompt = format!(
        "{} {} {} ",
        c_prefix("[jmssh]"),
        "password for profile",
        c_accent(&args.label),
    );
    // 这里由于 rpassword 自己在 TTY 上输出 prompt，所以直接传纯文本也行，
    // 如果你想保持前缀样式，可以自己先 eprint 再用 `read_password_from_tty`，不过先简单用它自带 API 也可以：
    let pwd = prompt_password(prompt)
        .map_err(|e| AppError::IoError(format!("failed to read password: {e}")))?;

    // 2. 交给 usecase + PasswordStore
    usecase::profile::set_profile_password_by_label(ctx, args.label.clone(), Some(pwd)).await?;

    // 3. 打一行确认信息，突出 profile 名
    eprintln!(
        "{} {} {}",
        c_prefix("[jmssh]"),
        "Password stored in OS keyring for profile",
        c_accent(&args.label),
    );

    Ok(())
}

async fn profile_unknown(ctx: &AppContext) -> AppResult<()> {
    Ok(())
}
