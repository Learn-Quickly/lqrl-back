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

pub trait DbRepository {
	const TABLE: &'static str;

	fn table_ref() -> TableRef {
		TableRef::Table(SIden(Self::TABLE).into_iden())
	}

	fn has_timestamps() -> bool {
		true
	}

	fn has_owner_id() -> bool {
		false
	}
}
