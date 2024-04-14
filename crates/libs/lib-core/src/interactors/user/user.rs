use lib_auth::pwd::{self, ContentToHash};
use uuid::Uuid;

use crate::{
    ctx::Ctx, interactors::error::UserError, interfaces::{command_repository_manager::ICommandRepositoryManager, user::UserResult}, models::user::{
        UserForChangePassword,
        UserForCreate,
        UserForUpdate
    }
};

pub struct UserInteractor<'a> {
    repository_manager: &'a (dyn ICommandRepositoryManager + Send + Sync),
}

impl<'a> UserInteractor<'a> {
    pub fn new(repository_manager: &'a (dyn ICommandRepositoryManager + Send + Sync)) -> Self {
        Self {
            repository_manager,
        }
    }
}

impl<'a> UserInteractor<'a> {
    pub async fn create_user(
        &self,
        ctx: &Ctx,
        pwd_clear: String,
        username: String
    ) -> UserResult<i64> {
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

        let user_repository = self.repository_manager.get_user_repository();
        user_repository.create_user(ctx, user_for_c).await
    }

    pub async fn update_user(
        &self,
        ctx: &Ctx,
        user_for_u: UserForUpdate
    ) -> UserResult<()> {
        let user_repository = self.repository_manager.get_user_repository();
        user_repository.update_user(ctx, user_for_u).await
    }

    pub async fn change_pwd(
        &self,
        ctx: &Ctx,
        user_for_change_pwd: UserForChangePassword
    ) -> UserResult<()> {
        let user_id = ctx.user_id();
        let user_repository = self.repository_manager.get_user_repository();
        let user = user_repository.get_user(ctx, user_id).await?;

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

        self.update_pwd(ctx, ctx.user_id(), &user_for_change_pwd.new_pwd_clear).await
    }

    pub async fn update_pwd(
        &self, 
        ctx: &Ctx,
        user_id: i64, 
        pwd_clear: &str
    ) -> UserResult<()> {
        let user_repository = self.repository_manager.get_user_repository();
        let user = user_repository.get_user(ctx, user_id).await?;
		let pwd = pwd::hash_pwd(ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt,
		})
		.await
        .map_err(UserError::PwdError)?;

        user_repository.update_pwd(ctx, user_id, pwd).await
    }
}