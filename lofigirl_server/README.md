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
[video] # soon to be deprecated in favour of dynamic track endpoint
links = [
    "https::///www.youtube.com/something",
    "https::///www.youtube.com/something", # optional
    "https::///www.youtube.com/something", # optional
]

[server_settings]
token_db = "token.db"
port = 8888 
```

You might keep other config fields in your config files which will be ignored.

## Usage
```
Scrobble the tracks you listen on lofigirl streams

Usage: lofigirl_server [OPTIONS]

Options:
  -c, --config <CONFIG>  Configuration toml file [default: config.toml]
  -h, --help             Print help
  -V, --version          Print version
```

## Endpoints

### GET `/track/{encoded_url}`

#### Response

`200`

```json
{
    "artist": "XXX",
    "song": "XXX",
}
```

`202` 

Process started but not ready.

### POST `/send`

#### Request

```json
{
    "token": "XXX", // token is not in header for now
    "action": "Listened" | "Playing Now"  
    "track": {
        "artist": "XXX",
        "song": "XXX",
    },
}
```

### Response

`200`

### GET `/health`

#### Response

`200`

### POST `/session`

#### Request

```json
{
    "password_config": {
            "username": "XXX",
            "password": "XXX",
    }
}
```

#### Response

`200`

```json
{
    "session_config": {
        "session_key": "XXX"
    }
}
```

### POST `/token`

#### Request Body

```json
{
    "lastfm_session_key": "XXX", // either is optional, at least one should be present
    "listenbrainz_token": "XXX", // either is optional, at least one should be present
}
```

#### Response Body

```json
{
    "token": "XXX"
}
```

### GET `/track_ws`

#### Client side

Initialises the socket with a `text` message which includes the requested url and with this, the client is subscribed to track retrieval.

After the initial agreemen, the client is tasked to send periodic `ping` messages to inform the server that they are still alive and demanding track changes.

No authentication on the socket for now.

#### Server side

After a client is succesfully subscribed, the server sends the serialised track information (in json) in the socket channel whenever the track information changes for the requested url.

Responds the subscribed client's `ping` messages with `pong`. If the server does not receive a ping from a client for `60 seconds`, it drops the socket. 

## Install Lofi Girl Server as a service

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
