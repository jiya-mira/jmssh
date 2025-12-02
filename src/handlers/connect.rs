use crate::app::AppContext;
use crate::cli::ConnectArgs;
use crate::entity::profiles::AuthMode;
use crate::error::AppResult;
use crate::term::{c_accent, log_error, log_info};
use crate::usecase::{connect, ConnectInput};
use itertools::Itertools;
use std::process::{Command, ExitStatus};

pub async fn handle_connect(ctx: &AppContext, args: ConnectArgs) -> AppResult<()> {
    // 1. CLI -> usecase 输入
    let input = ConnectInput {
        target: args.target,
        id: args.id,
    };

    // 2. 计算连接计划（含跳板链）
    let plan = connect::build_connect_plan(ctx, input).await?;

    if plan.hops.is_empty() {
        log_error(c_accent("empty connect plan (no hops)"));
        return Ok(());
    }

    let password_profile_id = match plan.hops.as_slice() {
        [] => None,
        [single] => (single.auth_mode == AuthMode::Password).then_some(single.id),
        [first, ..] => (first.auth_mode == AuthMode::Password).then_some(first.id),
    };

    let password_opt = password_profile_id
        .map(|id| ctx.password_store.get_profile_password(id))
        .transpose()?
        .flatten();

    // 3. 拼 ssh 参数（尽量不用 ssh_args 的 mut，按片段组合）
    // 3.1 jump 链：前 N-1 个 hop 作为 ProxyJump
    let proxy_args = (plan.hops.len() > 1)
        .then(|| {
            vec![
                "-J".to_string(),
                plan.hops[..plan.hops.len() - 1]
                    .iter()
                    .map(|h| format!("{}@{}:{}", h.user, h.host, h.port))
                    .collect_vec()
                    .join(","),
            ]
        })
        .unwrap_or_default();

    // 3.2 最终目标
    let target = plan.hops.last().unwrap();

    // 端口参数
    let port_args = (target.port != 22)
        .then(|| vec!["-p".to_string(), target.port.to_string()])
        .unwrap_or_default();

    // key 参数（先只管最终目标的 key）
    let key_args = matches!(
        (&target.auth_mode, &target.key_path_local),
        (AuthMode::Key, Some(_))
    )
    .then(|| {
        vec![
            "-i".to_string(),
            // safe unwrap: 上面 match 已经保证是 Some
            target.key_path_local.clone().unwrap(),
        ]
    })
    .unwrap_or_default();

    // user@host
    let dest_arg = vec![target.user.clone(), target.host.clone()].join("@");

    // 汇总成最终的 ssh_args（这里才需要一次 collect）
    let ssh_args = proxy_args
        .into_iter()
        .chain(port_args)
        .chain(key_args)
        .chain(std::iter::once(dest_arg))
        .collect_vec();

    // 4. 登入前 log（彩色）
    let prefix_target = c_accent(&format!("{}@{}:{}", target.user, target.host, target.port));
    let via_desc = (plan.hops.len() > 1)
        .then(|| {
            let chain = plan.hops[..plan.hops.len() - 1]
                .iter()
                .map(|h| format!("{}@{}", h.user, h.host))
                .collect_vec()
                .join(" -> ");
            format!(" via {}", c_accent(&chain))
        })
        .unwrap_or_default();

    if let Some(pid) = password_profile_id {
        let label_for_log = plan
            .hops
            .iter()
            .find(|h| h.id == pid)
            .map(|h| h.label.clone())
            .unwrap_or_else(|| format!("#{pid}"));

        match password_opt.as_ref() {
            Some(_) => log_info(format!(
                "using {} for profile {}",
                c_accent("stored password"),
                c_accent(&label_for_log),
            )),
            None => log_error(format!(
                "{}={} but {}",
                c_accent("auth_mode"),
                c_accent("password"),
                c_accent("no password stored in keyring"),
            )),
        }
    }

    log_info(format!("connecting to {prefix_target}{via_desc} ..."));
    log_info(format!("exec {} {}", c_accent("ssh"), ssh_args.join(" ")));

    // 5. 调用系统 ssh，继承当前终端 I/O
    let status = run_ssh_with_password(&ssh_args, password_opt.as_deref())?;

    // 6. 退出 log（彩色）
    if status.success() {
        log_info(c_accent("ssh session finished OK"));
    } else if let Some(code) = status.code() {
        log_error(format!(
            "{} {}",
            c_accent("ssh exited with"),
            c_accent(&format!("code {}", code)),
        ));
    } else {
        // 信号之类的情况
        log_error(c_accent("ssh exited (terminated by signal)"));
    }

    Ok(())
}

#[cfg(unix)]
fn run_ssh_with_password(args: &[String], password: Option<&str>) -> AppResult<ExitStatus> {
    fn plain_ssh(args: &[String]) -> AppResult<ExitStatus> {
        Ok(Command::new("ssh").args(args).status()?)
    }

    let pwd = match password {
        None => return plain_ssh(args),
        Some(p) => p,
    };

    // 有密码：优先尝试 sshpass
    match Command::new("sshpass")
        .arg("-p")
        .arg(pwd)
        .arg("ssh")
        .args(args)
        .status()
    {
        Ok(status) => {
            log_info(format!(
                "{} {}",
                c_accent("sshpass finished with status"),
                c_accent(&format!("{status}")),
            ));
            Ok(status)
        }

        // sshpass 不存在：打一行提示，然后 fallback 到普通 ssh
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            log_error(format!(
                "{} `{}` not found, falling back to plain ssh (you'll need to type password manually)",
                c_accent("sshpass"),
                c_accent("sshpass"),
            ));
            plain_ssh(args)
        }

        // 其他 IO 错误：上抛
        Err(e) => Err(e.into()),
    }
}

#[cfg(windows)]
fn run_ssh_with_password(args: &[String], password: Option<&str>) -> AppResult<ExitStatus> {
    let status = Command::new("ssh").args(args).status()?;
    Ok(status)
}
