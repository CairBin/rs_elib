mod request;
mod dto;

use request::*;
use dto::CommentDto;

use crate::utils::api::*;
use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};

use sea_orm::{EntityTrait, Set, QueryFilter, ColumnTrait, ActiveModelTrait, QueryOrder};
use crate::entity::prelude::*;
use crate::state::AppState;
use crate::middleware::auth::AuthMiddleware;
use chrono::Utc;
use crate::entity::comment;
use crate::service::settings as settings_service;

// 辅助函数：将评论模型转换为DTO
async fn comment_to_dto(state: &AppState, comment: comment::Model) -> CommentDto {
    // 查询用户信息获取用户名
    let username = match User::find_by_id(comment.user_id).one(&state.db).await {
        Ok(Some(user)) => user.username,
        _ => "未知用户".to_string(),
    };
    
    CommentDto {
        id: comment.id,
        book_id: comment.book_id,
        chapter_id: comment.chapter_id,
        user_id: comment.user_id,
        username,
        content: comment.content,
        status: comment.status,
        created_at: comment.created_at,
        updated_at: comment.updated_at,
    }
}

// 辅助函数：将评论模型列表转换为DTO列表
async fn comments_to_dtos(state: &AppState, comments: Vec<comment::Model>) -> Vec<CommentDto> {
    let mut dtos = Vec::with_capacity(comments.len());
    for comment in comments {
        dtos.push(comment_to_dto(state, comment).await);
    }
    dtos
}


#[axum::debug_handler]
pub async fn create_comment(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<CreateCommentRequest>,
) -> Response {
    let allow_comments = settings_service::is_comments_allowed(&state).await;
    
    if !allow_comments && claims.role != "admin" && claims.role != "root" {
        return forbidden("Comments are disabled");
    }

    if req.book_id.is_none() && req.chapter_id.is_none() {
        return bad_request("Either book_id or chapter_id is required");
    }

    if req.book_id.is_some() && req.chapter_id.is_some() {
        return bad_request("Cannot comment on both book and chapter");
    }

    if let Some(book_id) = req.book_id {
        let book = match Book::find_by_id(book_id).one(&state.db).await {
            Ok(Some(b)) => b,
            Ok(None) => return not_found("Book not found"),
            Err(_) => return internal_error("Database error"),
        };

        if book.status != "approved" && claims.role != "admin" && claims.role != "root" {
            return forbidden("Cannot comment on pending book");
        }
    }

    if let Some(chapter_id) = req.chapter_id {
        let chapter = match Chapter::find_by_id(chapter_id).one(&state.db).await {
            Ok(Some(c)) => c,
            Ok(None) => return not_found("Chapter not found"),
            Err(_) => return internal_error("Database error"),
        };

        let book = match Book::find_by_id(chapter.book_id).one(&state.db).await {
            Ok(Some(b)) => b,
            Ok(None) => return not_found("Book not found"),
            Err(_) => return internal_error("Database error"),
        };

        if book.status != "approved" && claims.role != "admin" && claims.role != "root" {
            return forbidden("Cannot comment on chapter of pending book");
        }
    }

    let enable_comment_review = settings_service::is_comment_review_enabled(&state).await;
    
    let status = if claims.role == "admin" || claims.role == "root" || !enable_comment_review {
        "approved".to_string()
    } else {
        "pending".to_string()
    };

    let now = Utc::now().naive_utc();
    let comment = CommentActiveModel {
        book_id: Set(req.book_id),
        chapter_id: Set(req.chapter_id),
        user_id: Set(claims.sub),
        content: Set(req.content),
        status: Set(status),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match comment.insert(&state.db).await {
        Ok(comment) => {
            let comment_dto = comment_to_dto(&state, comment).await;
            created(comment_dto)
        },
        Err(_) => internal_error("Failed to create comment"),
    }
}

#[axum::debug_handler]
pub async fn get_book_comments(
    State(state): State<AppState>,
    Path(book_id): Path<i32>,
    crate::middleware::auth::OptionalAuthMiddleware(claims): crate::middleware::auth::OptionalAuthMiddleware,
) -> Response {
    let mut query = Comment::find()
        .filter(comment::Column::BookId.eq(book_id));
    
    if let Some(auth) = claims {
        if auth.role != "admin" && auth.role != "root" {
            query = query.filter(comment::Column::Status.eq("approved"));
        }
    } else {
        query = query.filter(comment::Column::Status.eq("approved"));
    }
    
    let comments = query
        .order_by_desc(comment::Column::CreatedAt)
        .all(&state.db)
        .await;

    match comments {
        Ok(comments) => {
            let comment_dtos = comments_to_dtos(&state, comments).await;
            success(comment_dtos)
        },
        Err(_) => internal_error("Database error"),
    }
}

#[axum::debug_handler]
pub async fn get_chapter_comments(
    State(state): State<AppState>,
    Path(chapter_id): Path<i32>,
    crate::middleware::auth::OptionalAuthMiddleware(claims): crate::middleware::auth::OptionalAuthMiddleware,
) -> Response {
    let mut query = Comment::find()
        .filter(comment::Column::ChapterId.eq(chapter_id));
    
    if let Some(auth) = claims {
        if auth.role != "admin" && auth.role != "root" {
            query = query.filter(comment::Column::Status.eq("approved"));
        }
    } else {
        query = query.filter(comment::Column::Status.eq("approved"));
    }
    
    let comments = query
        .order_by_desc(comment::Column::CreatedAt)
        .all(&state.db)
        .await;

    match comments {
        Ok(comments) => {
            let comment_dtos = comments_to_dtos(&state, comments).await;
            success(comment_dtos)
        },
        Err(_) => internal_error("Database error"),
    }
}

#[axum::debug_handler]
pub async fn delete_comment(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let comment = match Comment::find_by_id(id).one(&state.db).await {
        Ok(Some(c)) => c,
        Ok(None) => return not_found("Comment not found"),
        Err(_) => return internal_error("Database error"),
    };

    if comment.user_id != claims.sub && claims.role != "admin" && claims.role != "root" {
        return forbidden("Access denied");
    }

    match Comment::delete_by_id(id).exec(&state.db).await {
        Ok(_) => success("Comment deleted"),
        Err(_) => internal_error("Failed to delete comment"),
    }
}

#[axum::debug_handler]
pub async fn get_pending_comments(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    if claims.role != "admin" && claims.role != "root" {
        return forbidden("Access denied");
    }

    let comments = Comment::find()
        .filter(comment::Column::Status.eq("pending"))
        .order_by_desc(comment::Column::CreatedAt)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let comment_dtos = comments_to_dtos(&state, comments).await;
    success(comment_dtos)
}



#[axum::debug_handler]
pub async fn review_comment(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<ReviewCommentRequest>,
) -> Response {
    if claims.role != "admin" && claims.role != "root" {
        return forbidden("Access denied");
    }

    if req.status != "approved" && req.status != "rejected" {
        return bad_request("Invalid status");
    }

    let comment = match Comment::find_by_id(id).one(&state.db).await {
        Ok(Some(c)) => c,
        Ok(None) => return not_found("Comment not found"),
        Err(_) => return internal_error("Database error"),
    };

    let now = Utc::now().naive_utc();
    let mut comment: comment::ActiveModel = comment.into();
    comment.status = Set(req.status);
    comment.updated_at = Set(now);

    match comment.update(&state.db).await {
        Ok(comment) => {
            let comment_dto = comment_to_dto(&state, comment).await;
            success(comment_dto)
        },
        Err(_) => internal_error("Failed to update comment"),
    }
}
