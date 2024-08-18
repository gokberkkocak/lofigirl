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
  final LastFMClientPasswordConfig passwordConfig;
  SessionRequest(this.passwordConfig);

  Map<String, dynamic> toJson() => {
        'password_config': passwordConfig.toJson(),
      };
}

class TokenRequest {
  final String? lastfmSessionKey;
  final String? listenbrainzToken;
  TokenRequest(this.lastfmSessionKey, this.listenbrainzToken);

  Map toJson() => {
        'lastfm_session_key': lastfmSessionKey,
        'listenbrainz_token': listenbrainzToken,
      };
}

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
