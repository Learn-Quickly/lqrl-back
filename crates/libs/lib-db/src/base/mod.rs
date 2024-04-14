pub mod table_ref;
pub mod idens;

mod crud_fns;
mod macro_utils;
mod utils;

// -- Flatten hierarchy for user code.
pub use crud_fns::*;
pub use utils::*;

use modql::SIden;
use sea_query::{IntoIden, TableRef};

const LIST_LIMIT_DEFAULT: i64 = 1000;
const LIST_LIMIT_MAX: i64 = 5000;

/// The DbRepository trait must be implemented for the Bmc struct of an entity.
/// It specifies meta information such as the table name,
/// whether the table has timestamp columns (cid, ctime, mid, mtime), and more as the
/// code evolves.
///
/// Note: This trait should not be confused with the BaseCrudBmc trait, which provides
///       common default CRUD BMC functions for a given Bmc/Entity.
pub trait DbRepository {
	const TABLE: &'static str;

	fn table_ref() -> TableRef {
		TableRef::Table(SIden(Self::TABLE).into_iden())
	}

	/// Specifies that the table for this Bmc has timestamps (cid, ctime, mid, mtime) columns.
	/// This will allow the code to update those as needed.
	///
	/// default: true
	fn has_timestamps() -> bool {
		true
	}

	/// Specifies if the entity table managed by this BMC
	/// has an `owner_id` column that needs to be set on create (by default ctx.user_id).
	///
	/// default: false
	fn has_owner_id() -> bool {
		false
	}
}
