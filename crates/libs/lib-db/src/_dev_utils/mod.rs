// region:    --- Modules

mod dev_db;

use lib_core::ctx::Ctx;
use modql::filter::OpValString;
use tokio::sync::OnceCell;
use tracing::info;

use crate::{command_repository, store::{error::DbResult, DbManager}};

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

// region:    --- User seed/clean

pub async fn seed_users(
	ctx: &Ctx,
	dbm: &DbManager,
	usernames: &[&str],
) -> DbResult<Vec<i64>> {
	let mut ids = Vec::new();

	for name in usernames {
		let id = seed_user(ctx, dbm, name).await?;
		ids.push(id);
	}

	Ok(ids)
}

pub async fn seed_user(
	ctx: &Ctx,
	dbm: &DbManager,
	username: &str,
) -> DbResult<i64> {
	let pwd_clear = "seed-user-pwd";

	let id = command_repository::user::UserBmc::create(
		ctx,
		dbm,
		command_repository::user::UserForCreate {
			username: username.to_string(),
			pwd_clear: pwd_clear.to_string(),
		},
	)
	.await?;

	Ok(id)
}

pub async fn clean_users(
	ctx: &Ctx,
	dbm: &DbManager,
	contains_username: &str,
) -> DbResult<usize> {
	let users = command_repository::user::UserBmc::list(
		ctx,
		dbm,
		Some(vec![command_repository::user::UserFilter {
			username: Some(
				OpValString::Contains(contains_username.to_string()).into(),
			),
			..Default::default()
		}]),
		None,
	)
	.await?;
	let count = users.len();

	for user in users {
		command_repository::user::UserBmc::delete(ctx, dbm, user.id).await?;
	}

	Ok(count)
}

// endregion: --- User seed/clean