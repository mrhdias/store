//
// Last Modification: 2024-07-24 18:47:23
//

use crate::controllers::auth;

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
    let rendered = tera.render("backend/sidebar.html", &data).unwrap();
    Html(rendered)
}

pub async fn dashboard(
    auth::RequireAuth { role }: auth::RequireAuth,
    Extension(tera): Extension<Tera>) -> Html<String> {

    println!("Role: {:?}", role);

    let mut data = Context::new();
    data.insert("partial", "dashboard");
    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}
