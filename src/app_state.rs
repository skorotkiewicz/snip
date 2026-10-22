use crate::database::Database;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: Database,
}
