use crate::entity::{book, book_uploader, chapter};
use crate::{
    entity::{book_group, group, prelude::*, user_group},
    format::{ChapterParsed, FormatParser},
    state::AppState,
};
use axum::extract::Multipart;
use futures::TryStreamExt;
use futures::{Stream, stream};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, RelationTrait, Set,
};
use sha2::{Digest, Sha256};
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

// pub async fn calculate_file_hash(data: &[u8]) -> String {
//     let mut hasher = Sha256::new();
//     hasher.update(data);
//     let result = hasher.finalize();
//     format!("{:x}", result)
// }

pub async fn find_book_by_hash(
    db: &sea_orm::DatabaseConnection,
    hash: &str,
) -> Option<book::Model> {
    Book::find()
        .filter(book::Column::FileHash.eq(hash))
        .one(db)
        .await
        .ok()
        .flatten()
}

/// 管理员可以获取所有书籍
pub async fn manager_get_books_by_page(
    state: &AppState,
    page_size: u64,
    offset: u64,
) -> (Vec<BookModel>, u64) {
    let count = Book::find().count(&state.db).await.unwrap_or(0);
    let books = Book::find()
        .order_by_desc(book::Column::CreatedAt)
        .limit(page_size)
        .offset(offset)
        .all(&state.db)
        .await
        .unwrap_or_default();
    (books, count)
}

