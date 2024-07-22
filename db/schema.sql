-- psql -W -U store_admin -d mystoredb -a -w -f schema.sql
-- drop tables if exists
DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS tokens;
DROP TABLE IF EXISTS dimentions;
DROP TABLE IF EXISTS product_media;
DROP TABLE IF EXISTS media;
DROP TABLE IF EXISTS product_categories;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS products;

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
CREATE TYPE order_status AS ENUM ('pending', 'processing', 'on-hold', 'completed', 'cancelled', 'refunded', 'failed', 'trash');
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

CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    parent INTEGER NOT NULL DEFAULT 0,
    UNIQUE(id, slug)
);

CREATE TABLE product_categories (
    product_id INT REFERENCES products(id) ON DELETE CASCADE,
    category_id INT REFERENCES categories(id) ON DELETE CASCADE,
    UNIQUE(product_id, category_id)
);

CREATE TABLE media (
    id SERIAL PRIMARY KEY,
    src VARCHAR(512) NOT NULL,
    name VARCHAR(512) NOT NULL,
    alt VARCHAR(255),
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP
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
CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id),
    billing_id INT REFERENCES billing(id),
    shipping_id INT REFERENCES shipping(id),
    is_paying_customer BOOLEAN DEFAULT FALSE,
);

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
    tax_id_number VARCHAR(255) DEFAULT ''
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
    country	iso_contry_code NOT NULL
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY
    date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    date_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    customer_id INT REFERENCES customers(id) ON DELETE CASCADE,
    customer_ip_address VARCHAR(255) NOT NULL,
    customer_user_agent VARCHAR(255) NOT NULL,
    customer_note VARCHAR(512) DEFAULT '',
    billing_id INT REFERENCES billing(id) ON DELETE CASCADE,
    shipping_id INT REFERENCES shipping(id) ON DELETE CASCADE,
    payment_method VARCHAR(255) NOT NULL, -- The payment gateway method used by the
    payment_method_title VARCHAR(255) NOT NULL,
    status order_status DEFAULT 'pending',
    currency currency DEFAULT 'USD',
    discount_total NUMERIC(10, 2) DEFAULT 0.00, -- Total discount amount for the order
    discount_tax NUMERIC(10, 2) DEFAULT 0.00, -- Total discount tax amount for the order
    prices_include_tax BOOLEAN DEFAULT FALSE, -- True the prices included tax during checkout
    date_paid TIMESTAMP,
    date_completed TIMESTAMP,
    cart_hash VARCHAR(512) NOT NULL
);
*/


INSERT INTO users (username, email, first_name, last_name, password, role, avatar_url) VALUES ('demo', 'demo@example.com', 'John', 'Doe', '12345678', 'admin', 'https://secure.gravatar.com/avatar/e1930bd4d635a8ed77450426e269eaa9?s=32&d=mm&r=g');

-- INSERT INTO categories (name, slug, parent) VALUES
-- ('Uncategorized', 'uncategorized', 0),
-- ('Lorem ipsum', 'lorem-ipsum', 0),
-- ('Pellentesque eget', 'pellentesque-eget', 2),
-- ('Aliquam erat', 'aliquam-erat', 3),
-- ('Sed convallis eget', 'sed-convallis-eget', 3),
-- ('Duis et risus tempor', 'duis-et-risus-tempor', 3),
-- ('Etiam felis dui', 'etiam-felis-dui', 0),
-- ('Etiam lobortis at justo', 'etiam-lobortis-at-justo', 7),
-- ('Sed nec neque', 'sed-nec-neque', 7);

INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/210/300', 'Image 1', 'Image 1');
INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/220/300', 'Image 2', 'Image 2');
INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/230/300', 'Image 3', 'Image 3');
INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/240/300', 'Image 4', 'Image 4');
INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/250/300', 'Image 5', 'Image 5');
INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/260/300', 'Image 6', 'Image 6');
INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/270/300', 'Image 7', 'Image 7');
INSERT INTO media (src, name, alt) VALUES ('https://picsum.photos/280/300', 'Image 8', 'Image 8');

-- 1 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, weight)
VALUES ('7165432', 'Praesent Blandit', 'praesent-blandit', 'http://127.0.0.1:8080/product/praesent-blandit', 'Praesent blandit venenatis neque nec tincidunt. Donec imperdiet in sem a pharetra. Duis justo ante, tincidunt ultrices sem ut, dapibus pellentesque ante. Aliquam lobortis mattis ligula. Proin nisl mi, condimentum gravida sapien sit amet, dictum bibendum elit. Sed faucibus leo vel interdum blandit.', 'Praesent blandit venenatis neque nec tincidunt. Donec imperdiet in sem a pharetra.', 29.99, 39.99, 19.99, TRUE, 100, 500);

-- INSERT INTO product_categories (product_id, category_id) VALUES (1, 1);
INSERT INTO product_media (product_id, media_id) VALUES (1, 1);

-- 2 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, stock_status, weight)
VALUES ('6543212', 'Etiam Lectus Odio', 'etiam-lectus-odio', 'http://127.0.0.1:8080/product/etiam-lectus-odio', 'Etiam lectus odio, mollis ut accumsan eget, ullamcorper vel nisi. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Proin pulvinar finibus elementum. Mauris a tellus dui. Sed pellentesque id justo vel faucibus. Pellentesque vulputate quam id sem placerat, et ultrices lorem mollis.', 'Etiam lectus odio, mollis ut accumsan eget, ullamcorper vel nisi.', 32.00, 32.00, 0.00, FALSE, 0, 'outofstock', 120);

