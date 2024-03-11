/// Convenience macro rules to generate default CRUD functions for a Bmc/Entity.
/// Note: If custom functionality is required, use the code below as foundational
///       code for the custom implementations.
#[macro_export]
macro_rules! generate_common_bmc_fns {
	(
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		$(ForCreate: $for_create:ty,)?
		$(ForUpdate: $for_update:ty,)?
		$(Filter: $filter:ty,)?
	) => {
		impl $struct_name {
			$(
				pub async fn create(
					ctx: &Ctx,
					dbm: &DbManager,
					entity_c: $for_create,
				) -> Result<i64> {
					base::create::<Self, _>(ctx, dbm, entity_c).await
				}
			)?

				pub async fn get(
					ctx: &Ctx,
					dbm: &DbManager,
					id: i64,
				) -> Result<$entity> {
					base::get::<Self, _>(ctx, dbm, id).await
				}

			$(
				pub async fn first(
					ctx: &Ctx,
					dbm: &DbManager,
					filter: Option<Vec<$filter>>,
					list_options: Option<ListOptions>,
				) -> Result<Option<$entity>> {
					base::first::<Self, _, _>(ctx, dbm, filter, list_options).await
				}

				pub async fn list(
					ctx: &Ctx,
					dbm: &DbManager,
					filter: Option<Vec<$filter>>,
					list_options: Option<ListOptions>,
				) -> Result<Vec<$entity>> {
					base::list::<Self, _, _>(ctx, dbm, filter, list_options).await
				}
			)?

			$(
				pub async fn update(
					ctx: &Ctx,
					dbm: &DbManager,
					id: i64,
					entity_u: $for_update,
				) -> Result<()> {
					base::update::<Self, _>(ctx, dbm, id, entity_u).await
				}
			)?

				pub async fn delete(
					ctx: &Ctx,
					dbm: &DbManager,
					id: i64,
				) -> Result<()> {
					base::delete::<Self>(ctx, dbm, id).await
				}
		}
	};
}
