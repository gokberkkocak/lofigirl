# Lofi Girl Scrobbler 🎧 - Server 

Uses [actix-web](https://actix.rs/) for the web server backend and [sqlx](https://github.com/launchbadge/sqlx) for sqlite token storage database.

## Compiling

```sqlx``` does compile time query checking so a db must be present on the compilation time. To set the compile time db and run migrations;

```
cargo install sqlx-cli
export DATABASE_URL=sqlite:token.db
sqlx db create 
sqlx migrate run
```

Or you can use ```sqlite```. The environment variable ```DATABASE_URL``` is still required for compilation.

```
sqlite3 token.db < migrations/20210525000135_table.sql 
export DATABASE_URL=sqlite:token.db
```

Compiling the release build.

```
cargo build --release
```

To compile only the server module in the workspace use ```-p``` flag.

```
cargo build --release -p lofigirl_server
```

## Example Config

```toml
[video]
link = "https::///www.youtube.com/something"
second_link = "https::///www.youtube.com/something" # optional

[server_settings]
token_db = "token.db"
port = 8888 
```

You might keep other config fields in your config files which will be ignored.

## Usage
```
lofigirl_server 0.1.1
Scrobble the tracks you listen on lofigirl streams

USAGE:
    lofigirl_server [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -o, --only-first    Only provide information for the first given link
    -V, --version       Prints version information

OPTIONS:
    -c, --config <config>    Configuration toml file [default: config.toml]
```

## Install Lofigirl-Server as a service

### Using Docker/Podman

Here's an example configuration which uses rootless podman with the docker container.

Pull from docker.io.

```
podman pull docker.io/gokberkkocak/lofigirl
```

Run first time to give a container name and ctrl-c. Then give the configuration file and the db by ```-v``` flag. To create the db, check the beginning of the compilation [section](#compiling).

```
podman run --name lofigirl -p 8888:8888 -v /path/to/config.toml:/config.toml \
           -v /path/to/token.db:/token.db gokberkkocak/lofigirl:latest
```

Service file.

```
[Unit]
Description=Lofigirl
After=network.target

[Service]
ExecStart=podman start -a lofigirl
ExecStop=podman stop -t 2 lofigirl
User=podman

[Install]
WantedBy=multi-user.target
```

### Using Compiled Assets

```
[Unit]
Description=Lofigirl
After=network.target

[Service]
ExecStart=/path/to/lofigirl_server --config /path/to/config.toml
User=user

[Install]
WantedBy=multi-user.target
```
