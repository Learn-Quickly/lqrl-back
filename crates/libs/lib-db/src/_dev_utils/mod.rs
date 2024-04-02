// region:    --- Modules

mod dev_db;

use tokio::sync::OnceCell;
use tracing::info;

use crate::store::DbManager;

// endregion: --- Modules

/// Initialize environment for local development.
/// (for early development, will be called from main()).
pub async fn init_dev() {
	static INIT: OnceCell<()> = OnceCell::const_new();

	INIT.get_or_init(|| async {
		info!("{:<12} - init_dev_all()", "FOR-DEV-ONLY");

		dev_db::init_dev_db().await.unwrap();
	})
	.await;
}

/// Initialize test environment.
pub async fn init_test() -> DbManager {
	static INIT: OnceCell<DbManager> = OnceCell::const_new();

	let dbm = INIT
		.get_or_init(|| async {
			init_dev().await;
			// NOTE: Rare occasion where unwrap is kind of ok.
			DbManager::new().await.unwrap()
		})
		.await;

	dbm.clone()
}
