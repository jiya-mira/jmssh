use crate::entity::{local_auth, profiles, routes};
use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema};
use std::fs;
use std::path::PathBuf;

pub fn db_path() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "jiyamira", "jmssh")
        .ok_or_else(|| anyhow!("Could not find jmssh directory"))?;
    let data_dir = proj.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("jmssh.sqlite"))
}

pub async fn connect_db() -> Result<DatabaseConnection> {
    let path = db_path()?;
    let path_str = path.to_string_lossy();
    let url = format!("sqlite:{}?mode=rwc", path_str);
    let db = Database::connect(&url).await?;
    Ok(db)
}

pub async fn init_schema(db: &DatabaseConnection) -> Result<()> {
    let backend = DbBackend::Sqlite;
    let schema = Schema::new(backend);
    {
        let mut stmt = schema.create_table_from_entity(profiles::Entity);
        stmt.if_not_exists();
        db.execute(backend.build(&stmt)).await?;
    }

    {
        let mut stmt = schema.create_table_from_entity(routes::Entity);
        stmt.if_not_exists();
        db.execute(backend.build(&stmt)).await?;
    }

    {
        let mut stmt = schema.create_table_from_entity(local_auth::Entity);
        stmt.if_not_exists();
        db.execute(backend.build(&stmt)).await?;
    }

    Ok(())
}
