# Lofi Girl Scrobbler 🎧 - Client 

## Features

- Standalone - When activated, it allows serverless execution of the system. As it is previously stated, it requires opencv/tesseract dependencies to be available system wide.
- Notify - When activated, it provides notification support for three operating systems linux, windows and macos.

![asda](../images/client_notifications.png)

## Compiling

### Default client

It's possible to run default client with stable compiler.

```
cargo build --release
```

To compile only the client module in the workspace use ```-p``` flag.

```
cargo build --release -p lofigirl
```

### Standalone

```
cargo +nightly build --release --features standalone
```

### With notification support

```
cargo build --release --features notify
```

## Example Config file

```toml
[lastfm] # optional - choose one or both
api_key = "api_key"
api_secret = "api_secret"
username = "username"
password = "password"

[listenbrainz] # optional - choose one or both
token = "token"

[server] # Standalone would not need this section
link = "http://127.0.0.1:8080"
```

You might keep have other config fields in your config files which will be ignored.

## Usage

```
lofigirl 0.1.1
Scrobble the tracks you listen on lofigirl streams

USAGE:
    lofigirl [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -s, --second     Use second video link for listen info
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Configuration toml file [default: config.toml]
```