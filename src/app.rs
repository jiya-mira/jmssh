use sea_orm::DatabaseConnection;

pub struct AppContext {
    pub db: DatabaseConnection,
}

impl AppContext {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
