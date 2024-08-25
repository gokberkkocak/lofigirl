# Lofi Girl Scrobbler 🎧 [![Build](https://github.com/gokberkkocak/lofigirl/actions/workflows/build.yml/badge.svg)](https://github.com/gokberkkocak/lofigirl/actions/workflows/build.yml)

Lofi Girl Scrobbler helps you scrobble (mark as listened) Lofi Girl (previously known as chilledcow) YouTube live stream music tracks. It is a rewrite of a previous weekend [project](https://github.com/gokberkkocak/chilledcow-scrobbler/) of mine. Now it supports both ```LastFM``` and ```ListenBrainz``` with now playing support on both platforms as well. This time it's written in Rust and it's completely panic free!

# Getting Started

## Pre-Requisites 

This project uses ```opencv``` library to capture/process images and ```tesseract-ocr``` to make an image to text analysis.

## Modules

This project includes different modules and several features which you can choose according to your preference. This project can be used in two ways:

- Standalone CLI:
    - If the client module compiled with `standalone` feature, the system does the image processing of the given url, ocr and sends listening information for the user to `lastfm` and/or `listenbrainz`. This module requires all heavy dependencies i.e ```opencv``` and ```tesseract-ocr``` to be available in the system.
- Server / Client:
    - Server - A REST / Websocket API broadcasting server module which does the image processing, ocr and serves it on a selected port. Requires all the dependencies to be present in the system. It can keep user sessions and also sends listening information to `lastfm` and `listenbrainz`.
    - Client - There are multiple thin clients that can communicate with the server:
        - CLI - TUI client that uses the given server configuration to communicate with the ```server``` module using the REST / Websocket API.
        - Lofigirl Web Client - Designed with [TAE](https://guide.elm-lang.org/architecture/), it compiles to wasm, runs on browser and communicates with the ```server``` module using the REST / Websocket API.
        - Lofigirl Flutter Client - Flutter multi platform front-end that communicates with the ```server``` module using the REST / Websocket API.

## Installing all dependencies

### Arch Linux

On a minimal arch installation, these packages (besides rust and cargo) were required to be able compile and run the system:

```
pacman -S openssl pkgconf opencv vtk hdf5 qt5-base glew tesseract tesseract-data-eng clang
```

### Windows

Using [vcpkg](https://github.com/microsoft/vcpkg), it should be possible to compile and use the system.

```
vcpkg install llvm opencv4[contrib,nonfree] tesseract
```

### MacOS

```
brew install opencv tesseract leptonica
```

## Releases

You can find the latest tagged release in the release section.

Alternatively, you should be able to grab latest artifacts on `main` on github actions artifacts.


## Compiling

Instead of using supplied binaries, you can compile the project yourself as well.

Server side uses ```sqlx``` which does compile time query checking so a db must be present on the compilation time. To set the compile time db and run migrations;

```
cargo install sqlx-cli
export DATABASE_URL=sqlite:token.db
sqlx db create 
sqlx migrate run
```

Instead of ```sqlx-cli```, ```sqlite3``` also works to create the db. However, the compilation still requires the ```DATABASE_URL``` environment variable.

```
sqlite3 token.db < migrations/20210525000135_table.sql 
export DATABASE_URL=sqlite:token.db
```

Compile all;

```
cargo build --release
```

Check [server](lofigirl_server/README.md), [client](lofigirl_client/README.md), [web-client](lofigirl_web_client/README.md) and [flutter-client](lofigirl_flutter_client/README.md) for more information on specific module.

## Configuration

To use the system with LastFM, you need a API key and secret. These can be obtained [here](https://www.last.fm/api/account/create). To use with ListenBrainz, you'll need to give your user token which can be found [here](https://listenbrainz.org/profile/).

On the previous project, the system would use the youtube channel information to get live streams however, since it's required YouTube API as well, in this version I decided to simply give the youtube video link directly. Since the livestreams last months, it shouldn't be a big deal. Additionally, it's now also possible to give the second livestream link as well if you want to scrobble that instead. Check ```usage``` for how to do it.

All of the configuration can be put into a toml file like in the [example](https://github.com/gokberkkocak/lofigirl/blob/main/example_config.toml):

```toml
[lastfm] # client optional - server ignore
username = "username" # will be removed after first run and turned into session_key 
password = "password" # will be removed after first run and turned into session_key

[lastfm_api] #standalone client and server use. The others ignore
api_key = "api_key"
api_secret = "api_secret"

[listenbrainz] # client optional - server ignore
token = "token"

[server] # client only mandatory, others would ignore.
link = "http://127.0.0.1:8080"

[server_settings] # server uses, others ignore
token_db = "token.db"
port = 8888
```

Both LastFM and ListenBrainz are optional. You can use one or both depending however you want.

## Security

LastFM username and password only used once to receive the ```session_key``` and they are not stored. Only LastFM session_key and ListenBrainz token are stored on server side which can be seen in the [server table schema](migrations/20210525000135_table.sql).

Token exchange and lastfm session key retrival is done by AES256-GCM encryption however, the pre-compiled binaries and docker images will be using the default hard-coded key in the source code that, it's not 100% secure. If you are worried that someone may know that you are using this application and find the key from here, change the [ENCRYPTION_KEY_BASE64](./lofigirl_shared_common/src/lib.rs) on rust side and [aesKey](./lofigirl_flutter_client/lib/security.dart) on flutter side with your own key. The key needs the same matching and should be 44 bytes base64 - i.e. encoded from 32 bytes string/data hence 256 bits key.  

## Usage

Check [server](lofigirl_server/README.md), [client](lofigirl_client/README.md) or [web-client](lofigirl_web_client/README.md) usage in their sections.

## Docker

Docker images includes all binaries (without notification support) for amd64 architecture. 

```
docker pull gokberkkocak/lofigirl
```

The default entry point of the Docker image is the server module.

Use ```-v``` to pass your configuration file and token db file to the container and ```-p``` for port arrangement.

```
docker run -d -v /path/to/config.toml:/config.toml \
              -v /path/to/token.db:/token.db -p 8888:8888 gokberkkocak/lofigirl:latest 
```
To use with other modules, use ``--entrypoint`` flag.

```
docker run -d -v /path/to/your/config.toml:/config.toml --entrypoint {lofigirl_standalone|lofigirl} gokberkkocak/lofigirl:latest 
```

The dockerhub and github packages do not include images for arm architectures because building these images with qemu take many hours that github actions cannot handle it without timing out. You can check the cross compilation docker images for extracting arm binaries instead.

## Cross-compiling for Armhf and Arm64

Using `Dockerfile.arm64` or `Dockerfile.armhf` you can cross compile the project on `x86_64` for arm devices without qemu emulation.

Example for building and extracting the arm64 binaries.

```bash
docker build --target builder -t lofi_arm64 -f Dockerfile.arm64 .
docker create --name dummy_lofi_arm64 localhost/lofi_arm64:latest
docker cp dummy_lofi_arm64:"/app/bin/*" .
```

# How does it Work

## Main stream as an example

- The program takes the video link and extracts the raw video stream link using ytextract/rustube.
- Using opencv, it opens the stream and captures a single frame as an image periodically.
![full_1](images/example_1_full.jpg)
- The image gets cropped.
![cropped_1](images/example_1_cropped.jpg)
- The background is removed by a mask.
![masked_1](images/example_1_masked.jpg)
- The final image is checked by tesseract-ocr.
- The info is sent to LastFM and/or ListenBrainz.
![lastfm_1](images/example_1_lastfm.png)
![listenbrainz_1](images/example_1_listenbrainz.png)

## Second stream

It works the same way for the second stream as well.
- Full image
![full_2](images/example_2_full.jpg)
- Cropped image
![cropped_](images/example_2_cropped.jpg)
- Masked
![masked_2](images/example_2_masked.jpg)
- Sending to LastFM and ListenBrainz.
![lastfm_2](images/example_2_lastfm.png)
![listenbrainz_2](images/example_2_listenbrainz.png)