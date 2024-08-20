import 'package:lofigirl_flutter_client/encrypt.dart';

class SessionRequest {
  final String username;
  final SecureString securePassword;

  SessionRequest(this.username, this.securePassword);

  Map<String, dynamic> toJson() => {
        'username': username,
        'secure_password': securePassword,
      };
}

class TokenRequest {
  final SecureString? lastfmSessionKey;
  final SecureString? listenbrainzToken;
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
  final String action;
  final Track track;

  ScrobbleRequest(this.track, this.action);

  Map<String, dynamic> toJson() => {
        'action': action,
        'track': track.toJson(),
      };
}
