mod request;
mod dto;

use request::*;
use dto::*;
use axum::{
    extract::{Path, State, Json},
    response::Response,
};
use sea_orm::{EntityTrait, Set, QueryFilter, ColumnTrait, ActiveModelTrait, QuerySelect, RelationTrait, JoinType, PaginatorTrait};
use crate::entity::prelude::*;
use crate::entity::user;
use crate::entity::group;
use crate::entity::user_group;
use crate::entity::book;
use crate::entity::book_group;
use crate::entity::invite_code;
use crate::state::AppState;
use crate::middleware::auth::AuthMiddleware;
use crate::utils::api::*;
use chrono::{Utc, Duration};


#[axum::debug_handler]
pub async fn get_groups(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let groups = if claims.role == "admin" || claims.role == "root" {
        Group::find().all(&state.db).await.unwrap_or_default()
    } else {
        let joined_groups = Group::find()
            .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
            .filter(user_group::Column::UserId.eq(claims.sub))
            .all(&state.db)
            .await
            .unwrap_or_default();
        
        let created_groups = Group::find()
            .filter(group::Column::CreatedBy.eq(Some(claims.sub)))
            .all(&state.db)
            .await
            .unwrap_or_default();
        
        let mut all_groups = joined_groups;
        for created_group in created_groups {
            if !all_groups.iter().any(|g| g.id == created_group.id) {
                all_groups.push(created_group);
            }
        }
        
        all_groups
    };

    success(groups)
}



