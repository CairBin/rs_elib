mod request;
mod response;

use crate::service::{book as book_service, settings as settings_service};
use crate::utils::api::*;
use crate::{
    entity::{book, book_group, book_uploader, chapter, group, prelude::*, user_group},
    middleware::auth::AuthMiddleware,
    state::AppState,
};
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use mime_guess::from_path;
use request::*;
use response::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, RelationTrait, Set,
};
use std::path::Path as StdPath;

#[axum::debug_handler]
pub async fn get_books(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    Query(query): Query<PaginationQuery>,
) -> Response {
    tracing::info!("Get books request: user_id={}, role={}, page={}", claims.sub, claims.role, query.get_page());
    
    let page = query.get_page();
    let page_size = query.get_page_size();
    let offset = (page - 1) * page_size;

    let (books, count) = if claims.role == "root" || claims.role == "admin" {
        book_service::manager_get_books_by_page(&state, page_size, offset).await
    } else if claims.role == "contributor" {
        book_service::contributor_get_books_by_page(&state, claims.sub, page_size, offset).await
    } else {
        book_service::common_user_get_books_by_page(&state, claims.sub, page_size, offset).await
    };

    tracing::debug!("Books retrieved: count={}", count);
    let page_response = PageResponse::new(books, page, page_size, count);

    success(page_response)
}

#[axum::debug_handler]
pub async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Database error"),
    };

    if claims.role != "root" && claims.role != "admin" {
        let is_owner = book.created_by == Some(claims.sub);

        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0)
            > 0;

        let is_owner_or_uploader = is_owner || is_uploader;

        if !is_owner_or_uploader {
            if book.status != "approved" {
                return forbidden("Access denied");
            }

            let has_access = Book::find()
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(book::Column::Id.eq(id))
                .count(&state.db)
                .await
                .unwrap_or(0)
                > 0;

            if !has_access {
                return forbidden("Access denied");
            }
        }
    }

    success(book)
}

#[axum::debug_handler]
pub async fn create_book(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    multipart: Multipart,
) -> Response {
    tracing::info!("Create book request: user_id={}, role={}", claims.sub, claims.role);
    
    if claims.role != "contributor" && claims.role != "admin" && claims.role != "root" {
        tracing::warn!("Create book denied: insufficient permissions - user_id={}", claims.sub);
        return forbidden("Access denied");
    }

    let enable_upload_review = settings_service::is_upload_review_enabled(&state).await;
    let status = if claims.role == "admin" || claims.role == "root" || !enable_upload_review {
        "approved".to_string()
    } else {
        "pending".to_string()
    };
    let status_clone = status.clone();

    tracing::debug!("Book will be created with status: {}", status);

    match book_service::add_book(&state, claims.sub, status, multipart).await {
        Ok(book) => {
            tracing::info!("Book created successfully: book_id={}, title={}, status={}", book.id, book.title, status_clone);
            book_service::add_book_uploader(&state.db, book.id, claims.sub).await;
            success(book)
        }
        Err(e) => {
            tracing::error!("Failed to create book: user_id={}, error={:?}", claims.sub, e);
            internal_error(e)
        }
    }
}

#[axum::debug_handler]
pub async fn update_book(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<UpdateBookRequest>,
) -> Response {
    tracing::info!("Update book request: user_id={}, role={}, book_id={}", claims.sub, claims.role, id);
    
    if claims.role != "admin" && claims.role != "root" && claims.role != "contributor" {
        tracing::warn!("Update book denied: insufficient permissions - user_id={}, book_id={}", claims.sub, id);
        return forbidden("Access denied");
    }

    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => {
            tracing::warn!("Update book failed: book not found - book_id={}", id);
            return not_found("Book not found");
        }
        Err(e) => {
            tracing::error!("Update book database error: book_id={}, error={:?}", id, e);
            return internal_error("Db error");
        }
    };

    if claims.role != "admin" && claims.role != "root" {
        // 查看是否允许贡献者更新
        let allow_uploader_edit = settings_service::is_uploader_edit_allowed(&state).await;

        if !allow_uploader_edit {
            tracing::warn!("Update book denied: uploader edit disabled - user_id={}, book_id={}", claims.sub, id);
            return forbidden("Access denied");
        }

        let is_owner = book.created_by == Some(claims.sub);
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0)
            > 0;

        if !is_owner && !is_uploader {
            tracing::warn!("Update book denied: not owner or uploader - user_id={}, book_id={}", claims.sub, id);
            return forbidden("Access denied");
        }
    }

    let now = chrono::Utc::now().naive_utc();
    let mut book: book::ActiveModel = book.into();

    if let Some(title) = req.title {
        book.title = Set(title);
    }
    if let Some(author) = req.author {
        book.author = Set(Some(author));
    }
    if let Some(description) = req.description {
        book.description = Set(Some(description));
    }
    if let Some(isbn) = req.isbn {
        book.isbn = Set(Some(isbn));
    }
    if let Some(category) = req.category {
        book.category = Set(Some(category));
    }
    book.updated_at = Set(now);

    match book.update(&state.db).await {
        Ok(book) => {
            tracing::info!("Book updated successfully: book_id={}, title={}", book.id, book.title);
            success(book)
        }
        Err(e) => {
            tracing::error!("Failed to update book: book_id={}, error={:?}", id, e);
            internal_error("Failed to update book")
        }
    }
}

