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
VALUES ('6896521', 'Nulla Justo Justo', 'nulla-justo-justo', 'http://127.0.0.1:8080/product/nulla-justo-justo', 'Nulla justo justo, molestie non convallis eu, mollis ut orci. Phasellus non posuere leo, nec pretium enim. Ut vitae metus eget elit mattis varius quis a ex. Nam bibendum felis ac euismod dapibus.', 'Nulla justo justo, molestie non convallis eu, mollis ut orci. Phasellus non posuere leo, nec pretium enim.', 8.55, 8.55, 8.55, FALSE, 14, 210);

-- INSERT INTO product_categories (product_id, category_id) VALUES (8, 1);
INSERT INTO product_media (product_id, media_id) VALUES (8, 8);