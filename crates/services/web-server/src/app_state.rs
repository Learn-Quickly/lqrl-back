use std::sync::Arc;

use lib_core::interfaces::command_repository_manager::ICommandRepositoryManager;
use lib_db::store::{command_repository_manager::CommandRepositoryManager, error::DbError, query_repository_manager::QueryRepositoryManager};

#[derive(Clone)]
pub struct AppState {
    pub query_repository_manager: QueryRepositoryManager,
    pub command_repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl AppState {
    pub async fn new() -> Result<Self, DbError> {
        let query_repository_manager = QueryRepositoryManager::new().await?;
        let command_repository_manager =  CommandRepositoryManager::new().await?;

        let result = Self {
            query_repository_manager,
            command_repository_manager: Arc::new(command_repository_manager),
        };

        Ok(result)
    }
}