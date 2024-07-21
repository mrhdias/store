//
// Last Modification: 2024-07-21 23:08:51
//

pub mod media;
pub mod products;
pub mod categories;
pub mod users;

use crate::auth;

use axum::{
    extract::Extension,
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