#[axum::debug_handler]
pub async fn get_group(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let group = match Group::find_by_id(id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    if claims.role != "admin" && claims.role != "root" {
        let is_owner = group.created_by == Some(claims.sub);
        
        let has_access = if is_owner {
            true
        } else {
            Group::find()
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(group::Column::Id.eq(id))
                .count(&state.db)
                .await
                .unwrap_or(0) > 0
        };

        if !has_access {
            return forbidden("Access denied");
        }
    }

    success(group)
}



#[axum::debug_handler]
pub async fn create_group(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<CreateGroupRequest>,
) -> Response {
    if claims.role != "admin" && claims.role != "root" && claims.role != "contributor" {
        return forbidden("Access denied");
    }

    let now = Utc::now().naive_utc();

    let group = group::ActiveModel {
        name: Set(req.name),
        description: Set(req.description),
        created_by: Set(Some(claims.sub)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let group = match group.insert(&state.db).await {
        Ok(group) => group,
        Err(_) => return internal_error("Failed to create group"),
    };

    let user_group = user_group::ActiveModel {
        user_id: Set(claims.sub),
        group_id: Set(group.id),
        created_at: Set(now),
        ..Default::default()
    };

    let _ = user_group.insert(&state.db).await;

    created(group)
}


#[axum::debug_handler]
pub async fn update_group(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<UpdateGroupRequest>,
) -> Response {
    let group = match Group::find_by_id(id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub)
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    let now = Utc::now().naive_utc();

    let mut group: group::ActiveModel = group.into();

    if let Some(name) = req.name {
        group.name = Set(name);
    }
    if let Some(description) = req.description {
        group.description = Set(Some(description));
    }
    group.updated_at = Set(now);

    match group.update(&state.db).await {
        Ok(group) => success(group),
        Err(_) => internal_error("Failed to update group"),
    }
}



#[axum::debug_handler]
pub async fn delete_group(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let group = match Group::find_by_id(id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub)
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    let _ = UserGroup::delete_many()
        .filter(user_group::Column::GroupId.eq(id))
        .exec(&state.db)
        .await;

    let _ = BookGroup::delete_many()
        .filter(book_group::Column::GroupId.eq(id))
        .exec(&state.db)
        .await;

    match Group::delete_by_id(id).exec(&state.db).await {
        Ok(_) => success("Group deleted"),
        Err(_) => internal_error("Failed to delete group"),
    }
}

#[axum::debug_handler]
pub async fn add_user_to_group(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<AddUserToGroupRequest>,
) -> Response {
    let group = match Group::find_by_id(id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub)
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    let now = Utc::now().naive_utc();

    let user_group = user_group::ActiveModel {
        user_id: Set(req.user_id),
        group_id: Set(id),
        created_at: Set(now),
        ..Default::default()
    };

    match user_group.insert(&state.db).await {
        Ok(ug) => created(ug),
        Err(_) => internal_error("Failed to add user to group"),
    }
}


#[axum::debug_handler]
pub async fn add_book_to_group(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<AddBookToGroupRequest>,
) -> Response {
    let group = match Group::find_by_id(id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub)
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    let now = Utc::now().naive_utc();

    let book_group = book_group::ActiveModel {
        book_id: Set(req.book_id),
        group_id: Set(id),
        created_at: Set(now),
        ..Default::default()
    };

    match book_group.insert(&state.db).await {
        Ok(bg) => created(bg),
        Err(_) => internal_error("Failed to add book to group"),
    }
}


#[axum::debug_handler]
pub async fn get_group_users(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let has_access = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        Group::find()
            .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
            .filter(user_group::Column::UserId.eq(claims.sub))
            .filter(group::Column::Id.eq(id))
            .count(&state.db)
            .await
            .unwrap_or(0) > 0
    };

    if !has_access {
        return forbidden("Access denied");
    }

    let users = User::find()
        .join(JoinType::InnerJoin, user::Relation::UserGroups.def())
        .filter(user_group::Column::GroupId.eq(id))
        .all(&state.db)
        .await
        .unwrap_or_default();
    
    let user_responses: Vec<UserDto> = users.into_iter().map(|u| u.into()).collect();

    success(user_responses)
}


#[axum::debug_handler]
pub async fn get_group_books(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let has_access = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        Group::find()
            .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
            .filter(user_group::Column::UserId.eq(claims.sub))
            .filter(group::Column::Id.eq(id))
            .count(&state.db)
            .await
            .unwrap_or(0) > 0
    };

    if !has_access {
        return forbidden("Access denied");
    }

    let books = Book::find()
        .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
        .filter(book_group::Column::GroupId.eq(id))
        .all(&state.db)
        .await
        .unwrap_or_default();

    success(books)
}

#[axum::debug_handler]
pub async fn remove_user_from_group(
    State(state): State<AppState>,
    Path((id, user_id)): Path<(i32, i32)>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let group = match Group::find_by_id(id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub)
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    match UserGroup::delete_many()
        .filter(user_group::Column::GroupId.eq(id))
        .filter(user_group::Column::UserId.eq(user_id))
        .exec(&state.db)
        .await
    {
        Ok(_) => success("User removed from group"),
        Err(_) => internal_error("Failed to remove user from group"),
    }
}

#[axum::debug_handler]
pub async fn remove_book_from_group(
    State(state): State<AppState>,
    Path((id, book_id)): Path<(i32, i32)>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let group = match Group::find_by_id(id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub)
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    match BookGroup::delete_many()
        .filter(book_group::Column::GroupId.eq(id))
        .filter(book_group::Column::BookId.eq(book_id))
        .exec(&state.db)
        .await
    {
        Ok(_) => success("Book removed from group"),
        Err(_) => internal_error("Failed to remove book from group"),
    }
}


fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    const CODE_LENGTH: usize = 8;
    let uuid = uuid::Uuid::new_v4();
    let mut bytes = uuid.as_bytes().iter();
    (0..CODE_LENGTH)
        .map(|_| {
            let byte = bytes.next().unwrap_or(&0);
            let idx = (byte % CHARSET.len() as u8) as usize;
            CHARSET[idx] as char
        })
        .collect()
}

