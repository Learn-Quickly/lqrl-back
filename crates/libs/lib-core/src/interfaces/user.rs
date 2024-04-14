use async_trait::async_trait;

use crate::{interactors::error::CoreError, ctx::Ctx, models::user::{User, UserForCreate, UserForUpdate}};


pub type UserResult<T> = core::result::Result<T, CoreError>;

#[async_trait]
pub trait IUserCommandRepository {
    async fn get_user(&self, ctx: &Ctx, user_id: i64) -> UserResult<User>;

    async fn create_user(&self, ctx: &Ctx, user_for_c: UserForCreate) -> UserResult<i64>;

    async fn update_user(&self, ctx: &Ctx, user_for_u: UserForUpdate) -> UserResult<()>;

    async fn update_pwd(&self, ctx: &Ctx, user_id: i64, new_pwd_clear: String) -> UserResult<()>;
}