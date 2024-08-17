# store
A sketch of an e-commerce store built with Rust, using PostgreSQL, [Axum](https://github.com/tokio-rs/axum), and the [Tera](https://keats.github.io/tera/) template engine.

[![Rust](https://github.com/mrhdias/store/actions/workflows/rust.yml/badge.svg)](https://github.com/mrhdias/store/actions/workflows/rust.yml)

If you're using ArchLinux, install [PostgreSQL](https://wiki.archlinux.org/title/PostgreSQL) by following these commands:
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
Download the latest nightly build from [here](https://github.com/mrhdias/store/tags) and uncompress it:
```
unzip nightly-build-YYYYMMDDHHMMSS.zip
```
Populate the Database with Mock Data for Testing:
```
psql -W -U store_admin -d mystoredb -a -w -f db/schema.sql
```
Run it:
```
./store
```
Take some time to review the configuration file: `./config/store.ini`

If you prefer, you can compile and run the server from the source:
```
cargo run
```
To view the store, open the following address in your browser: http://0.0.0.0:8080/

![Login Screenshot](https://raw.githubusercontent.com/mrhdias/store/main/screenshots/login.png)
![Cart Screenshot](https://raw.githubusercontent.com/mrhdias/store/main/screenshots/cart.png)