#[axum::debug_handler]
pub async fn delete_book(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    tracing::info!("Delete book request: user_id={}, role={}, book_id={}", claims.sub, claims.role, id);
    
    if claims.role != "admin" && claims.role != "root" && claims.role != "contributor" {
        tracing::warn!("Delete book denied: insufficient permissions - user_id={}, book_id={}", claims.sub, id);
        return forbidden("Access denied");
    }

    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => {
            tracing::warn!("Delete book failed: book not found - book_id={}", id);
            return not_found("Book not found");
        }
        Err(e) => {
            tracing::error!("Delete book database error: book_id={}, error={:?}", id, e);
            return internal_error("Db error");
        }
    };

    if claims.role != "admin" && claims.role != "root" {
        // 查看是否允许贡献者删除
        let allow_uploader_delete = settings_service::is_uploader_delete_allowed(&state).await;

        if !allow_uploader_delete {
            tracing::warn!("Delete book denied: uploader delete disabled - user_id={}, book_id={}", claims.sub, id);
            return forbidden("Access denied");
        }

        let is_owner = book.created_by == Some(claims.sub);
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0)
            > 0;

        if !is_owner && !is_uploader {
            tracing::warn!("Delete book denied: not owner or uploader - user_id={}, book_id={}", claims.sub, id);
            return forbidden("Access denied");
        }
    }

    tracing::info!("Deleting book file: {}", book.file_path);
    let _ = tokio::fs::remove_file(&book.file_path).await;

    let resources_dir = StdPath::new("uploads").join(format!("book_{}_resources", id));
    if resources_dir.exists() {
        tracing::info!("Deleting book resources directory: {:?}", resources_dir);
        let _ = tokio::fs::remove_dir_all(&resources_dir).await;
    }

    match Book::delete_by_id(id).exec(&state.db).await {
        Ok(_) => {
            tracing::info!("Book deleted successfully: book_id={}, title={}", id, book.title);
            success("Book deleted")
        }
        Err(e) => {
            tracing::error!("Failed to delete book from database: book_id={}, error={:?}", id, e);
            internal_error("Failed to delete book")
        }
    }
}

/// 提供图书资源（如图片等）
#[axum::debug_handler]
pub async fn serve_book_resource(
    State(state): State<AppState>,
    Path((book_id, filename)): Path<(i32, String)>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let book = match Book::find_by_id(book_id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Db error"),
    };

    if claims.role != "root" && claims.role != "admin" {
        // 如果不是管理员，检查是否有访问的权限

        let is_owner = book.created_by == Some(claims.sub);
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(book_id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0)
            > 0;

        let is_owner_or_uploader = is_owner || is_uploader;

        if !is_owner_or_uploader {
            if book.status != "approved" {
                return forbidden("Access denied");
            }

            let has_access = Book::find()
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(book::Column::Id.eq(book_id))
                .count(&state.db)
                .await
                .unwrap_or(0)
                > 0;
            if !has_access {
                return forbidden("Access denied");
            }
        }
    }

    let resource_path = StdPath::new("uploads")
        .join(format!("book_{}_resources", book_id))
        .join(&filename);

    if !resource_path.exists() {
        return not_found("Resource not found");
    }

    let mime_type = from_path(&resource_path)
        .first_or_octet_stream()
        .to_string();

    match tokio::fs::read(&resource_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type)
            .body(Body::from(content))
            .unwrap()
            .into_response(),
        Err(_) => internal_error("Failed to read resource"),
    }
}

