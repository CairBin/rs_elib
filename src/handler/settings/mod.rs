mod request;

use axum::{
    extract::{State, Path, Json},
    response::Response,
};

use chrono::Utc;
use request::*;
use crate::entity::prelude::*;
use crate::state::AppState;
use crate::middleware::auth::AuthMiddleware;
use crate::utils::api::*;
use crate::service::settings as settings_service;
use sea_orm::{EntityTrait, Set, ActiveModelTrait};

pub const SETTING_REGISTRATION_ENABLED: &str = "registration_enabled";


#[axum::debug_handler]
pub async fn get_settings(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    if claims.role != "admin" && claims.role != "root"{
        return forbidden("Access denied");
    }

    let registration_enabled = match settings_service::get_or_create_setting(&state, SETTING_REGISTRATION_ENABLED, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    };

    let allow_uploader_edit = match settings_service::get_or_create_setting(&state, settings_service::SETTING_ALLOW_UPLOADER_EDIT, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    };

    let allow_uploader_delete = match settings_service::get_or_create_setting(&state, settings_service::SETTING_ALLOW_UPLOADER_DELETE, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    };

    let enable_upload_review = match settings_service::get_or_create_setting(&state, settings_service::SETTING_ENABLE_UPLOAD_REVIEW, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    };

    let allow_comments = match settings_service::get_or_create_setting(&state, settings_service::SETTING_ALLOW_COMMENTS, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    };

    let enable_comment_review = match settings_service::get_or_create_setting(&state, settings_service::SETTING_ENABLE_COMMENT_REVIEW, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    };


    let settings = serde_json::json!({
        "registration_enabled": registration_enabled,
        "allow_uploader_edit": allow_uploader_edit,
        "allow_uploader_delete": allow_uploader_delete,
        "enable_upload_review": enable_upload_review,
        "allow_comments": allow_comments,
        "enable_comment_review": enable_comment_review,
    });

    success(settings)
}

#[axum::debug_handler]
pub async fn update_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<UpdateSettingRequest>
) -> Response {
    if claims.role != "admin" && claims.role != "root" {
        return forbidden("Access denied");
    }


    let setting = match Settings::find_by_id(&key).one(&state.db).await {
        Ok(Some(setting)) => {
            let mut setting: SettingsActiveModel = setting.into();
            setting.value = Set(req.value);
            setting.update(&state.db).await
        },

        Ok(None) => {
            let now = Utc::now().naive_local();
            let setting = SettingsActiveModel {
                key: Set(key),
                value: Set(req.value),
                created_at: Set(now),
                updated_at: Set(now),
            };
            setting.insert(&state.db).await
        },

        Err(e) => return internal_error(&format!("Db error: {}", e)), 
    };

    match setting {
        Ok(s) => success(s),
        Err(_) => internal_error("Failed to update this setting")
    }
}

