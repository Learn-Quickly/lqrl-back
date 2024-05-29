use crate::base::DbRepository;
use lib_utils::time::now_utc;
use modql::field::{Field, Fields};
use sea_query::IntoIden;

use super::idens::{CommonIden, TimestampIden};

pub fn prep_fields_for_create<MC>(fields: &mut Fields, user_id: i64)
where
	MC: DbRepository,
{
	if MC::has_owner_id() {
		fields.push(Field::new(CommonIden::OwnerId.into_iden(), user_id.into()));
	}
	if MC::has_timestamps() {
		add_timestamps_for_create(fields, user_id);
	}
}

pub fn prep_fields_for_update<MC>(fields: &mut Fields, user_id: i64)
where
	MC: DbRepository,
{
	if MC::has_timestamps() {
		add_timestamps_for_update(fields, user_id);
	}
}

fn add_timestamps_for_create(fields: &mut Fields, user_id: i64) {
	let now = now_utc();
	fields.push(Field::new(TimestampIden::Cid.into_iden(), user_id.into()));
	fields.push(Field::new(TimestampIden::Ctime.into_iden(), now.into()));

	fields.push(Field::new(TimestampIden::Mid.into_iden(), user_id.into()));
	fields.push(Field::new(TimestampIden::Mtime.into_iden(), now.into()));
}

fn add_timestamps_for_update(fields: &mut Fields, user_id: i64) {
	let now = now_utc();
	fields.push(Field::new(TimestampIden::Mid.into_iden(), user_id.into()));
	fields.push(Field::new(TimestampIden::Mtime.into_iden(), now.into()));
}