#[axum::debug_handler]
pub async fn search_books(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<SearchBooksWithPagination>,
) -> Response {
    let page = req.page.unwrap_or(1).max(1);
    let page_size = req.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    let mut query = Book::find();

    if let Some(keyword) = &req.keyword {
        query = query.filter(
            book::Column::Title
                .contains(keyword)
                .or(book::Column::Author.contains(keyword))
                .or(book::Column::Isbn.contains(keyword)),
        );
    }

    if let Some(category) = &req.category {
        query = query.filter(book::Column::Category.eq(category));
    }

    if let Some(format) = &req.format {
        query = query.filter(book::Column::FileType.eq(format));
    }

    let (books, total) = if claims.role == "admin" || claims.role == "root" {
        let count = query.clone().count(&state.db).await.unwrap_or(0);
        let books = query
            .order_by_desc(book::Column::CreatedAt)
            .limit(page_size)
            .offset(offset)
            .all(&state.db)
            .await
            .unwrap_or_default();
        (books, count)
    } else if claims.role == "contributor" {
        use sea_orm::Condition;

        let own_condition = Condition::any()
            .add(book::Column::CreatedBy.eq(Some(claims.sub)))
            .add(
                book::Column::Id.in_subquery(
                    BookUploader::find()
                        .select_only()
                        .column(book_uploader::Column::BookId)
                        .filter(book_uploader::Column::UserId.eq(claims.sub))
                        .into_query(),
                ),
            );

        let accessible_condition = book::Column::Id.in_subquery(
            Book::find()
                .select_only()
                .column(book::Column::Id)
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .into_query(),
        );

        let condition = Condition::any().add(own_condition).add(
            Condition::all()
                .add(accessible_condition)
                .add(book::Column::Status.eq("approved")),
        );

        let final_query = query.filter(condition);
        let count = final_query.clone().count(&state.db).await.unwrap_or(0);
        let books = final_query
            .order_by_desc(book::Column::CreatedAt)
            .limit(page_size)
            .offset(offset)
            .all(&state.db)
            .await
            .unwrap_or_default();

        (books, count)
    } else {
        let final_query = query
            .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
            .join(JoinType::InnerJoin, book_group::Relation::Group.def())
            .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
            .filter(user_group::Column::UserId.eq(claims.sub))
            .filter(book::Column::Status.eq("approved"));

        let count = final_query.clone().count(&state.db).await.unwrap_or(0);
        let books = final_query
            .order_by_desc(book::Column::CreatedAt)
            .limit(page_size)
            .offset(offset)
            .all(&state.db)
            .await
            .unwrap_or_default();
        (books, count)
    };

    let page_response = PageResponse::new(books, page, page_size, total);
    success(page_response)
}

/// 获取分类
#[axum::debug_handler]
pub async fn get_categories(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    use sea_orm::{FromQueryResult, QuerySelect};

    #[derive(FromQueryResult, serde::Serialize, serde::Deserialize)]
    struct CategoryResult {
        category: Option<String>,
    }

    let categories = if claims.role == "admin" || claims.role == "root" {
        Book::find()
            .select_only()
            .column(book::Column::Category)
            .distinct()
            .into_model::<CategoryResult>()
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|c| c.category)
            .collect::<Vec<_>>()
    } else if claims.role == "contributor" {
        use sea_orm::Condition;

        let own_condition = Condition::any()
            .add(book::Column::CreatedBy.eq(Some(claims.sub)))
            .add(
                book::Column::Id.in_subquery(
                    BookUploader::find()
                        .select_only()
                        .column(book_uploader::Column::BookId)
                        .filter(book_uploader::Column::UserId.eq(claims.sub))
                        .into_query(),
                ),
            );

        let accessible_condition = book::Column::Id.in_subquery(
            Book::find()
                .select_only()
                .column(book::Column::Id)
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .into_query(),
        );

        let condition = Condition::any().add(own_condition).add(
            Condition::all()
                .add(accessible_condition)
                .add(book::Column::Status.eq("approved")),
        );

        Book::find()
            .filter(condition)
            .select_only()
            .column(book::Column::Category)
            .distinct()
            .into_model::<CategoryResult>()
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|c| c.category)
            .collect::<Vec<_>>()
    } else {
        Book::find()
            .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
            .join(JoinType::InnerJoin, book_group::Relation::Group.def())
            .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
            .filter(user_group::Column::UserId.eq(claims.sub))
            .filter(book::Column::Status.eq("approved"))
            .select_only()
            .column(book::Column::Category)
            .distinct()
            .into_model::<CategoryResult>()
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|c| c.category)
            .collect::<Vec<_>>()
    };

    success(categories)
}




