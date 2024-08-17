-- psql -W -U store_admin -d mystoredb -a -w -f schema.sql
-- drop tables if exists
DROP TABLE IF EXISTS user_sessions;
DROP TABLE IF EXISTS customer_sessions;
DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS tokens;
DROP TABLE IF EXISTS dimentions;
DROP TABLE IF EXISTS product_media;
DROP TABLE IF EXISTS media;
DROP TABLE IF EXISTS product_categories;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS orders;

DROP TYPE type;
DROP TYPE status;
DROP TYPE stock_status;
DROP TYPE catalog_visibility;
DROP TYPE user_roles;
DROP TYPE order_status;
DROP TYPE currency;
DROP TYPE iso_contry_code;

CREATE TYPE type AS ENUM ('simple', 'grouped', 'external', 'variable');
CREATE TYPE status AS ENUM ('draft', 'pending', 'private', 'publish');
CREATE TYPE stock_status AS ENUM ('instock', 'outofstock', 'onbackorder');
CREATE TYPE catalog_visibility AS ENUM ('visible', 'catalog', 'search', 'hidden');
CREATE TYPE user_roles AS ENUM ('admin', 'customer', 'guest');
CREATE TYPE order_status AS ENUM ('pending', 'processing', 'onhold', 'completed', 'cancelled', 'refunded', 'failed', 'trash');
CREATE TYPE currency AS ENUM ('EUR', 'USD');
CREATE TYPE iso_contry_code AS ENUM ('FR', 'ES', 'PT', 'US');

CREATE TABLE settings (
    id SERIAL PRIMARY KEY,
    smtp_server VARCHAR(255) NOT NULL,
    smtp_port INTEGER DEFAULT 0,
    smtp_username VARCHAR(255) NOT NULL,
    smtp_password VARCHAR(255) NOT NULL,
    smtp_use_tls BOOLEAN DEFAULT FALSE
);

CREATE TABLE tokens (
    token VARCHAR(128) NOT NULL,
    user_id INTEGER UNIQUE NOT NULL,
    expires TIMESTAMP DEFAULT CURRENT_TIMESTAMP + interval '24 hours'
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(128) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(255) UNIQUE NOT NULL DEFAULT '',
    password TEXT NOT NULL,
    first_name VARCHAR(255) NOT NULL DEFAULT '',
    last_name VARCHAR(255) NOT NULL DEFAULT '',
    role user_roles DEFAULT 'guest',
    avatar_url VARCHAR(255) NOT NULL DEFAULT '',
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    sku VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    permalink VARCHAR(512),
    type type DEFAULT 'simple',
    status status DEFAULT 'publish',
    featured BOOLEAN DEFAULT FALSE,
    catalog_visibility catalog_visibility DEFAULT 'visible',
    description TEXT DEFAULT '',
    short_description TEXT DEFAULT '',
    price NUMERIC(10, 2),
    regular_price NUMERIC(10, 2) NOT NULL,
    sale_price NUMERIC(10, 2) DEFAULT 0.00,
    on_sale BOOLEAN DEFAULT FALSE,
    date_on_sale_from TIMESTAMP,
    date_on_sale_to TIMESTAMP,
    manage_stock BOOLEAN DEFAULT FALSE,
    stock_quantity INT,
    stock_status stock_status DEFAULT 'instock',
    weight INT DEFAULT 0,
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    primary_category INTEGER NOT NULL DEFAULT 0,
    UNIQUE(sku, slug)
);

CREATE TABLE media (
    id SERIAL PRIMARY KEY,
    src VARCHAR(512) NOT NULL,
    name VARCHAR(512) NOT NULL,
    alt VARCHAR(255),
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    parent INTEGER NOT NULL DEFAULT 0,
    description VARCHAR(512) NOT NULL DEFAULT '',
    media_id INT REFERENCES media(id),
    UNIQUE(id, slug)
);

-- A product can have multiple categories
CREATE TABLE product_categories (
    product_id INT REFERENCES products(id) ON DELETE CASCADE,
    category_id INT REFERENCES categories(id) ON DELETE CASCADE,
    UNIQUE(product_id, category_id)
);

CREATE TABLE product_media (
    product_id INT REFERENCES products(id) ON DELETE CASCADE,
    media_id INT REFERENCES media(id) ON DELETE CASCADE,
    position INT DEFAULT 0,
    PRIMARY KEY (product_id, media_id)
);

CREATE TABLE dimentions (
    product_id INT REFERENCES products(id) ON DELETE CASCADE,
    length INTEGER,
    width INTEGER,
    height INTEGER,
    PRIMARY KEY (product_id)
);

/*
CREATE TABLE billing (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    company VARCHAR(255) DEFAULT '',
    address	VARCHAR(512) NOT NULL,
    city VARCHAR(255) NOT NULL,
    state VARCHAR(255) NOT NULL,
    postcode VARCHAR(255) NOT NULL,
    country	iso_contry_code NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(255) DEFAULT '',
    tax_id_number VARCHAR(255) DEFAULT '',
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
);

CREATE TABLE shipping (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    company VARCHAR(255) DEFAULT '',
    address	VARCHAR(512) NOT NULL,
    city VARCHAR(255) NOT NULL,
    state VARCHAR(255) NOT NULL,
    postcode VARCHAR(255) NOT NULL,
    country	iso_contry_code NOT NULL,
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
);

CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id),
    billing_id INT REFERENCES billing(id),
    shipping_id INT REFERENCES shipping(id),
    is_paying_customer BOOLEAN DEFAULT FALSE,
);

CREATE TABLE customer_orders (
    order_id INT REFERENCES orders(id) ON DELETE CASCADE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(order_id, user_id)
);
*/

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    order_key VARCHAR(255) NOT NULL,
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    customer_ip_address VARCHAR(255) NOT NULL,
    customer_user_agent VARCHAR(255) NOT NULL,
    customer_note TEXT DEFAULT '',
    billing JSONB, -- JSONB field for storing billing
    shipping JSONB, -- JSONB field for storing shipping address
    line_items JSONB, -- JSONB field for storing line items
    shipping_lines JSONB, -- JSONB field for storing shipping lines
    payment_method VARCHAR(255) NOT NULL, -- The payment gateway method used by the
    payment_method_title VARCHAR(255) NOT NULL,
    status order_status DEFAULT 'pending',
    currency currency DEFAULT 'EUR',
    discount_total NUMERIC(10, 2) DEFAULT 0.00, -- Total discount amount for the order
    discount_tax NUMERIC(10, 2) DEFAULT 0.00, -- Total discount tax amount for the order
    shipping_total NUMERIC(10, 2) DEFAULT 0.00,
    shipping_tax NUMERIC(10, 2) DEFAULT 0.00,
    cart_tax NUMERIC(10, 2) DEFAULT 0.00, -- Sum of line item taxes only
    total NUMERIC(10, 2) DEFAULT 0.00,
    total_tax NUMERIC(10, 2) DEFAULT 0.00,
    prices_include_tax BOOLEAN DEFAULT FALSE, -- True the prices included tax during checkout
    date_paid TIMESTAMP,
    date_completed TIMESTAMP,
    cart_hash VARCHAR(512) NOT NULL
    -- UNIQUE(order_key, cart_hash)
);