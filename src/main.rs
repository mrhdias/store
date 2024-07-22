//
// Last Modified: 2024-07-22 18:41:41
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

mod models;
mod types;
mod products;
mod cart;
mod checkout;
mod admin;
mod utils;
mod shortcodes;
mod auth;
mod notifications;

use chrono::Duration;
use std::collections::HashMap;

use axum::{
    extract::{Extension, Form, Query, Request},
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

use tera::{Tera, Context};

use sqlx::postgres::PgPoolOptions;

use axum_session::{SessionConfig, SessionLayer};
use axum_session_sqlx::SessionPgSessionStore;

use std::path::{Path, PathBuf};
use ini::Ini;
use serde::Deserialize;

const APP_NAME: &str = "store";

#[derive(Debug, Deserialize)]
struct LoginForm {
    user: String,
    password: String,
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

async fn login(
    Query(parameters): Query<HashMap<String, String>>,
    Extension(tera): Extension<Tera>) -> (HeaderMap, Html<String>) {

    let mut headers = HeaderMap::new();
    if parameters.contains_key("action") && parameters.get("action").unwrap() == "logout" {
        headers.insert(axum::http::header::SET_COOKIE, HeaderValue::from_str("token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT").unwrap());
    }

    let data = Context::new();
    let rendered = tera.render("login.html", &data).unwrap();
    return (headers, Html(rendered));
}

async fn home(
    Extension(tera): Extension<Tera>) -> Html<String> {

    let data = Context::new();
    let rendered = tera.render("home.html", &data).unwrap();
    Html(rendered)
}


fn default_config_file(config_dir: &PathBuf) {

    let mut conf = Ini::new();

    conf.with_section(None::<String>)
        .set("encoding", "utf-8");

    conf.with_section(Some("database"))
        .set("host", "localhost")
        .set("user", "admin")
        .set("password", "unknown")
        .set("name", "unknown")
        .set("max_connections", "5");

    conf.write_to_file(config_dir.join(format!("{}.ini", APP_NAME))).unwrap();
}

#[tokio::main]
async fn main() {

    let config_dir: PathBuf = Path::new(".").join("config");
    let ini_file = config_dir.join(format!("{}.ini", APP_NAME));

    println!("Config Directory: {:?}", ini_file.display());

    if !ini_file.exists() {
        default_config_file(&config_dir);
    }

    let config = match Ini::load_from_file(&ini_file) {
        Ok(config) => config,
        Err(err) => {
            panic!("failed to parse config file: {}", err);
        }
    };

    // localhost, store_admin, PreviewSem100, mystoredb

    let db_settings = config.section(Some("database")).expect("no database section in config file");
    let db_host = db_settings.get("host").expect("no database host");
    let db_user = db_settings.get("user").expect("no database user");
    let db_password = db_settings.get("password").expect("no database password");
    let db_name = db_settings.get("name").expect("no database name");
    let db_max_connections: u32 = match db_settings.get("max_connections") {
        Some(value) => match value.parse() {
            Ok(value) => value,
            Err(_) => {
                panic!("invalid max_connections value");
            }
        },
        None => 5,
    };

    let pool = PgPoolOptions::new()
        .max_connections(db_max_connections)
        .connect(&format!("postgres://{}:{}@{}/{}", db_user, db_password, db_host, db_name))
        .await
        .expect("Failed to make connection pool");

    let session_config = SessionConfig::default()
        .with_table_name("user_sessions")
        .with_max_age(Some(Duration::hours(1)));

    // create SessionStore and initiate the database tables
    let session_store = SessionPgSessionStore::new(Some(pool.clone().into()), session_config)
        .await
        .expect("Failed to create session store");

    let tera = Tera::new("templates/**/*").unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/admin/media", get(admin::media::library))
        // admin products
        // .route("/admin/products/:id/media/update", post(admin::products::media::update))
        // .route("/admin/products/:id/media/add", post(admin::products::media::add))
        // .route("/admin/products/:id/media", post(admin::products::media::select))
        .route("/admin/products/:id", get(admin::products::edit)
            .post(admin::products::handle))
        .route("/admin/products/new", get(admin::products::new))
        .route("/admin/products", get(admin::products::list))
        .route("/admin/users/:id", get(admin::users::edit))
        .route("/admin/users", get(admin::users::list))
        // admin
        .route("/admin/sidebar", get(admin::sidebar))
        .route("/admin", get(admin::dashboard))
        .route_layer(from_extractor::<auth::RequireAuth>())
        .route("/", get(home))
        .route("/test", get(|| async { "Hello, World!" }))
        .route("/login", get(login).post(autentication))
        .route("/checkout", get(checkout::show).post(checkout::place_order))
        .route("/cart/update", post(cart::update_cart))
        .route("/cart/add", post(cart::add_to_cart))
        .route("/cart", get(cart::show))
        .route("/products", get(products::list))
        .route("/product-category/:slug", get(products::product_category))
        .route("/product/:slug", get(products::product))
        .route("/shortcode/products", get(shortcodes::products))
        .layer(Extension(pool))
        .layer(Extension(tera))
        .layer(SessionLayer::new(session_store))
        // .nest_service("/assets", ServeDir::new("static/assets"))
        // .nest_service("/uploads", ServeDir::new("static/uploads"))
        .fallback_service(ServeDir::new("static"));



    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await.unwrap();

}

