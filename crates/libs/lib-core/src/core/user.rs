use lib_auth::pwd::{self, ContentToHash};

use crate::{
    ctx::Ctx,
    interfaces::user::{
        IUserCommandRepository, UserResult
    },
    model::user::{
        UserForChangePassword,
        UserForCreate,
        UserForUpdate
    }
};

use super::error::UserError;

pub struct UserController<'a> {
    ctx: &'a Ctx,
    repository: &'a (dyn IUserCommandRepository + Send + Sync),
}

impl<'a> UserController<'a> {
    pub fn new(ctx: &'a Ctx, repository: &'a (impl IUserCommandRepository + Send + Sync)) -> Self {
        Self {
            ctx,
            repository,
        }
    }
}

impl<'a> UserController<'a> {
    pub async fn create_user(&self, user_for_c: UserForCreate) -> UserResult<i64> {
        self.repository.create_user(&self.ctx, user_for_c).await
    }

    pub async fn update_user(&self, user_for_u: UserForUpdate) -> UserResult<()> {
        self.repository.update_user(&self.ctx, user_for_u).await
    }

    /// For update scheme
    pub async fn update_pwd(&self, user_id: i64, pwd: &str) -> UserResult<()> {
        self.repository.update_pwd(&self.ctx, user_id, pwd).await
    }

    pub async fn change_pwd(&self, user_for_change_pwd: UserForChangePassword) -> UserResult<()> {
        let user_id = self.ctx.user_id();
        let user = self.repository.get_user(&self.ctx, user_id).await?;

        let pwd_clear = user_for_change_pwd.pwd_clear;

	    pwd::validate_pwd(
		    ContentToHash {
			    salt: user.pwd_salt,
			    content: pwd_clear.clone(),
		    },
		    user.pwd,
	    )
	    .await
	    .map_err(|_| UserError::WrongPasswordError{ user_id })?;

        self.repository.update_pwd(&self.ctx, self.ctx.user_id(), &user_for_change_pwd.new_pwd_clear).await
    }
}