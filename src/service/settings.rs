use crate::state::AppState;
use crate::entity::prelude::*;
use sea_orm::{EntityTrait, Set, ActiveModelTrait};
use chrono::Utc;

pub const SETTING_REGISTRATION_ENABLED: &str = "registration_enabled";
pub const SETTING_ALLOW_UPLOADER_EDIT: &str = "allow_uploader_edit";
pub const SETTING_ALLOW_UPLOADER_DELETE: &str = "allow_uploader_delete";
pub const SETTING_ENABLE_UPLOAD_REVIEW: &str = "enable_upload_review";
pub const SETTING_ALLOW_COMMENTS: &str = "allow_comments";
pub const SETTING_ENABLE_COMMENT_REVIEW: &str = "enable_comment_review";


pub async fn get_or_create_setting(
    state: &AppState,
    key: &str,
    default_value: &str,
) -> Result<SettingsModel, sea_orm::DbErr> {
    match Settings::find_by_id(key).one(&state.db).await {
        Ok(Some(setting)) => Ok(setting),
        Ok(None) => {
            let now = Utc::now().naive_utc();
            let setting = SettingsActiveModel {
                key: Set(key.to_string()),
                value: Set(default_value.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };
            setting.insert(&state.db).await
        }
        Err(e) => Err(e),
    }
}


pub async fn is_registration_enabled(state: &AppState) -> bool {
    match get_or_create_setting(state, SETTING_REGISTRATION_ENABLED, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    }
}

pub async fn is_uploader_edit_allowed(state: &AppState) -> bool {
    match get_or_create_setting(state, SETTING_ALLOW_UPLOADER_EDIT, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    }
}

pub async fn is_uploader_delete_allowed(state: &AppState) -> bool {
    match get_or_create_setting(state, SETTING_ALLOW_UPLOADER_DELETE, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    }
}

pub async fn is_upload_review_enabled(state: &AppState) -> bool {
    match get_or_create_setting(state, SETTING_ENABLE_UPLOAD_REVIEW, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    }
}

pub async fn is_comments_allowed(state: &AppState) -> bool {
    match get_or_create_setting(state, SETTING_ALLOW_COMMENTS, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    }
}

pub async fn is_comment_review_enabled(state: &AppState) -> bool {
    match get_or_create_setting(state, SETTING_ENABLE_COMMENT_REVIEW, "true").await {
        Ok(s) => s.value == "true",
        Err(_) => true,
    }
}