#[axum::debug_handler]
pub async fn create_invite_code(
    State(state): State<AppState>,
    Path(group_id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<CreateInviteCodeRequest>,
) -> Response {
    let group = match Group::find_by_id(group_id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub)
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    let now = Utc::now().naive_utc();
    let code = generate_invite_code();
    let expires_at = req.expires_in_days.and_then(|days| {
        if days > 0 {
            Some(now + Duration::days(days as i64))
        } else {
            None
        }
    });

    let invite_code = invite_code::ActiveModel {
        code: Set(code),
        group_id: Set(group_id),
        created_by: Set(claims.sub),
        max_users: Set(req.max_users),
        used_count: Set(0),
        expires_at: Set(expires_at),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match invite_code.insert(&state.db).await {
        Ok(invite_code) => created(InviteCodeDto::from(invite_code)),
        Err(_) => internal_error("Failed to create invite code"),
    }
}

#[axum::debug_handler]
pub async fn get_group_invite_codes(
    State(state): State<AppState>,
    Path(group_id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let group = match Group::find_by_id(group_id).one(&state.db).await {
        Ok(Some(group)) => group,
        Ok(None) => return not_found("Group not found"),
        Err(_) => return internal_error("Database error"),
    };

    let can_view = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        group.created_by == Some(claims.sub) || {
            Group::find()
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(group::Column::Id.eq(group_id))
                .count(&state.db)
                .await
                .unwrap_or(0) > 0
        }
    };

    if !can_view {
        return forbidden("Access denied");
    }

    let invite_codes = InviteCode::find()
        .filter(invite_code::Column::GroupId.eq(group_id))
        .all(&state.db)
        .await
        .unwrap_or_default();

    let responses: Vec<InviteCodeDto> = invite_codes.into_iter().map(|c| c.into()).collect();
    success(responses)
}

#[axum::debug_handler]
pub async fn deactivate_invite_code(
    State(state): State<AppState>,
    Path((group_id, code_id)): Path<(i32, i32)>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let invite_code = match InviteCode::find_by_id(code_id).one(&state.db).await {
        Ok(Some(code)) => code,
        Ok(None) => return not_found("Invite code not found"),
        Err(_) => return internal_error("Database error"),
    };

    if invite_code.group_id != group_id {
        return bad_request("Invalid group");
    }

    let can_manage = if claims.role == "admin" || claims.role == "root" {
        true
    } else {
        invite_code.created_by == claims.sub
    };

    if !can_manage {
        return forbidden("Access denied");
    }

    let now = Utc::now().naive_utc();
    let mut invite_code: invite_code::ActiveModel = invite_code.into();
    invite_code.is_active = Set(false);
    invite_code.updated_at = Set(now);

    match invite_code.update(&state.db).await {
        Ok(code) => success(InviteCodeDto::from(code)),
        Err(_) => internal_error("Failed to deactivate invite code"),
    }
}

#[axum::debug_handler]
pub async fn join_with_invite_code(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<JoinWithInviteCodeRequest>,
) -> Response {
    let invite_code = match InviteCode::find()
        .filter(invite_code::Column::Code.eq(&req.code))
        .filter(invite_code::Column::IsActive.eq(true))
        .one(&state.db)
        .await
    {
        Ok(Some(code)) => code,
        Ok(None) => return not_found("Invalid or inactive invite code"),
        Err(_) => return internal_error("Database error"),
    };

    if let Some(max_users) = invite_code.max_users {
        if invite_code.used_count >= max_users {
            return bad_request("Invite code has reached maximum users");
        }
    }

    if let Some(expires_at) = invite_code.expires_at {
        if Utc::now().naive_utc() > expires_at {
            return bad_request("Invite code has expired");
        }
    }

    let already_member = UserGroup::find()
        .filter(user_group::Column::UserId.eq(claims.sub))
        .filter(user_group::Column::GroupId.eq(invite_code.group_id))
        .count(&state.db)
        .await
        .unwrap_or(0) > 0;

    if already_member {
        return bad_request("Already a member of this group");
    }

    let now = Utc::now().naive_utc();
    let user_group = user_group::ActiveModel {
        user_id: Set(claims.sub),
        group_id: Set(invite_code.group_id),
        created_at: Set(now),
        ..Default::default()
    };

    let mut invite_code: invite_code::ActiveModel = invite_code.into();
    invite_code.used_count = Set(invite_code.used_count.as_ref() + 1);
    invite_code.updated_at = Set(now);

    match (user_group.insert(&state.db).await, invite_code.update(&state.db).await) {
        (Ok(ug), Ok(_)) => created(ug),
        _ => internal_error("Failed to join group"),
    }
}
