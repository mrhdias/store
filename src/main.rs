//
// Last Modified: 2024-08-17 10:03:11
//
// References:
// https://dev.to/krowemoh/a-web-app-in-rust-02-templates-5do1
// https://gist.github.com/jeremychone/34d1e3daffc38eb602b1a9ab21298d10
// https://matam-kirankumar.medium.com/row-to-jsonb-in-postgresql-6c46eab5ebd3
// https://dev.to/carlosm27/creating-a-rest-api-with-axum-sqlx-rust-381d
// https://bitfieldconsulting.com/posts/rust-errors-option-result?ref=dailydev
// https://www.shuttle.rs/blog/2022/08/11/authentication-tutorial
// https://docs.tvix.dev/rust/axum/middleware/fn.from_extractor.html ***
// https://github.com/AscendingCreations/AxumSessionAuth
// https://docs.rs/axum/latest/axum/struct.Json.html
// https://www.getzola.org/themes/
// https://developer.mozilla.org/en-US/docs/Glossary/MVC
//

mod setup;
mod models;
mod controllers;
mod types;
mod utils;
mod notifications;

use time::Duration;
use std::path::{
    Path,
    PathBuf
};

use axum::{
    extract::{Extension, Form, Request},
    http::{header::LOCATION, HeaderMap, HeaderValue, StatusCode},
    middleware::from_extractor,
    response::Html,
    routing::{get, post},
    Router,
    ServiceExt,
};


use tower::Layer;
use tower_http::{
    services::ServeDir,
    normalize_path::NormalizePathLayer,
};

use tower_sessions::{
    Expiry,
    SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;

use tera::{
    Tera,
    Context
};

use sqlx::{
    postgres::{
        PgConnectOptions,
        PgPoolOptions,
    },
    Executor,
    Pool,
    Postgres,
    Error,
};

use serde::{
    Deserialize,
    Serialize
};

const APP_NAME: &str = "store";
const DEFAULT_CONFIG_FILE: &str = "config/store.ini";

use clap::Parser;

#[derive(Parser)]
#[command(name = APP_NAME)]
// #[command(author = "Author Name")]
#[command(version = "0.0.1")]
#[command(about = "E-Commerce Plataform", long_about = None)]
struct Cli {
    #[arg(short = 'c', long = "config", default_value = DEFAULT_CONFIG_FILE)]
    config_file: String,

    #[arg(short = 'p', long = "port", default_value_t = 8080)]
    port: u16,
}

#[derive(Debug, Deserialize)]
struct LoginForm {
    user: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConf {
    host: String,
    user: String,
    password: String,
    name: String,
    max_connections: u32,
}

async fn autentication(
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>,
    Form(payload): Form<LoginForm>,
) -> (StatusCode, HeaderMap, Html<String>) {

    let user = models::users::Users::new(pool.clone());

    let mut data = Context::new();

    match user.cardentials(&payload.user).await {
        Ok(user) => {
            if payload.password == user.password {
                // Create session here
                let tokens_manager = models::tokens::Tokens::new(pool.clone());
                let token = match tokens_manager.add(&user.user_id).await {
                    Ok(token) => token,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), "Failed to generate token.".to_string().into());
                    }
                };
                println!("TOKEN: {}", token);
                let mut headers = HeaderMap::new();
                headers.insert(axum::http::header::SET_COOKIE, format!("token={}; Path=/; HttpOnly", token).parse().unwrap());

                if user.role.is_admin() {
                    // data.insert("partial", "dashboard");
                    // let rendered = tera.render("admin/admin.html", &data).unwrap();
                    // return (StatusCode::OK, headers, Html(rendered));
                    headers.insert(LOCATION, HeaderValue::from_str("/admin").unwrap());
                    return (StatusCode::FOUND, headers, Html("".to_string()));
                } else {
                    return (StatusCode::OK, headers, Html("<h1>User Login Successful</h1>".to_string()));
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return (StatusCode::NOT_FOUND, HeaderMap::new(), "User not found!".to_string().into());
        }
    };

    data.insert("alert", "Invalid username or password");

    let rendered = tera.render("login.html", &data).unwrap();
    // Html(rendered)
    (StatusCode::OK, HeaderMap::new(), Html(rendered))
}

async fn is_database_empty(pool: &Pool<Postgres>) -> Result<bool, Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
            FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_type = 'BASE TABLE';
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.0 == 0)
}

