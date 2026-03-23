use rs_elib::*;

use axum::{
    routing::get,
    Router,
    extract::DefaultBodyLimit,
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::compression::CompressionLayer;
use tower_http::catch_panic::CatchPanicLayer;
use clap::Parser;
use tracing::warn;

async fn health_check() -> &'static str {
    "OK"
}


#[macro_export]
macro_rules! if_embed_static {
    ($app:expr) => {{
        #[cfg(feature = "embed_static")]
        {
            $app.fallback(axum::response::Html(include_str!("../static/index.min.html")))
        }
        
        #[cfg(not(feature = "embed_static"))]
        {
            let static_service = ServeDir::new("static")
                .not_found_service(ServeFile::new("static/index.html"));
            $app.fallback_service(static_service)
        }
    }};
}

const DEFAULT_UPLOAD_DIR: &str = "uploads";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_DB:&str = db::DEFAULT_DB_CONN_STR;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = DEFAULT_HOST)]
    host: String,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    #[arg(long, default_value = DEFAULT_UPLOAD_DIR)]
    upload_dir: String,

    #[arg(long, default_value = DEFAULT_DB)]
    db_conn_str: String,

    #[arg(long)]
    log_level: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 初始化日志
    logger::init_logger(args.log_level.as_deref());

    if std::env::var("JWT_SECRET").is_err() {
        warn!("JWT_SECRET环境变量未设置，可能导致安全风险！");
    }

    // 初始化数据库连接，默认SQLite数据库文件为elib.db
    let db = db::init_db(&args.db_conn_str).await?;

    // 上传目录，用于存储书籍文件
    let upload_dir = std::path::PathBuf::from(&args.upload_dir);
    
    // 格式解析器注册，用于解析特定格式的书籍文件
    let mut parser_register = format::FormatParserRegister::new();
    parser_register.register::<format::epub::EpubParser>("epub");
    parser_register.register::<format::txt::TxtParser>("txt");
    parser_register.register::<format::markdown::MarkdownParser>("markdown");
    
    // 全局状态
    let state = state::AppState::new(db, upload_dir, parser_register);

    // CORS跨域
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let compression = CompressionLayer::new()
        .gzip(true)
        .deflate(true)
        .br(true);

    // 获取注册的路由
    let public_routes = router::create_public_routes()
        .with_state(state.clone());

    let protected_routes = router::create_protected_routes()
        .with_state(state.clone())
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .layer(axum::middleware::from_fn(middleware::auth::auth));

    let api_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(|| async {
            (axum::http::StatusCode::NOT_FOUND, axum::Json(serde_json::json!({
                "error": "Not Found",
                "message": "The requested resource was not found"
            })))
        })
        .layer(compression.clone());

    let app = if_embed_static!(Router::new()
    .nest("/api", api_routes)
    .route("/health", get(health_check))
    .layer(compression)
    .layer(CatchPanicLayer::new())
    .layer(cors));

    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}