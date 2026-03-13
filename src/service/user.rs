use bcrypt::{hash, DEFAULT_COST};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use crate::{entity::prelude::*, state::AppState};


/// 修改密码
async fn update_password(state: &AppState, target_user: UserModel, new_password: &str) -> Result<(), &'static str>{
    let password_hash = match hash(new_password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return Err("Failed to hash new password")
    };

    let mut user:UserActiveModel = target_user.into();
    user.password_hash = Set(password_hash);
    
    match user.update(&state.db).await {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to update user password"),
    }
}

/// root用户修改密码
pub async fn update_password_by_root(state: &AppState, target_user: UserModel, new_password: &str) ->Result<(), &'static str>{
    update_password(state, target_user, new_password).await
}

/// admin用户修改密码
pub async fn update_password_by_admin(
    state: &AppState, 
    target_user: UserModel, 
    path_id:i32, 
    claim_id: i32, 
    new_password: &str
) -> Result<(), &'static str>{
    if target_user.role == "root"{
        return Err("Access denied");
    }

    if target_user.role == "admin"{
        if path_id != claim_id{
            return Err("Access denied");
        }
        return update_password(state, target_user, new_password).await;
    }

    // 可以修改贡献者和阅读者的密码
    update_password(state, target_user, new_password).await
}

/// 贡献者和阅读者修改密码
pub async fn update_password_by_contributor_or_reader(
    state: &AppState,
    target_user: UserModel,
    path_id: i32,
    claim_id: i32,
    new_password: &str
) -> Result<(), &'static str>{
    // 普通用户仅能修改自己的代码
    if target_user.role == "root" || target_user.role == "admin" || path_id != claim_id {
        return Err("Access denied");
    }

    update_password(state, target_user, new_password).await
}


/// 按照id删除用户
pub async fn delete_by_id(state: &AppState, id: i32) -> Result<(), &'static str> {
    match User::delete_by_id(id).exec(&state.db).await{
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to delete user"),
    }
}
