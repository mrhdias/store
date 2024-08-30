//
// Last Modification: 2024-08-30 20:25:42
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