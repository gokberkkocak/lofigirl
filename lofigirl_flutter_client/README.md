# Lofi Girl Scrobbler 🎧 - Flutter Client

Flutter frontend for the lofigirl server component.

## Get Started

Check releases for the native builds. 

If you want to play around with the git version of the frontend:

```
flutter pub get
flutter run
```

To build release version

```
flutter build <platform>
```

## Usage

Sharing ideas from the rust wasm web client, flutter client has two page views: main and settings.

- Initial home page:

![home](images/before_login.png)

- Initial settings page:

![settings](images/empty_settings.png)

- Filling the settings:

![home](images/filled_settings.png)

- After connected to the server:

![home](images/ready.png)

- Scrobbling to LastFM and ListenBrainz:

![example scrobble](images/example_song.png)

![example scrobble to lastfm](images/example_song_lastfm.png)

![example scrobble to listenbrainz](images/example_song_listenbrainz.png)