#[tokio::main]
async fn main() {

    let args = Cli::parse();

    // println!("Config file: {}", args.config_file);
    // println!("Port: {}", args.port);

    if args.config_file == DEFAULT_CONFIG_FILE {
        let config_dir: PathBuf = Path::new(".").join("config");
        if !config_dir.exists() {
            std::fs::create_dir(&config_dir)
                .expect("Failed to create config directory");
        }
    }

    let config = setup::Config::new(args.config_file.into());

    let settings= match config.load() {
        Ok(conf) => conf,
        Err(e) => panic!("Error loading database configuration: {}", e),
    };

    let options = PgConnectOptions::new()
        .host(&settings.database.host)
        .username(&settings.database.user)
        .password(&settings.database.password)
        .database(&settings.database.name)
        .to_owned();

    let pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .connect_with(options)
        .await
        .expect("Failed to make connection pool! Please check if the PostgreSQL server is running and try again");

    match is_database_empty(&pool).await {
        Ok(true) => {
            println!("Database is empty, applying schema migration");
            // Embed the schema file as a resource
            match pool.execute(include_str!("../db/schema.sql")).await {
                Ok(_) => println!("Database schema migration successful"),
                Err(e) => panic!("Error during schema migration: {}", e),
            }
        },
        Ok(false) => println!("Database is not empty, skipping schema migration"),
        Err(e) => panic!("Error checking database schema: {}", e),
    };

    // https://github.com/maxcountryman/tower-sessions
    // => \dt *.*
    let session_store = PostgresStore::new(pool.clone())
        // .with_schema_name("test_schema").unwrap()
        .with_table_name(format!("{}_sessions", APP_NAME)).unwrap();

    match session_store.migrate().await {
        Ok(()) => println!("Session store migration successful"),
        Err(e) => panic!("Error during migration: {}", e),
    };

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)))
        .with_secure(false);

    // tera template engine
    let mut tera = match Tera::new(&format!("{}/**/*", settings.directories.templates_dir)) {
        Ok(t) => t,
        Err(e) => panic!("Error initializing Tera: {}", e),
    };
    tera.register_filter("round_and_format", utils::round_and_format_filter);
    tera.register_function("shortcode", controllers::frontend::shortcodes::make_shortcode());

    let admin_router = Router::new()
        .route("/media", get(controllers::backend::media::library))
        // .route("/products/:id/media/update", post(admin::products::media::update))
        // .route("/products/:id/media/add", post(admin::products::media::add))
        // .route("/products/:id/media", post(admin::products::media::select))
        // backend orders
        .route("/orders/new", get(controllers::backend::orders::new))
        .route("/orders/:id", get(controllers::backend::orders::edit))
        .route("/orders", get(controllers::backend::orders::list))
        // backend categories
        .route("/categories/new", get(controllers::backend::categories::new))
        .route("/categories/:id", get(controllers::backend::categories::edit))
        .route("/categories", get(controllers::backend::categories::list))
        // backend products
        .route("/products/:id", get(controllers::backend::products::edit)
            .post(controllers::backend::products::handle))
        .route("/products/new", get(controllers::backend::products::new))
        .route("/products", get(controllers::backend::products::list))
        // backend users
        .route("/users/new", get(controllers::backend::users::new))
        .route("/users/:id", get(controllers::backend::users::edit))
        .route("/users", get(controllers::backend::users::list))
        // admin
        .route("/sidebar", get(controllers::admin::sidebar))
        .route("/", get(controllers::admin::dashboard));

    let app = Router::new()
        .nest("/admin", admin_router)
        .route_layer(from_extractor::<controllers::auth::RequireAuth>())
        .route("/", get(controllers::storefront::facade))
        // .route("/test", get(|| async { "Hello, World!" }))
        .route("/login", get(controllers::auth::login)
            .post(autentication))
        .route("/checkout", get(controllers::frontend::checkout::show)
            .post(controllers::frontend::checkout::place_order))
        .route("/cart/update", post(controllers::frontend::cart::update_cart))
        .route("/cart/add", post(controllers::frontend::cart::add_to_cart))
        .route("/cart", get(controllers::frontend::cart::show))
        .route("/products", get(controllers::frontend::products::list))
        .route("/product-category/:slug", get(controllers::frontend::products::product_category))
        .route("/product/:slug", get(controllers::frontend::products::product))
        .route("/shortcode/products", get(controllers::frontend::shortcodes::products))
        .layer(Extension(pool))
        .layer(Extension(tera))
        // .nest_service("/assets", ServeDir::new("static/assets"))
        // .nest_service("/uploads", ServeDir::new("static/uploads"))
        .layer(session_layer)
        .fallback_service(ServeDir::new(&settings.directories.static_dir));

    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        // .with_graceful_shutdown(async {})
        .await
        .unwrap();

}

