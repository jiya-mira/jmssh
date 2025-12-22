use crate::app::AppContext;
use crate::error::AppResult;
use crate::usecase;
use itertools::Itertools;
use skim::options::SkimOptionsBuilder;
use skim::prelude::SkimItemReader;
use skim::Skim;
use std::io::Cursor;

pub async fn pick_profile_label_for_connect(ctx: &AppContext) -> AppResult<Option<String>> {
    let profiles = usecase::profile::list_profiles(ctx).await?;
    if profiles.is_empty() {
        return Ok(None);
    }

    // 每行：label \t user@host:port \t mode（方便搜索 + 看一眼信息）
    let input = profiles
        .iter()
        .map(|p| format!("{}\t{}@{}:{}\t{}", p.label, p.user, p.host, p.port, p.mode))
        .join("\n");

    let options = SkimOptionsBuilder::default()
        .height("60%".to_string())
        .prompt("connect> ".to_string())
        .multi(false)
        .build()
        .unwrap();

    let items = SkimItemReader::default().of_bufread(Cursor::new(input));
    let out = Skim::run_with(&options, Some(items));

    let picked = out
        .and_then(|o| o.selected_items.get(0).cloned())
        .map(|it| it.output().to_string());

    // picked 是整行 output（包含 tab），取第一列 label
    Ok(picked
        .and_then(|line| line.split('\t').next().map(|s| s.to_string()))
        .filter(|s| !s.is_empty()))
}
