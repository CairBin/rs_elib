mod request;
mod dto;
mod response;

use axum::{
    extract::{State, Json, Path},
    response::Response,
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use sea_orm::{EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use crate::utils::api::*;
use request::*;
use dto::*;
use crate::permissions;
use crate::entity::prelude::*;
use crate::state::AppState;
use crate::middleware::auth::AuthMiddleware;
use crate::service::user as user_service;
use crate::utils::validator;
use crate::entity::user;

/// 获取所有用户
#[axum::debug_handler]
pub async fn get_users(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response{
    if claims.role != "admin" && claims.role != "root" {
        return forbidden("Access denied");
    }


    let users = User::find().all(&state.db).await.unwrap_or_default();
    let user_responses: Vec<UserDto> = users.into_iter().map(|u| u.into()).collect();

    success(user_responses)
}


/// 查询指定id用户
#[axum::debug_handler]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware
) -> Response{
    // 允许管理员/超级管理员查询任何用户
    // 允许用户查询自己信息
    // 不允许非管理员查询其他用户信息
    if claims.role != "admin" && claims.role != "root" && claims.sub != id {
        return forbidden("Access denied");
    }

    match User::find_by_id(id).one(&state.db).await {
        Ok(Some(user)) => success(UserDto::from(user)),
        Ok(None) => not_found("User not found"),
        Err(_) => internal_error("Db error"),
    }

}


/// 更改用户身份
#[axum::debug_handler]
pub async fn update_user_role(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<UpdateUserRoleRequest>,
) -> Response {
    let target_user = match User::find_by_id(id).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => return not_found("User not found"),
        Err(_) => return internal_error("Server error"),
    };

    // 不允许设置更多的超级管理员
    if req.role == "root" {
        return forbidden("Cannot set root account");
    }

    // 不允许更改超级管理员权限
    if target_user.role == "root" {
        return forbidden("Cannot modify the role of root account");
    }


    if !permissions::can_change_user_role(&state, claims.sub, &claims.role, id, &req.role).await{
        return forbidden("Access denied");
    }

    let mut user: UserActiveModel = target_user.into();
    user.role = Set(req.role);
    match user.update(&state.db).await {
        Ok(user) => success(UserDto::from(user)),
        Err(_) => internal_error("Failed to update user"),
    }
}

#[axum::debug_handler]
pub async fn update_user_password(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<UpdateUserPasswordRequest>,
) -> Response {
    let target_user = match User::find_by_id(id).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => return not_found("User not found"),
        Err(_) => return internal_error("Db error"),
    };

    // 格式不正确
    if validator::validate_password(&req.password).is_err(){
        return bad_request("Password does not meet the requirements");
    }

    // 此方法根据身份来选择业务逻辑
    let res = match claims.role.as_str() {
        "root" => user_service::update_password_by_root(&state, target_user, &req.password).await,
        "admin" => user_service::update_password_by_admin(&state, target_user, claims.sub, id, &req.password).await,
        _ => user_service::update_password_by_contributor_or_reader(&state, target_user, claims.sub, id, &req.password).await,
    };

    if res.is_err(){
        return forbidden(res.err().unwrap());
    }

    success("Password updated successfully")
}


#[axum::debug_handler]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response{
    if claims.role != "root" && claims.role != "admin" {
        return forbidden("Access denied");
    }

    let target_user = match User::find_by_id(id).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => return not_found("User not found"),
        Err(_) => return internal_error("Db error"),
    };

    // root 用户是无法被删除的
    if target_user.role == "root" {
        return forbidden("Cannot delete root account");
    }

    if claims.role == "root" {
        return match user_service::delete_by_id(&state, id).await{
            Ok(_) => success("User deleted successfully"),
            Err(_) => internal_error("Failed to delete user"),
        };
    }

    if claims.role == "admin" && target_user.role == "admin"{
        return forbidden("Cannot delete other admin account");
    }

    return match user_service::delete_by_id(&state, id).await{
            Ok(_) => success("User deleted successfully"),
            Err(_) => internal_error("Failed to delete user"),
    };
}

#[axum::debug_handler]
pub async fn create_user(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<CreateUserRequest>,
) -> Response{
    if claims.role != "admin" && claims.role != "root"{
        return forbidden("Access denied");
    }

    if req.role == "root" {
        return bad_request("Root account is just only one");
    }

    if claims.role == "admin" && req.role == "admin"{
        return forbidden("Only root can create admin accounts");
    }

    // 密码格式不正确
    if let Err(error) = validator::validate_password(&req.password){
        return bad_request(error);
    }

    // 检查用户是否存在
    let existing = User::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(&state.db)
        .await;
    match existing {
        Ok(Some(_)) => {
            return conflict("Username already exists");
        },

        Ok(None) => {},

        Err(_) => {
            return internal_error("Database error");
        }
    };

    let password_hash = hash(&req.password, DEFAULT_COST).unwrap();
    let now = Utc::now().naive_utc();
    let user = UserActiveModel{
        username: Set(req.username),
        password_hash: Set(password_hash),
        role: Set(req.role),
        disabled: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match user.insert(&state.db).await {
        Ok(user) => created(UserDto::from(user)),
        Err(_) => internal_error("Failed to create user")
    }
}


#[axum::debug_handler]
pub async fn update_user_profile(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<UpdateUserProfileRequest>
) -> Response {
    let user = match User::find_by_id(claims.sub).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => return not_found("User not found"),
        Err(_) => return internal_error("Db error")
    };


    let mut user: UserActiveModel = user.into();
    if let Some(password) = req.password {
        if let Err(error) = validator::validate_password(&password){
            return bad_request(error);
        }

        let password_hash = hash(&password, DEFAULT_COST).unwrap();
        user.password_hash = Set(password_hash);
    }

    match user.update(&state.db).await {
        Ok(user) => success(UserDto::from(user)),
        Err(_) => internal_error("Failed to update user"),
    }
}

#[axum::debug_handler]
pub async fn update_user_disabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<UpdateUserDisabledRequest>
) -> Response {
    if claims.role != "root" && claims.role != "admin" {
        return forbidden("Access denied");
    }

    let target_user = match User::find_by_id(id).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => return not_found("User not found"),
        Err(_) => return internal_error("Db error")
    };

    if target_user.role == "root" {
        return forbidden("Cannot disable root account");
    }

    if target_user.role == "admin" && claims.role != "root"{
        return forbidden("Only root can disable admin account");
    }

    if claims.sub == id {
        return forbidden("Cannot disable self account");
    }

    let mut user: UserActiveModel = target_user.into();
    user.disabled = Set(req.disabled);
    
    match user.update(&state.db).await {
        Ok(user) => success(UserDto::from(user)),
        Err(_) => internal_error("Failed to update user")
    }
}