#[axum::debug_handler]
pub async fn read_book(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Db error"),
    };

    if claims.role != "admin" && claims.role != "root" {
        let is_owner = book.created_by == Some(claims.sub);
        
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0) > 0;
        
        let is_owner_or_uploader = is_owner || is_uploader;
        
        if !is_owner_or_uploader {
            if book.status != "approved" {
                return forbidden("Access denied");
            }
            
            let has_access = Book::find()
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(book::Column::Id.eq(id))
                .count(&state.db)
                .await
                .unwrap_or(0) > 0;

            if !has_access {
                return forbidden("Access denied");
            }
        }
    }

    let chapter_count = Chapter::find()
        .filter(chapter::Column::BookId.eq(id))
        .count(&state.db)
        .await
        .unwrap_or(0);

    let file_type = book.file_type.to_lowercase();
    
    if chapter_count > 0 {
        success(serde_json::json!({
            "book": book,
            "has_chapters": true,
            "supported": true
        }))
    } else if file_type == "txt" {
        let file_path = StdPath::new(&book.file_path);
        if !file_path.exists() {
            return not_found("Book file not found");
        }

        let content = match tokio::fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(_) => {
                return internal_error("Failed to read book file");
            }
        };

        let chunks: Vec<&str> = content.split("\n\n").filter(|s| !s.is_empty()).collect();

        success(serde_json::json!({
            "book": book,
            "total_chunks": chunks.len(),
            "content": content,
            "has_chapters": false,
            "supported": true
        }))
    } else {
        success(serde_json::json!({
            "book": book,
            "has_chapters": false,
            "total_chunks": 1,
            "content": format!("抱歉，{} 格式的文件暂时不支持在线阅读。\n\n支持的格式：TXT, EPUB", file_type.to_uppercase()),
            "supported": false
        }))
    }
}



#[axum::debug_handler]
pub async fn get_book_chunk(
    State(state): State<AppState>,
    Path((id, chunk_index)): Path<(i32, usize)>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Database error"),
    };

    if claims.role != "admin" && claims.role != "root" {
        let is_owner = book.created_by == Some(claims.sub);
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0) > 0;
        
        let is_owner_or_uploader = is_owner || is_uploader;
        
        if !is_owner_or_uploader {
            if book.status != "approved" {
                return forbidden("Access denied");
            }
            
            let has_access = Book::find()
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(book::Column::Id.eq(id))
                .count(&state.db)
                .await
                .unwrap_or(0) > 0;

            if !has_access {
                return forbidden("Access denied");
            }
        }
    }

    let file_path = StdPath::new(&book.file_path);
    if !file_path.exists() {
        return not_found("Book file not found");
    }

    let file_type = book.file_type.to_lowercase();
    
    if file_type != "txt" {
        return success(serde_json::json!({
            "chunk_index": chunk_index,
            "total_chunks": 1,
            "content": format!("抱歉，{} 格式的文件暂时不支持在线阅读。\n\n支持的格式：TXT", file_type.to_uppercase())
        }));
    }

    let content = match tokio::fs::read_to_string(file_path).await {
        Ok(content) => content,
        Err(_) => {
            return internal_error("Failed to read book file");
        }
    };

    let chunks: Vec<&str> = content.split("\n\n").filter(|s| !s.is_empty()).collect();
    
    if chunk_index >= chunks.len() {
        return bad_request("Invalid chunk index");
    }

    success(serde_json::json!({
        "chunk_index": chunk_index,
        "total_chunks": chunks.len(),
        "content": chunks[chunk_index]
    }))
}

#[axum::debug_handler]
pub async fn serve_book_file(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Database error"),
    };

    if claims.role != "admin" && claims.role != "root" {
        let is_owner = book.created_by == Some(claims.sub);
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0) > 0;
        
        let is_owner_or_uploader = is_owner || is_uploader;
        
        if !is_owner_or_uploader {
            if book.status != "approved" {
                return forbidden("Access denied");
            }
            
            let has_access = Book::find()
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(book::Column::Id.eq(id))
                .count(&state.db)
                .await
                .unwrap_or(0) > 0;

            if !has_access {
                return forbidden("Access denied");
            }
        }
    }

    let file_path = StdPath::new(&book.file_path);
    if !file_path.exists() {
        return not_found("Book file not found");
    }

    let mime_type = from_path(file_path)
        .first_or_octet_stream()
        .to_string();

    match tokio::fs::read(file_path).await {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", book.title))
                .body(Body::from(content))
                .unwrap()
                .into_response()
        }
        Err(_) => internal_error("Failed to read file"),
    }
}

