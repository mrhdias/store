# store
A sketch of an e-commerce store built with Rust, using PostgreSQL, [Axum](https://github.com/tokio-rs/axum), and the [Tera](https://keats.github.io/tera/) template engine.

If using ArchLinux, install [PostgreSQL](https://wiki.archlinux.org/title/PostgreSQL) by following these commands:
```
sudo pacman -S postgresql
sudo -i -u postgres
initdb -D /var/lib/postgres/data
exit
sudo systemctl start postgresql
```
Create the store admin user
```
sudo -i -u postgres
psql
postgres=# CREATE ROLE store_admin WITH LOGIN PASSWORD 'mypassword';
postgres=# CREATE DATABASE mystoredb OWNER store_admin;
postgres=# GRANT ALL PRIVILEGES ON DATABASE mystoredb TO store_admin;
postgres=# \q
```
Populate the database with mock data to perform a test.
```
git clone https://github.com/mrhdias/store
cd store
psql -W -U store_admin -d mystoredb -a -w -f db/schema.sql
```
Compile and then run it:
```
cargo run
Config Directory: "./config/store.ini"
Take some time to check the configuration file.: ./config/store.ini
```
Edit the configuration file, save it, and run it again:
```
nano -w ./config/store.ini
cargo run
```
To view the store, enter the address http://0.0.0.0:8080/ in your favorite browser.

![Login Screenshot](https://raw.githubusercontent.com/mrhdias/store/main/screenshots/login.png)