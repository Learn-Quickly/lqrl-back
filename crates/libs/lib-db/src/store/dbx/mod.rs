pub(crate) mod error;

use sea_query_binder::SqlxValues;
use sqlx::postgres::any::AnyConnectionBackend;
use sqlx::query::{Query, QueryAs, QueryScalar};
use sqlx::{FromRow, IntoArguments, Pool, Postgres, Transaction};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex;

use self::error::{DbxResult, DbxError};

use super::db_manager::Db;

#[derive(Debug, Clone)]
pub struct Dbx {
	db_pool: Db,
	txn_holder: Arc<Mutex<Option<TxnHolder>>>,
	with_txn: bool,
}

impl Dbx {
	pub fn new(db_pool: Db, with_txn: bool) -> DbxResult<Self> {
		Ok(Dbx {
			db_pool,
			txn_holder: Arc::default(),
			with_txn,
		})
	}
}

#[derive(Debug)]
struct TxnHolder {
	txn: Transaction<'static, Postgres>,
	counter: i32,
}

impl TxnHolder {
	fn new(txn: Transaction<'static, Postgres>) -> Self {
		TxnHolder { txn, counter: 1 }
	}

	fn inc(&mut self) {
		self.counter += 1;
	}

	fn dec(&mut self) -> i32 {
		self.counter -= 1;
		self.counter
	}
}

impl Deref for TxnHolder {
	type Target = Transaction<'static, Postgres>;

	fn deref(&self) -> &Self::Target {
		&self.txn
	}
}

impl DerefMut for TxnHolder {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.txn
	}
}

impl Dbx {
	pub async fn begin_txn(&self) -> DbxResult<()> {
		if !self.with_txn {
			return Err(DbxError::CannotBeginTxnWithTxnFalse);
		}

		let mut txh_g = self.txn_holder.lock().await;
		// If we already have a tx holder, then, we increment
		if let Some(txh) = txh_g.as_mut() {
			txh.inc();
		}
		// If not, we create one with a new transaction
		else {
			let transaction = self.db_pool.begin().await?;
			let _ = txh_g.insert(TxnHolder::new(transaction));
		}

		Ok(())
	}

	pub async fn commit_txn(&self) -> DbxResult<()> {
		if !self.with_txn {
			return Err(DbxError::CannotCommitTxnWithTxnFalse);
		}

		let mut txh_g = self.txn_holder.lock().await;
		if let Some(txh) = txh_g.as_mut() {
			let counter = txh.dec();
			// If 0, then, it should be matching commit for the first first begin_txn
			// so we can commit.
			if counter == 0 {
				// here we take the txh out of the option
				if let Some(mut txn) = txh_g.take() {
					txn.txn.as_mut().commit().await?;
				} // TODO: Might want to add a warning on the else.
			} // TODO: Might want to add a warning on the else.

			Ok(())
		}
		// Ohterwise, we have an error
		else {
			Err(DbxError::TxnCantCommitNoOpenTxn)
		}
	}

	pub fn db(&self) -> &Pool<Postgres> {
		&self.db_pool
	}

	pub async fn fetch_one_scalar<'q>(
		&self,
		query: QueryScalar<'q, Postgres, i64, SqlxValues>,
	) -> DbxResult<i64>{
		let data = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.fetch_one(txn.as_mut()).await?
			} else {
				query.fetch_one(self.db()).await?
			}
		} else {
			query.fetch_one(self.db()).await?
		};

		Ok(data)
	}

	pub async fn fetch_one<'q, O, A>(
		&self,
		query: QueryAs<'q, Postgres, O, A>,
	) -> DbxResult<O>
	where
		O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let data = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.fetch_one(txn.as_mut()).await?
			} else {
				query.fetch_one(self.db()).await?
			}
		} else {
			query.fetch_one(self.db()).await?
		};

		Ok(data)
	}

	pub async fn fetch_optional<'q, O, A>(
		&self,
		query: QueryAs<'q, Postgres, O, A>,
	) -> DbxResult<Option<O>>
	where
		O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let data = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.fetch_optional(txn.as_mut()).await?
			} else {
				query.fetch_optional(self.db()).await?
			}
		} else {
			query.fetch_optional(self.db()).await?
		};

		Ok(data)
	}

	pub async fn fetch_all<'q, O, A>(
		&self,
		query: QueryAs<'q, Postgres, O, A>,
	) -> DbxResult<Vec<O>>
	where
		O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let data = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.fetch_all(txn.as_mut()).await?
			} else {
				query.fetch_all(self.db()).await?
			}
		} else {
			query.fetch_all(self.db()).await?
		};

		Ok(data)
	}

	pub async fn execute<'q, A>(&self, query: Query<'q, Postgres, A>) -> DbxResult<u64>
	where
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let row_affected = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.execute(txn.as_mut()).await?.rows_affected()
			} else {
				query.execute(self.db()).await?.rows_affected()
			}
		} else {
			query.execute(self.db()).await?.rows_affected()
		};

		Ok(row_affected)
	}
}
