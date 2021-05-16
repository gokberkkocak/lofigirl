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

## Install Lofigirl-Server as a service

Here's an example configuration which uses rootless podman with the docker container.

Pull from docker.io.

```
podman pull docker.io/gokberkkocak/lofigirl
```

Run first time to give a container name. 

```
podman run --name lofigirl -p 8888:8888 -v ~/config.toml:/config.toml gokberkkocak/lofigirl:latest
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
