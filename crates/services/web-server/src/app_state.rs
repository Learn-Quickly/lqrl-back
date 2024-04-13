use std::sync::Arc;

use lib_core::{interactors::permission_manager::PermissionManager, interfaces::command_repository_manager::ICommandRepositoryManager};
use lib_db::store::{command_repository_manager::CommandRepositoryManager, error::DbError, query_repository_manager::QueryRepositoryManager};

#[derive(Clone)]
pub struct AppState {
    pub query_repository_manager: Arc<QueryRepositoryManager>,
    pub command_repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
    pub permission_manager: Arc<PermissionManager>,
}

impl AppState {
    pub async fn new() -> Result<Self, DbError> {
        let query_repository_manager = Arc::new(QueryRepositoryManager::new().await?);
        let command_repository_manager = Arc::new(CommandRepositoryManager::new().await?);
        let permission_manager = Arc::new(PermissionManager::new(command_repository_manager.clone()));

        let result = Self {
            query_repository_manager,
            command_repository_manager,
            permission_manager,
        };

        Ok(result)
    }
}