/// 贡献者可以获取自己建立的书籍和可以访问的书籍
pub async fn contributor_get_books_by_page(
    state: &AppState,
    user_id: i32,
    page_size: u64,
    offset: u64,
) -> (Vec<BookModel>, u64) {
    use sea_orm::{Condition, QuerySelect};

    let own_condition = Condition::any()
        .add(book::Column::CreatedBy.eq(Some(user_id)))
        .add(
            book::Column::Id.in_subquery(
                BookUploader::find()
                    .select_only()
                    .column(book_uploader::Column::BookId)
                    .filter(book_uploader::Column::UserId.eq(user_id))
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
            .filter(user_group::Column::UserId.eq(user_id))
            .into_query(),
    );

    let condition = Condition::any().add(own_condition).add(
        Condition::all()
            .add(accessible_condition)
            .add(book::Column::Status.eq("approved")),
    );

    let count = Book::find()
        .filter(condition.clone())
        .count(&state.db)
        .await
        .unwrap_or(0);

    let books = Book::find()
        .filter(condition)
        .order_by_desc(book::Column::CreatedAt)
        .limit(page_size)
        .offset(offset)
        .all(&state.db)
        .await
        .unwrap_or_default();

    (books, count)
}

pub async fn common_user_get_books_by_page(
    state: &AppState,
    user_id: i32,
    page_size: u64,
    offset: u64,
) -> (Vec<BookModel>, u64) {
    let count = Book::find()
            .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
            .join(JoinType::InnerJoin, book_group::Relation::Group.def())
            .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
            .filter(user_group::Column::UserId.eq(user_id))
            .filter(book::Column::Status.eq("approved"))
            .count(&state.db)
            .await
            .unwrap_or(0);

        let books = Book::find()
            .join(JoinType::InnerJoin, book::Relation::BookGroups.def())
            .join(JoinType::InnerJoin, book_group::Relation::Group.def())
            .join(JoinType::InnerJoin, group::Relation::UserGroups.def())
            .filter(user_group::Column::UserId.eq(user_id))
            .filter(book::Column::Status.eq("approved"))
            .order_by_desc(book::Column::CreatedAt)
            .limit(page_size)
            .offset(offset)
            .all(&state.db)
            .await
            .unwrap_or_default();

    (books, count)
}

async fn create_upload_dir(state: &AppState) -> std::io::Result<()> {
    if !state.upload_dir.exists() {
        tokio::fs::create_dir_all(&state.upload_dir).await?;
    }

    Ok(())
}

/// parser流包装器
fn chapter_stream(
    parser: Arc<Mutex<dyn FormatParser + Send + Sync>>,
    book_id: i32,
) -> impl Stream<Item = crate::format::Result<ChapterParsed>> {
    stream::unfold((), move |_| {
        let parser = parser.clone();
        async move {
            let mut lock = parser.lock().await;
            match lock.parse_chapters(book_id).await {
                Ok(chapter) => Some((Ok(chapter), ())),
                Err(crate::format::FormatParserError::EndOfChapters) => None,
                Err(e) => Some((Err(e), ())),
            }
        }
    })
}

pub async fn add_book(
    state: &AppState,
    user_id: i32,
    status: String,
    mut multipart: Multipart,
) -> Result<BookModel, &'static str> {
    let mut title = String::new();
    let mut author = None;
    let mut category = None;
    let mut isbn = None;
    let mut description = None;
    let mut file_path = String::new();
    let mut file_type = String::new();
    let mut file_hash = String::new();
    let mut file_size = 0i64;
    let mut has_file = false;

    create_upload_dir(state)
        .await
        .map_err(|_| "Failed to create dir")?;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            has_file = true;
            let filename = field.file_name().unwrap_or("book").to_string();
            let ext = StdPath::new(&filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let file_id = uuid::Uuid::new_v4().to_string();
            let save_path = state.upload_dir.join(format!("{}_{}", file_id, filename));

            let mut file = File::create(&save_path)
                .await
                .map_err(|_| "Failed to save file")?;
            let mut hasher = Sha256::new();

            while let Some(chunk) = field
                .chunk()
                .await
                .map_err(|_| "Failed to read file chunk")?
            {
                hasher.update(&chunk);
                file.write_all(&chunk)
                    .await
                    .map_err(|_| "Failed to write file")?;
            }

            file_hash = format!("{:x}", hasher.finalize());
            file_path = save_path.to_string_lossy().to_string();
            file_type = if ext == "md" || ext == "markdown" {
                "markdown".to_string()
            } else {
                ext
            };
            file_size = tokio::fs::metadata(&save_path)
                .await
                .map(|m| m.len() as i64)
                .unwrap_or(0);

            if let Some(existing_book) = find_book_by_hash(&state.db, &file_hash).await {
                add_book_uploader(&state.db, existing_book.id, user_id).await;
                return Ok(existing_book);
            }
        } else if name == "title" {
            let data = field.bytes().await.map_err(|_| "Failed to read title")?;
            title = String::from_utf8_lossy(&data).to_string();
        } else if name == "author" {
            let data = field.bytes().await.map_err(|_| "Failed to read author")?;
            author = Some(String::from_utf8_lossy(&data).to_string());
        } else if name == "category" {
            let data = field.bytes().await.map_err(|_| "Failed to read category")?;
            let category_str = String::from_utf8_lossy(&data).to_string();
            if !category_str.is_empty() {
                category = Some(category_str);
            }
        } else if name == "isbn" {
            let data = field.bytes().await.map_err(|_| "Failed to read isbn")?;
            let isbn_str = String::from_utf8_lossy(&data).to_string();
            if !isbn_str.is_empty() {
                isbn = Some(isbn_str);
            }
        } else if name == "description" {
            let data = field.bytes().await.map_err(|_| "Failed to read description")?;
            let description_str = String::from_utf8_lossy(&data).to_string();
            if !description_str.is_empty() {
                description = Some(description_str);
            }
        }
    }

    if !has_file {
        return Err("No file uploaded");
    }
    if title.is_empty() {
        return Err("Title required");
    }
    if file_type.is_empty() {
        return Err("File type not found");
    }

    // 写入 book
    let now = chrono::Utc::now().naive_utc();
    let book = book::ActiveModel {
        title: Set(title),
        author: Set(author),
        category: Set(category),
        isbn: Set(isbn),
        description: Set(description),
        file_path: Set(file_path.clone()),
        file_type: Set(file_type.clone()),
        file_size: Set(file_size),
        file_hash: Set(Some(file_hash)),
        created_by: Set(Some(user_id)),
        created_at: Set(now),
        updated_at: Set(now),
        status: Set(status),
        ..Default::default()
    };

    let book = book
        .insert(&state.db)
        .await
        .map_err(|_| "Failed to insert book")?;

    // 异步解析章节 (stream)
    // 使用 state.parser_register.get() 自动根据格式获取解析器
    let parser = state.parser_register.get(&file_type, &file_path);

    if let Some(parser) = parser {
        let stream = chapter_stream(parser.clone(), book.id);
        tokio::pin!(stream); // Pin 避免每次 await 移动 Stream

        while let Some(ch_res) = stream
            .try_next()
            .await
            .map_err(|_| "Failed to parse chapter")?
        {
            let chp = chapter::ActiveModel {
                book_id: Set(book.id),
                chapter_number: Set(ch_res.chapter_number),
                title: Set(ch_res.title),
                content: Set(ch_res.content),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

            chp.insert(&state.db)
                .await
                .map_err(|_| "Failed to insert chapter")?;
        }
    }

    Ok(book)
}

pub async fn add_book_uploader(db: &sea_orm::DatabaseConnection, book_id: i32, user_id: i32) {
    let existing = BookUploader::find()
        .filter(book_uploader::Column::BookId.eq(book_id))
        .filter(book_uploader::Column::UserId.eq(user_id))
        .one(db)
        .await;

    if let Ok(None) = existing {
        let now = chrono::Utc::now().naive_utc();
        let uploader = book_uploader::ActiveModel {
            book_id: Set(book_id),
            user_id: Set(user_id),
            created_at: Set(now),
            ..Default::default()
        };
        let _ = uploader.insert(db).await;
    }
}
