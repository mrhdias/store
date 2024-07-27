//
// Last Modification: 2024-07-26 10:43:49
//

use axum::{
    extract::Extension,
    response::Html,
};
use tera::{Tera, Context};

pub async fn facade(
    Extension(tera): Extension<Tera>) -> Html<String> {

    let data = Context::new();
    let rendered = tera.render("facade.html", &data).unwrap();
    Html(rendered)
}