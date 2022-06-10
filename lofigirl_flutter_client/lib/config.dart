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
