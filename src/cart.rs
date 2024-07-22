//
// Last Modified: 2024-07-03 16:45:16
//

use crate::utils;
use crate::models::cart;

use std::collections::HashMap;

use axum::{
    extract::{Extension, Form, RawForm},
    response::Html,
};

use tera::{
    Tera,
    Context
};

use axum_session_sqlx::SessionPgSession;

pub async fn update_cart(
    session: SessionPgSession,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
    RawForm(form): RawForm) -> Html<String> {

    let raw_form_data = String::from_utf8(form.to_vec()).unwrap();

    let mut current_cart: HashMap<i32, i32> = match session.get("cart") {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    cart.update(&raw_form_data);

    match cart.get().await {
        Ok(products) => {
            session.set("cart", current_cart);

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}

pub async fn add_to_cart(
    session: SessionPgSession,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
    Form(payload): Form<cart::ProductToCart>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart") {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    cart.add(&payload);

    match cart.get().await {
        Ok(products) => {
            session.set("cart", current_cart);

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}

pub async fn show(
    session: SessionPgSession,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart") {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    match cart.get().await {
        Ok(products) => {
            session.set("cart", current_cart);

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }

}