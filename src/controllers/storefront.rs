//
// Last Modification: 2024-07-27 19:16:22
//

use axum::{
    extract::Extension,
    response::Html,
};
use tera::{Tera, Context};

pub async fn facade(
    Extension(tera): Extension<Tera>) -> Html<String> {

    let data = Context::new();
    let rendered = tera.render("frontend/facade.html", &data).unwrap();
    Html(rendered)
}