-- INSERT INTO product_categories (product_id, category_id) VALUES (2, 1);
INSERT INTO product_media (product_id, media_id) VALUES (2, 2);

-- 3 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, weight)
VALUES ('6754321', 'Quisque Fermentum', 'quisque-fermentum', 'http://127.0.0.1:8080/product/quisque-fermentum', 'Quisque fermentum malesuada lorem, tincidunt eleifend purus lacinia nec. Suspendisse potenti. In hac habitasse platea dictumst. Pellentesque aliquet lacus eu libero scelerisque eleifend. Integer consectetur, eros ut faucibus tempor, nibh nunc egestas odio, id suscipit nulla urna vitae turpis.', 'Quisque fermentum malesuada lorem, tincidunt eleifend purus lacinia nec. Suspendisse potenti.', 60.00, 65.50, 60.00, TRUE, 51, 210);

-- INSERT INTO product_categories (product_id, category_id) VALUES (3, 1);
INSERT INTO product_media (product_id, media_id) VALUES (3, 3);

-- 4 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, weight)
VALUES ('7439876', 'Donec Erat Eros', 'donec-erat-eros', 'http://127.0.0.1:8080/product/donec-erat-eros', 'Donec erat eros, dictum eget vestibulum et, eleifend vel turpis. Integer vehicula vitae leo eget auctor. Praesent vitae pellentesque urna, ac hendrerit nibh. Quisque fringilla justo eu mauris elementum, a cursus erat tempor. Nam hendrerit elit vel mauris suscipit, quis efficitur mauris tincidunt.', 'Donec erat eros, dictum eget vestibulum et, eleifend vel turpis.', 41.00, 41.00, 41.00, FALSE, 20, 150);

-- INSERT INTO product_categories (product_id, category_id) VALUES (4, 1);
INSERT INTO product_media (product_id, media_id) VALUES (4, 4);

-- 5 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, weight)
VALUES ('6598764', 'Pellentesque Habitant', 'pellentesque-habitant', 'http://127.0.0.1:8080/product/pellentesque-habitant', 'Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Aliquam interdum lacus dui. Integer imperdiet, enim sit amet finibus sollicitudin, mauris purus porttitor leo, sed sagittis mi dolor vitae nisl.', 'Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas.', 50.10, 74.00, 50.10, TRUE, 20, 150);

-- INSERT INTO product_categories (product_id, category_id) VALUES (5, 1);
INSERT INTO product_media (product_id, media_id) VALUES (5, 5);

-- 6 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, weight)
VALUES ('6789001', 'Ut Venenatis Nisi', 'ut-venenatis-nisi', 'http://127.0.0.1:8080/product/ut-venenatis-nisi', 'Ut venenatis nisi vel nisl tincidunt facilisis. Aliquam rutrum metus ut tincidunt vulputate. Nunc ut augue at felis sollicitudin congue nec eu erat. Mauris enim erat, cursus eget lobortis id, dapibus nec ante. Sed sit amet pharetra ipsum. Nulla semper in nunc sit amet pellentesque.', 'Ut venenatis nisi vel nisl tincidunt facilisis. Aliquam rutrum metus ut tincidunt vulputate.', 20.50, 20.50, 20.50, FALSE, 34, 120);

-- INSERT INTO product_categories (product_id, category_id) VALUES (6, 1);
INSERT INTO product_media (product_id, media_id) VALUES (6, 6);

-- 7 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, weight)
VALUES ('7545365', 'Aliquam Interdum', 'aliquam-interdum', 'http://127.0.0.1:8080/product/aliquam-interdum', 'Aliquam interdum lacus dui. Integer imperdiet, enim sit amet finibus sollicitudin, mauris purus porttitor leo, sed sagittis mi dolor vitae nisl. Fusce enim massa, aliquam ac sodales non, imperdiet id metus. Donec mattis urna vel dictum commodo.', 'Aliquam interdum lacus dui. Integer imperdiet, enim sit amet finibus sollicitudin.', 32.99, 32.99, 32.99, FALSE, 21, 231);

-- INSERT INTO product_categories (product_id, category_id) VALUES (7, 1);
INSERT INTO product_media (product_id, media_id) VALUES (7, 7);

-- 8 --
INSERT INTO products (sku, name, slug, permalink, description, short_description, price, regular_price, sale_price, on_sale, stock_quantity, weight)
VALUES ('6896501', 'Nulla Justo Justo', 'nulla-justo-justo', 'http://127.0.0.1:8080/product/nulla-justo-justo', 'Nulla justo justo, molestie non convallis eu, mollis ut orci. Phasellus non posuere leo, nec pretium enim. Ut vitae metus eget elit mattis varius quis a ex. Nam bibendum felis ac euismod dapibus.', 'Nulla justo justo, molestie non convallis eu, mollis ut orci. Phasellus non posuere leo, nec pretium enim.', 8.55, 8.55, 8.55, FALSE, 14, 210);

-- INSERT INTO product_categories (product_id, category_id) VALUES (8, 1);
INSERT INTO product_media (product_id, media_id) VALUES (8, 8);