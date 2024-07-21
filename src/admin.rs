//
// Last Modification: 2024-06-23 20:46:09
//

pub mod media;
pub mod products;
pub mod categories;
pub mod users;

use crate::auth;

use std::collections::HashMap;

use axum::{
    extract::{Extension, FromRequest, FromRequestParts, Query},
    response::Html,
};

use tera::{
    Tera,
    Context
};


pub async fn sidebar(
    Extension(_pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {


    let data = Context::new();
    let rendered = tera.render("admin/sidebar.html", &data).unwrap();
    Html(rendered)
}

pub async fn dashboard(
    auth::RequireAuth { role }: auth::RequireAuth,
    Extension(tera): Extension<Tera>) -> Html<String> {

    println!("Role: {:?}", role);

    let mut data = Context::new();
    data.insert("partial", "dashboard");
    let rendered = tera.render("admin/admin.html", &data).unwrap();
    Html(rendered)
}

pub async fn admin(
    auth::RequireAuth { role }: auth::RequireAuth,
    Query(parameters): Query<HashMap<String, String>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    println!("Admin Role: {:?}", role);

    let mut data = Context::new();

    if role.is_admin() {
        data.insert("partial", "dashboard");
        let rendered = tera.render("admin/admin.html", &data).unwrap();
        return Html(rendered)
    }

    if parameters.contains_key("action") {
        match parameters.get("action").unwrap().as_str() {
            "login" => {
                // Handle login action
                // check the login and password fields
                println!("Login");
            },
            "logout" => {
                // Handle logout action
                println!("Logout");
            },
            _ => {
                // Handle unknown action
                // if authenticated user go to dashboard
                // else show login page
                println!("Unknown action");
            }
        }
    }

    println!("Admin Parameters: {:?}", parameters);

    let rendered = tera.render("login.html", &data).unwrap();
    Html(rendered)
}