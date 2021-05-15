# Lofi Girl Scrobbler 🎧 - Server 

Uses [actix-web](https://actix.rs/) for the web server backend. 

## Compiling

```
cargo +nightly build --release
```

To compile only the server module in the workspace use ```-p``` flag.

```
cargo +nightly build --release -p lofigirl_server
```

## Example Config

```toml
[video] # it is mandatory for every module except client only
link = "https::///www.youtube.com/something"
second_link = "https::///www.youtube.com/something" # optional
```

You might keep other config fields in your config files which will be ignored.

## Usage
```
lofigirl_server 0.1.0
Scrobble the tracks you listen on lofigirl streams

USAGE:
    lofigirl_server [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -o, --only-first    Only provide information for the first given link
    -V, --version       Prints version information

OPTIONS:
    -c, --config <config>    Configuration toml file [default: config.toml]
    -p, --port <port>        Configuration toml file [default: 8888]
```
