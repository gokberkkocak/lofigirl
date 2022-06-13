class LastFMClientPasswordConfig {
  final String username;
  final String password;

  LastFMClientPasswordConfig(this.username, this.password);

  Map<String, dynamic> toJson() => {
        'username': username,
        'password': password,
      };
}

class SessionRequest {
  final LastFMClientPasswordConfig password_config;
  SessionRequest(this.password_config);

  Map<String, dynamic> toJson() => {
        'password_config': password_config.toJson(),
      };
}

class TokenRequest {
  final String? lastfm_session_key;
  final String? listenbrainz_token;
  TokenRequest(this.lastfm_session_key, this.listenbrainz_token);

  Map toJson() => {
        'lastfm_session_key': lastfm_session_key,
        'listenbrainz_token': listenbrainz_token,
      };
}

enum LofiStream { Chill, Sleep }

class Track {
  final String artist;
  final String song;

  Track(this.artist, this.song);

  Map<String, dynamic> toJson() => {
        'artist': artist,
        'song': song,
      };

  factory Track.fromJson(Map<String, dynamic> json) {
    return Track(json['artist'] as String, json['song'] as String);
  }
}

class ScrobbleRequest {
  final String token;
  final String action;
  final Track track;

  ScrobbleRequest(this.token, this.track, this.action);

  Map<String, dynamic> toJson() => {
        'token': token,
        'action': action,
        'track': track.toJson(),
      };
}
