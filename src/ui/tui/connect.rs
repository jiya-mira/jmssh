use crate::app::AppContext;
use crate::error::AppResult;
use crate::usecase;
use crate::usecase::ProfileView;
use console::{pad_str, truncate_str, Alignment};
use skim::options::SkimOptionsBuilder;
use skim::prelude::unbounded;
use skim::tuikit::attr::{Attr, Color};
use skim::tuikit::prelude::Effect;
use skim::{
    AnsiString, DisplayContext, ItemPreview, PreviewContext, Skim, SkimItem, SkimItemReceiver,
    SkimItemSender,
};
use std::borrow::Cow;
use std::cmp::min;
use std::sync::Arc;

pub async fn pick_profile_for_connect(ctx: &AppContext) -> AppResult<Option<ProfileView>> {
    let profiles = usecase::profile::list_profiles(ctx).await?;
    if profiles.is_empty() {
        return Ok(None);
    }

    let profiles_len = profiles.len();
    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    // 别用 for_each（会把 tx move 进闭包，导致 drop(tx) 不能用）
    for p in profiles {
        let _ = tx.send(Arc::new(ProfileSkimItem { p }));
    }
    drop(tx);

    let options = SkimOptionsBuilder::default()
        .height(min(profiles_len, 12).to_string()) // 10~15 行都行；这很像 fzf
        .prompt("connect> ".to_string())
        .multi(false)
        .ansi(true)
        // 保险：显式把 ctrl-c 绑定为 abort（如果你这版 skim builder 没这个方法就删掉，仅靠 is_abort 也行）
        .bind(vec!["ctrl-c:abort".to_string(), "esc:abort".to_string()])
        .build()
        .unwrap();

    let out = Skim::run_with(&options, Some(rx));

    // 关键：abort 直接返回 None，别“误选”
    let out = match out {
        None => return Ok(None),
        Some(o) if o.is_abort => return Ok(None),
        Some(o) => o,
    };

    let picked = out
        .selected_items
        .first()
        .and_then(|it| it.as_any().downcast_ref::<ProfileSkimItem>())
        .map(|x| x.p.clone()); // 需要 ProfileView: Clone（不想 Clone 我下面给替代方案）

    Ok(picked)
}

struct ProfileSkimItem {
    p: ProfileView,
}

fn fit_col(s: &str, width: usize) -> String {
    // 先截断，再右侧补空格到固定宽度（保证列对齐）
    let t = truncate_str(s, width, "...");
    pad_str(&t, width, Alignment::Left, None).to_string()
}

impl ProfileSkimItem {
    const W_LABEL: usize = 22;
    const W_DEST: usize = 30;
    const W_MODE: usize = 10;
    const SEP: &'static str = "  ";

    fn cols(&self) -> (String, String, String) {
        let label = fit_col(&self.p.label, Self::W_LABEL);
        let dest_raw = format!("{}@{}:{}", self.p.user, self.p.host, self.p.port);
        let dest = fit_col(&dest_raw, Self::W_DEST);
        let mode = fit_col(&self.p.mode, Self::W_MODE);
        (label, dest, mode)
    }

    /// 纯文本行：用于搜索(text)；也作为 display 的“底稿”
    fn line(&self) -> String {
        let (label, dest, mode) = self.cols();
        format!("{label}{}{dest}{}{mode}", Self::SEP, Self::SEP)
    }

    /// 用 skim 的 fragments 做染色：不会产生 \x1b[..m，也就不会显示 ?[36m
    fn colored_line(&self) -> AnsiString<'static> {
        let (label, dest, mode) = self.cols();
        let sep = Self::SEP;

        let stripped = format!("{label}{sep}{dest}{sep}{mode}");

        // fragments 用的是“字符索引”(char index)，不是 byte index
        let label_len = label.chars().count() as u32;
        let sep_len = sep.chars().count() as u32;
        let dest_len = dest.chars().count() as u32;
        let mode_len = mode.chars().count() as u32;

        let dest_start = label_len + sep_len;
        let dest_end = dest_start + dest_len;

        let mode_start = dest_end + sep_len;
        let mode_end = mode_start + mode_len;

        // 颜色方案（你可以再微调）：label 青色加粗、dest 白色、mode 黄色
        let a_label = Attr {
            fg: Color::AnsiValue(6),
            effect: Effect::BOLD,
            ..Attr::default()
        };

        let a_dest = Attr {
            fg: Color::AnsiValue(7),
            ..Attr::default()
        };

        let a_mode = Attr {
            fg: Color::AnsiValue(3),
            ..Attr::default()
        };

        let fragments = vec![
            (a_label, (0, label_len)),
            (a_dest, (dest_start, dest_end)),
            (a_mode, (mode_start, mode_end)),
        ];

        AnsiString::new_string(stripped, fragments)
    }
}

impl SkimItem for ProfileSkimItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Owned(self.line())
    }

    fn display(&self, _context: DisplayContext) -> AnsiString<'_> {
        self.colored_line()
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        // 不启用 preview_window 时，这里无意义；给个空即可
        ItemPreview::Text(String::new())
    }

    fn output(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.p.label)
    }
}
