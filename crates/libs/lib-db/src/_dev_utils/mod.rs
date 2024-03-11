// region:    --- Modules

mod dev_db;

use crate::model::{self, ModelManager};
use lib_core::ctx::Ctx;
use modql::filter::OpValString;
use tokio::sync::OnceCell;
use tracing::info;

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
pub async fn init_test() -> ModelManager {
	static INIT: OnceCell<ModelManager> = OnceCell::const_new();

	let mm = INIT
		.get_or_init(|| async {
			init_dev().await;
			// NOTE: Rare occasion where unwrap is kind of ok.
			ModelManager::new().await.unwrap()
		})
		.await;

	mm.clone()
}

// region:    --- User seed/clean

pub async fn seed_users(
	ctx: &Ctx,
	mm: &ModelManager,
	usernames: &[&str],
) -> model::Result<Vec<i64>> {
	let mut ids = Vec::new();

	for name in usernames {
		let id = seed_user(ctx, mm, name).await?;
		ids.push(id);
	}

	Ok(ids)
}

pub async fn seed_user(
	ctx: &Ctx,
	mm: &ModelManager,
	username: &str,
) -> model::Result<i64> {
	let pwd_clear = "seed-user-pwd";

	let id = model::user::UserBmc::create(
		ctx,
		mm,
		model::user::UserForCreate {
			username: username.to_string(),
			pwd_clear: pwd_clear.to_string(),
		},
	)
	.await?;

	Ok(id)
}

pub async fn clean_users(
	ctx: &Ctx,
	mm: &ModelManager,
	contains_username: &str,
) -> model::Result<usize> {
	let users = model::user::UserBmc::list(
		ctx,
		mm,
		Some(vec![model::user::UserFilter {
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
		model::user::UserBmc::delete(ctx, mm, user.id).await?;
	}

	Ok(count)
}

// endregion: --- User seed/clean