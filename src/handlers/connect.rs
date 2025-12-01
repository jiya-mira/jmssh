use crate::app::AppContext;
use crate::cli::ConnectArgs;
use crate::error::{AppError, AppResult};
use crate::term::{c_accent, c_prefix};
use crate::usecase::{ConnectInput, connect};
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
        eprintln!("{} {}", c_prefix("[jmssh]"), "empty connect plan (no hops)");
        return Ok(());
    }

    // 3. 拼 ssh 参数（尽量不用 ssh_args 的 mut，按片段组合）
    // 3.1 jump 链：前 N-1 个 hop 作为 ProxyJump
    let proxy_args = if plan.hops.len() > 1 {
        let proxy_jump = plan.hops[..plan.hops.len() - 1]
            .into_iter()
            .map(|h| format!("{}@{}:{}", h.user, h.host, h.port))
            .collect_vec()
            .join(",");
        vec!["-J".to_string(), proxy_jump]
    } else {
        Vec::new()
    };

    // 3.2 最终目标
    let target = plan.hops.last().unwrap();

    // 端口参数
    let port_args = if target.port != 22 {
        vec!["-p".to_string(), target.port.to_string()]
    } else {
        Vec::new()
    };

    // key 参数（先只管最终目标的 key）
    let key_args = match &target.key_path_local {
        Some(key_path) => vec!["-i".to_string(), key_path.clone()],
        None => Vec::new(),
    };

    // user@host
    let dest_arg = format!("{}@{}", target.user, target.host);

    // 汇总成最终的 ssh_args（这里才需要一次 collect）
    let ssh_args = proxy_args
        .into_iter()
        .chain(port_args.into_iter())
        .chain(key_args.into_iter())
        .chain(std::iter::once(dest_arg))
        .collect_vec();

    // 4. 登入前 log（彩色）
    let prefix = c_prefix("[jmssh]");
    let target_desc = c_accent(&format!("{}@{}:{}", target.user, target.host, target.port));
    let via_desc = if plan.hops.len() > 1 {
        let chain = plan.hops[..plan.hops.len() - 1]
            .into_iter()
            .map(|h| format!("{}@{}", h.user, h.host))
            .collect_vec()
            .join(" -> ");
        format!(" via {}", c_accent(&chain))
    } else {
        String::new()
    };

    eprintln!("{prefix} connecting to {target_desc}{via_desc} ...");
    eprintln!("{prefix} exec {} {}", c_accent("ssh"), ssh_args.join(" "));

    // 5. 调用系统 ssh，继承当前终端 I/O
    let status = Command::new("ssh").args(&ssh_args).status()?;

    // 6. 退出 log（彩色）
    if status.success() {
        eprintln!("{} {}", prefix, c_accent("ssh session finished OK"));
    } else if let Some(code) = status.code() {
        eprintln!(
            "{} ssh exited with {}",
            prefix,
            c_accent(&format!("code {}", code))
        );
    } else {
        // 信号之类的情况
        eprintln!("{} ssh exited (terminated by signal)", prefix);
    }

    Ok(())
}

fn run_ssh_plain(args: &[String]) -> AppResult<ExitStatus> {
    Command::new("ssh")
        .args(args)
        .status()
        .map_err(|e| AppError::IoError(format!("failed to spawn ssh: {e}")))
}

fn run_ssh_with_password(args: &[String], password: Option<String>) -> AppResult<ExitStatus> {
    if let Some(password) = password {}
    run_ssh_plain(args)
}
