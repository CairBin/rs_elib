mod dto;
mod request;
mod response;

use axum::{
    extract::{State, Json},
    response::{Response},
};
use sea_orm::{EntityTrait, Set, QueryFilter, ColumnTrait, ActiveModelTrait, PaginatorTrait};
use crate::{middleware::auth::AuthMiddleware, service};
use crate::utils::{api::*, validator, auth};
use crate::entity::user;
use crate::state::AppState;
use crate::entity::prelude::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use dto::*;
use request::*;
use response::*;

/// 注册业务逻辑
#[axum::debug_handler]
pub async fn register(
    State(state) : State<AppState>,
    Json(req) : Json<RegisterRequest>
) -> Response{
    tracing::info!("Register request for username: {}", req.username);
    
    if !service::settings::is_registration_enabled(&state).await {
        tracing::warn!("Registration is disabled, rejecting user: {}", req.username);
        return forbidden("Registration is disabled");
    }

    if let Err(error) = validator::validate_password(&req.password){
        tracing::warn!("Password validation failed for user: {}, error: {}", req.username, error);
        return bad_request(error);
    }

    let existing = User::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(&state.db)
        .await;

    match existing {
        Ok(Some(_)) => {
            tracing::warn!("Username already exists: {}", req.username);
            return conflict("Username already exists");
        }

        Err(e) => {
            tracing::error!("Database error during registration for user {}: {:?}", req.username, e);
            return internal_error("Database error");
        }

        Ok(None) => {}
    };

    let username = req.username.clone();
    let password_hash = hash(&req.password, DEFAULT_COST).unwrap();
    let count = User::find().count(&state.db).await.unwrap_or(0);
    let role = if count == 0 { "root".to_string() } else { "user".to_string() };

    let user = UserActiveModel {
        username: Set(req.username),
        password_hash: Set(password_hash),
        role: Set(role.clone()),
        disabled: Set(false),
        ..Default::default()
    };

    match user.insert(&state.db).await {
        Ok(user) => {
            tracing::info!("User registered successfully: username={}, role={}", user.username, role);
            created(UserDto::from_entity(user))
        },
        Err(e) => {
            tracing::error!("Failed to create user {}: {:?}", username, e);
            internal_error("Failed to create user")
        }
    }
}


/// 登录业务逻辑
pub async fn login(
    State(state) : State<AppState>,
    Json(req) : Json<request::LoginRequest>
) -> Response {
    tracing::info!("Login request for username: {}", req.username);
    
    let user = match User::find().filter(user::Column::Username.eq(&req.username)).one(&state.db).await{
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("Login failed: user not found - {}", req.username);
            return unauthorized("Invalid credentials");
        },
        Err(e) => {
            tracing::error!("Database error during login for user {}: {:?}", req.username, e);
            return internal_error("Database error");
        },
    };

    if user.disabled {
        tracing::warn!("Login failed: user is disabled - {}", req.username);
        return forbidden("User is disabled");
    }

    if !verify(&req.password, &user.password_hash).unwrap_or(false) {
        tracing::warn!("Login failed: invalid password - {}", req.username);
        return unauthorized("Invalid credentials");
    }

    let token = auth::create_token(user.id, user.username.clone(), user.role.clone()).unwrap();

    tracing::info!("User logged in successfully: username={}, role={}", user.username, user.role);

    success(
        LoginResponse{
            token,
            user: UserDto::from_entity(user)
        }
    )
}

#[axum::debug_handler]
pub async fn get_current_user(
    AuthMiddleware(claims): AuthMiddleware
) -> Response{
    success(UserDto{
        id: claims.sub,
        username: claims.username,
        role: claims.role,
    })
}