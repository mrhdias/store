# store
A sketch of an e-commerce store built with Rust, using PostgreSQL, Axum, and the Tera template engine.

If you use ArchLinux install [PostgreSQL](https://wiki.archlinux.org/title/PostgreSQL) by following these commands.
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