#[axum::debug_handler]
pub async fn serve_book_file_public(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Database error"),
    };

    let file_path = StdPath::new(&book.file_path);
    if !file_path.exists() {
        return not_found("Book file not found");
    }

    let mime_type = from_path(file_path)
        .first_or_octet_stream()
        .to_string();

    match tokio::fs::read(file_path).await {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", book.title))
                .body(Body::from(content))
                .unwrap()
                .into_response()
        }
        Err(_) => internal_error("Failed to read file"),
    }
}



#[axum::debug_handler]
pub async fn get_chapters(
    State(state): State<AppState>,
    Path(book_id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let book = match Book::find_by_id(book_id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Database error"),
    };

    if claims.role != "admin" && claims.role != "root" {
        let is_owner = book.created_by == Some(claims.sub);
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(book_id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0) > 0;
        
        let is_owner_or_uploader = is_owner || is_uploader;
        
        if !is_owner_or_uploader {
            if book.status != "approved" {
                return forbidden("Access denied");
            }
            
            let has_access = Book::find()
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(book::Column::Id.eq(book_id))
                .count(&state.db)
                .await
                .unwrap_or(0) > 0;

            if !has_access {
                return forbidden("Access denied");
            }
        }
    }

    let chapters = Chapter::find()
        .filter(chapter::Column::BookId.eq(book_id))
        .order_by_asc(chapter::Column::ChapterNumber)
        .all(&state.db)
        .await
        .unwrap_or_default();

    success(chapters)
}

#[axum::debug_handler]
pub async fn get_chapter(
    State(state): State<AppState>,
    Path((book_id, chapter_number)): Path<(i32, i32)>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    let book = match Book::find_by_id(book_id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Database error"),
    };

    if claims.role != "admin" && claims.role != "root" {
        let is_owner = book.created_by == Some(claims.sub);
        let is_uploader = BookUploader::find()
            .filter(book_uploader::Column::BookId.eq(book_id))
            .filter(book_uploader::Column::UserId.eq(claims.sub))
            .count(&state.db)
            .await
            .unwrap_or(0) > 0;
        
        let is_owner_or_uploader = is_owner || is_uploader;
        
        if !is_owner_or_uploader {
            if book.status != "approved" {
                return forbidden("Access denied");
            }
            
            let has_access = Book::find()
                .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
                .join(JoinType::InnerJoin, book_group::Relation::Group.def())
                .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
                .filter(user_group::Column::UserId.eq(claims.sub))
                .filter(book::Column::Id.eq(book_id))
                .count(&state.db)
                .await
                .unwrap_or(0) > 0;

            if !has_access {
                return forbidden("Access denied");
            }
        }
    }

    let chapter = Chapter::find()
        .filter(chapter::Column::BookId.eq(book_id))
        .filter(chapter::Column::ChapterNumber.eq(chapter_number))
        .one(&state.db)
        .await;

    match chapter {
        Ok(Some(chapter)) => success(chapter),
        Ok(None) => not_found("Chapter not found"),
        Err(_) => internal_error("Database error"),
    }
}

#[axum::debug_handler]
pub async fn get_pending_books(
    State(state): State<AppState>,
    AuthMiddleware(claims): AuthMiddleware,
) -> Response {
    if claims.role != "admin" && claims.role != "root" {
        return forbidden("Access denied");
    }

    let books = Book::find()
        .filter(book::Column::Status.eq("pending"))
        .order_by_desc(book::Column::CreatedAt)
        .all(&state.db)
        .await
        .unwrap_or_default();

    success(books)
}



#[axum::debug_handler]
pub async fn review_book(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    AuthMiddleware(claims): AuthMiddleware,
    Json(req): Json<ReviewBookRequest>,
) -> Response {
    if claims.role != "admin" && claims.role != "root" {
        return forbidden("Access denied");
    }

    if req.status != "approved" && req.status != "rejected" {
        return bad_request("Invalid status");
    }

    let book = match Book::find_by_id(id).one(&state.db).await {
        Ok(Some(book)) => book,
        Ok(None) => return not_found("Book not found"),
        Err(_) => return internal_error("Database error"),
    };

    let now = chrono::Utc::now().naive_utc();
    let mut book: book::ActiveModel = book.into();
    book.status = Set(req.status);
    book.updated_at = Set(now);

    match book.update(&state.db).await {
        Ok(book) => success(book),
        Err(_) => internal_error("Failed to update book"),
    }
}
