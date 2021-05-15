# Lofi Girl Scrobbler 🎧

Lofi Girl Scrobbler helps you scrobble (mark as listened) Lofi Girl (previously known as chilledcow) YouTube live stream music tracks. It is a rewrite of a previous weekend [project](https://github.com/gokberkkocak/chilledcow-scrobbler/) of mine. Now it supports both ```LastFM``` and ```ListenBrainz``` with now playing support on both platforms as well. This time it's written in Rust and it's completely panic free!

# Getting Started



## Pre-Requisites 

This project uses ```opencv``` library to capture\process images and ```tesseract-ocr``` to make an image to text analysis.

## Modules

This project includes different modules and several features which you can choose according to your preference. The list of binaries which are compiled on releases are

- Lofigirl - Includes Optional multi-os notification system.
    - Client - A client-only version which doesn't require ```opencv``` or ```tesseract-ocr``` dependencies. It uses the given server configuration to retrieve information from the ```server``` module.
    - Standalone - The standalone version which runs the images processing and ocr by itself and submits the data on a regular interval. Requires all the dependencies to be present in the system.
- Lofigirl Server - The http server module which does the image processing, ocr and serves it on a selected port. Requires all the dependencies to be present in the system.

## Installing all dependencies

### Arch Linux

On a minimal arch installation, these packages (besides rust and cargo) were required to be able compile and run the system;

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

## Compiling

One of the crates [rustube](https://lib.rs/crates/rustube) in the project requires nightly compiler so the project only compiles on nightly compiler at the moment.

```
cargo +nightly build --release
```

Check [server](lofigirl_server/README.md) and/or [client](lofigirl_client/README.md) for more information.

## Configuration

To use the system with LastFM, you need a API key and secret. These can be obtained [here](https://www.last.fm/api/account/create). To use with ListenBrainz, you'll need to give your user token which can be found [here](https://listenbrainz.org/profile/).

On the previous project, the system would use the youtube channel information to get live streams however, since it's required YouTube API as well, in this version I decided to simply give the youtube video link directly. Since the livestreams last months, it shouldn't be a big deal. Additionally, it's now also possible to give the second livestream link as well if you want to scrobble that instead. Check ```usage``` for how to do it.

All of the configuration can be put into a toml file like in the [example](https://github.com/gokberkkocak/lofigirl/blob/main/example_config.toml)

```toml
[lastfm] # client optional - server ignore
api_key = "api_key"
api_secret = "api_secret"
username = "username"
password = "password"

[listenbrainz] # client optional - server ignore
token = "token"

[video] # it is mandatory for every module except client only
link = "https::///www.youtube.com/something"
second_link = "https::///www.youtube.com/something" # optional

[server] # client only mandatory, others would ignore.
link = "http://127.0.0.1:8080"
```

Both LastFM and ListenBrainz are optional. You can use one or both depending however you want.

## Usage

Check [server](lofigirl_server/README.md) or [client](lofigirl_client/README.md) usage on their sections.

## Docker

Docker images includes all binaries (without notification support).

```
docker pull gokberkkocak/lofigirl
```

Use ```-v``` to pass your configuration file to the container.

```
docker run -d -v /path/to/your/config.toml:/config.toml gokberkkocak/lofigirl:latest 
```

The default entry point of the Docker image is the server module.

To use with other modules, use ``--entrypoint`` flag.

```
docker run -d -v /path/to/your/config.toml:/config.toml --entrypoint {lofigirl_standalone|lofigirl} gokberkkocak/lofigirl:latest 
```

# How does it Work

## Main stream as an example

- rustube takes video link and brings the raw video stream for opencv.
- opencv opens the stream and captures a single frame.
![full_1](images/example_1_full.jpg)
- The image gets cropped
![cropped_1](images/example_1_cropped.jpg)
- The background is removed by a mask.
![masked_1](images/example_1_masked.jpg)
- tesseract-ocr checks the image
- The info is sent to LastFM and/or ListenBrainz.
![lastfm_1](images/example_1_lastfm.png)
![listenbrainz_1](images/example_1_listenbrainz.png)

## Second stream

It work the same way for the second stream as well.
- Full image
![full_2](images/example_2_full.jpg)
- Cropped image
![cropped_](images/example_2_cropped.jpg)
- Masked
![masked_2](images/example_2_masked.jpg)
- Sending to LastFM and ListenBrainz.
![lastfm_2](images/example_2_lastfm.png)
![listenbrainz_2](images/example_2_listenbrainz.png)

# Limitations

I'm aware that opencv occasionally (sometimes consecutively) fails to read header information from a stream but I haven't managed to find the source of the problem. Because of it, it's possible that some listen information might not be sent. So no guarantees, but I hope it's better than sending nothing!
