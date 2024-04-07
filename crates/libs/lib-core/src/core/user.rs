use lib_auth::pwd::{self, ContentToHash};
use uuid::Uuid;

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

pub struct UserInteractor<'a> {
    ctx: &'a Ctx,
    repository: &'a (dyn IUserCommandRepository + Send + Sync),
}

impl<'a> UserInteractor<'a> {
    pub fn new(ctx: &'a Ctx, repository: &'a (impl IUserCommandRepository + Send + Sync)) -> Self {
        Self {
            ctx,
            repository,
        }
    }
}

impl<'a> UserInteractor<'a> {
    pub async fn create_user(&self, pwd_clear: String, username: String) -> UserResult<i64> {
        let pwd_salt = Uuid::new_v4();
        let token_salt = Uuid::new_v4();

        let pwd = pwd::hash_pwd(ContentToHash {
			content: pwd_clear.to_string(),
			salt: pwd_salt,
		})
		.await
        .map_err(|err| UserError::PwdError(err))?;
        
        let user_for_c = UserForCreate {
            username,
            pwd,
            pwd_salt,
            token_salt,
        };

        self.repository.create_user(&self.ctx, user_for_c).await
    }

    pub async fn update_user(&self, user_for_u: UserForUpdate) -> UserResult<()> {
        self.repository.update_user(&self.ctx, user_for_u).await
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

        self.update_pwd(self.ctx.user_id(), &user_for_change_pwd.new_pwd_clear).await
    }

    pub async fn update_pwd(&self, user_id: i64, pwd_clear: &str) -> UserResult<()> {
        let user = self.repository.get_user(&self.ctx, user_id).await?;
		let pwd = pwd::hash_pwd(ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt,
		})
		.await
        .map_err(UserError::PwdError)?;

        self.repository.update_pwd(&self.ctx, user_id, pwd).await
    }
}