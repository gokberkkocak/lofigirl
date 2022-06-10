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
