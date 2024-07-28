//
// Last Modified: 2024-07-27 19:20:57
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
    SessionManagerLayer
};
use tower_sessions_sqlx_store::PostgresStore;

use tera::{
    Tera,
    Context
};

use sqlx::postgres::PgPoolOptions;

use serde::{
    Deserialize,
    Serialize
};

const APP_NAME: &str = "store";

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
    Form(payload): Form<LoginForm>) -> (StatusCode, HeaderMap, Html<String>) {

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

#[tokio::main]
async fn main() {

    let config_dir: PathBuf = Path::new(".").join("config");
    let config = setup::Config::new(APP_NAME, &config_dir);
    let db_conf= match config.load() {
        Ok(conf) => conf,
        Err(e) => {
            panic!("Error loading database configuration: {}", e);
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(db_conf.max_connections)
        .connect(&format!("postgres://{}:{}@{}/{}",
            db_conf.user,
            db_conf.password,
            db_conf.host,
            db_conf.name))
        .await
        .expect("Failed to make connection pool! Please check if the PostgreSQL server is running and try again");

    // https://github.com/maxcountryman/tower-sessions
    // => \dt *.*
    let session_store = PostgresStore::new(pool.clone())
        // .with_schema_name("test_schema").unwrap()
        .with_table_name(format!("{}_sessions", APP_NAME)).unwrap();

    match session_store.migrate().await {
        Ok(()) => println!("Migration successful"),
        Err(e) => {
            panic!("Error during migration: {}", e);
        }
    };

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)))
        .with_secure(false);

    let tera = Tera::new("templates/**/*").unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/admin/media", get(controllers::backend::media::library))
        // admin products
        // .route("/admin/products/:id/media/update", post(admin::products::media::update))
        // .route("/admin/products/:id/media/add", post(admin::products::media::add))
        // .route("/admin/products/:id/media", post(admin::products::media::select))
        .route("/admin/categories", get(controllers::backend::categories::list))
        .route("/admin/products/:id", get(controllers::backend::products::edit)
            .post(controllers::backend::products::handle))
        .route("/admin/products/new", get(controllers::backend::products::new))
        .route("/admin/products", get(controllers::backend::products::list))
        .route("/admin/users/:id", get(controllers::backend::users::edit))
        .route("/admin/users", get(controllers::backend::users::list))
        // admin
        .route("/admin/sidebar", get(controllers::admin::sidebar))
        .route("/admin", get(controllers::admin::dashboard))
        .route_layer(from_extractor::<controllers::auth::RequireAuth>())
        .route("/", get(controllers::storefront::facade))
        .route("/test", get(|| async { "Hello, World!" }))
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
        .fallback_service(ServeDir::new("static"));

    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .unwrap();

}

