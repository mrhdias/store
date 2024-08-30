//
// Last Modified: 2024-08-30 19:33:23
//

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

use tower_sessions::Session;

pub async fn update_cart(
    session: Session,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>,
    RawForm(form): RawForm) -> Html<String> {

    let raw_form_data = String::from_utf8(form.to_vec()).unwrap();

    let mut current_cart: HashMap<i32, i32> = match session.get("cart").await.unwrap() {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    cart.update(&raw_form_data);

    match cart.get().await {
        Ok(products) => {
            // session.set("cart", current_cart);
            session.insert("cart", current_cart).await.unwrap();

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("frontend/shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}

pub async fn add_to_cart(
    session: Session,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>,
    Form(payload): Form<cart::ProductToCart>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart").await.unwrap() {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    cart.add(&payload);

    match cart.get().await {
        Ok(products) => {
            session.insert("cart", current_cart).await.unwrap();

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("frontend/shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}

pub async fn show(
    session: Session,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart").await.unwrap() {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    match cart.get().await {
        Ok(products) => {
            session.insert("cart", current_cart).await.unwrap();

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("frontend/shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }

}