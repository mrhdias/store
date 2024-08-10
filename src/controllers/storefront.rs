//
// Last Modification: 2024-07-27 19:16:22
//

use crate::controllers::frontend::shortcodes;

use axum::{
    extract::Extension,
    response::Html,
};
use tera::{Tera, Context};

pub async fn facade(
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    tera.register_function("shortcode", shortcodes::make_shortcode());

    let data = Context::new();
    let rendered = tera.render("frontend/facade.html", &data).unwrap();
    Html(rendered)
}