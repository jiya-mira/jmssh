use crate::entity::profiles::AuthMode;

mod connect;
mod profile;

#[derive(Debug, Clone)]
pub struct EditProfileInput {
    pub label: String,
    pub host: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub mode: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub jumps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProfileView {
    pub id: u32,
    pub label: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub mode: String,
    pub tags: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConnectInput {
    /// CLI 里的 target，比如 "origin"
    pub target: String,
    /// 可选：直接按 id 连接
    pub id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ConnectHop {
    pub id: u32,
    pub label: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub auth_mode: AuthMode,
    pub key_path_local: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConnectPlan {
    /// 从第一个 jump 到最终目标，按顺序排列
    pub hops: Vec<ConnectHop>,
}
