import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:lofigirl_flutter_client/config.dart';
import 'package:lofigirl_flutter_client/settings.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:http/http.dart' as http;
import 'dart:developer' as developer;

void main() {
  runApp(const LofiGirlWithScaffold());
}

class LofiGirlWithScaffold extends StatelessWidget {
  const LofiGirlWithScaffold({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'LofiGirl Scrobbler Client',
      home: Scaffold(
        body: const LofiGirl(),
      ),
    );
  }
}

class LofiGirl extends StatefulWidget {
  const LofiGirl({Key? key}) : super(key: key);

  @override
  State<LofiGirl> createState() => _LofiGirlState();
}

class _LofiGirlState extends State<LofiGirl> {
  String? _lastFmSessionKey;
  String? _listenBrainzToken;
  String? _serverUrl;
  String? _sessionToken;
  bool _isScrobbling = false;
  Track? _currentTrack;
  String? _lastFmUsername;
  LofiStream? _lofiStreamName;
  int _seenCount = 0;
  Timer? _timer;

  @override
  void initState() {
    _timer = Timer.periodic(Duration(seconds: 15), (timer) {
      if (_isScrobbling) {
        _scrobble();
      }
    });
    super.initState();
    _loadValues();
  }

  // Before removing the widget, we need to stop the timer.
  // Timer is guaranteed to be not null since initState() sets it.
  @override
  void dispose() {
    _timer!.cancel();
    super.dispose();
  }

  void _loadValues() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _lastFmSessionKey = prefs.getString('lastFmSessionKey');
      _listenBrainzToken = prefs.getString('listenBrainzToken');
      _sessionToken = prefs.getString('sessionToken');
      _serverUrl = prefs.getString('serverUrl');
      _lastFmUsername = prefs.getString('lastFmUsername');
    });
  }

  void _scrobble() async {
    var nextTrack = await _getTrack();
    if (nextTrack == null) {
      return;
    }

    if (_currentTrack != null &&
        _currentTrack!.artist == nextTrack.artist &&
        _currentTrack!.song == nextTrack.song) {
      _seenCount += 1;
    } else {
      _seenCount = 0;
    }

    setState(() {
      _currentTrack = nextTrack;
    });

    developer.log("$_seenCount", name: 'seenCount');

    if (_seenCount == 3) {
      _sendInfo("Listened");
    }
    if (_seenCount == 0) {
      _sendInfo("PlayingNow");
    }
  }

  Future<Track?> _getTrack() async {
    var endPoint = (_lofiStreamName == LofiStream.Chill) ? "chill" : "sleep";
    final url = Uri.parse('$_serverUrl/track/$endPoint');
    developer.log('GET $url', name: 'LofiGirl');
    if (url.isAbsolute) {
      var track = await http.get(url).then((http.Response response) async {
        if (response.statusCode == 200) {
          var track = Track.fromJson(json.decode(response.body));
          return track;
        } else {
          developer.log('Error getting track', name: 'LofiGirl');
        }
      });
      return track;
    }
    return null;
  }

  void _sendInfo(String info) {
    final url = Uri.parse('$_serverUrl/send');
    developer.log('POST $url', name: 'LofiGirl');
    if (url.isAbsolute) {
      var request = ScrobbleRequest(_sessionToken!, _currentTrack!, info);
      var body = json.encode(request.toJson());
      http
          .post(url,
              headers: <String, String>{
                'Content-Type': 'application/json; charset=UTF-8',
              },
              body: body)
          .then((http.Response response) {
        if (response.statusCode == 200) {
          developer.log('Info sent correctly', name: 'LofiGirl');
        } else {
          developer.log('Error sending scrobble', name: 'LofiGirl');
        }
      });
    }
  }

  void onServerUrlChanged(String value) {
    final url = Uri.parse('$value/health');
    if (url.isAbsolute) {
      developer.log('GET $url', name: 'LofiGirl');
      http.get(url).then((http.Response response) async {
        if (response.statusCode == 200) {
          final SharedPreferences prefs = await SharedPreferences.getInstance();
          setState(() {
            _serverUrl = value;
            prefs.setString("serverUrl", value);
            const snackBar = SnackBar(
              content: const Text('Server is set!'),
            );
            ScaffoldMessenger.of(context).showSnackBar(snackBar);
          });
        } else {
          const snackBar = SnackBar(
            content: const Text('Server did not respond correctly!'),
          );
          ScaffoldMessenger.of(context).showSnackBar(snackBar);
        }
      });
    } else {
      const snackBar = SnackBar(
        content: Text('Server url is not valid!'),
      );
      ScaffoldMessenger.of(context).showSnackBar(snackBar);
    }
  }

  void onListenBrainzTokenChanged(String value) async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _listenBrainzToken = value;
      prefs.setString("listenBrainzToken", value);
    });
    const snackBar = SnackBar(
      content: Text('ListenBrainz token is set!'),
    );
    ScaffoldMessenger.of(context).showSnackBar(snackBar);
  }

  void onLastFmUsernameChanged(String value) async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _lastFmUsername = value;
      prefs.setString("lastFmUsername", value);
    });
    const snackBar = SnackBar(
      content: Text('Last.fm username is set!'),
    );
    ScaffoldMessenger.of(context).showSnackBar(snackBar);
  }

  void onLastFmPasswordChanged(String value) async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    if (_lastFmUsername != null) {
      final url = Uri.parse('$_serverUrl/session');
      developer.log('POST $url', name: 'LofiGirl');
      if (url.isAbsolute) {
        var config = LastFMClientPasswordConfig(_lastFmUsername!, value);
        var request = SessionRequest(config);
        final body = json.encode(request.toJson());
        final response = await http.post(
          url,
          headers: <String, String>{
            'Content-Type': 'application/json; charset=UTF-8',
          },
          body: body,
        );
        if (response.statusCode == 200) {
          final sessionKey =
              json.decode(response.body)['session_config']['session_key'];
          setState(() {
            _lastFmSessionKey = sessionKey;
            prefs.setString("lastFmSessionKey", sessionKey);
          });
          const snackBar = SnackBar(
            content: Text('Last.fm session key is set!'),
          );
          ScaffoldMessenger.of(context).showSnackBar(snackBar);
        }
      }
    }
  }

  void onLastFmSessionKeyDeleted() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _lastFmSessionKey = null;
      prefs.remove('lastFmSessionKey');
    });
    final snackBar = SnackBar(
      content: const Text('Last.fm session key is deleted!'),
    );
    ScaffoldMessenger.of(context).showSnackBar(snackBar);
  }

  Future<bool> onSessionTokenRequested() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    if (_lastFmSessionKey != null || _listenBrainzToken != null) {
      final url = Uri.parse('$_serverUrl/token');
      developer.log('POST $url', name: 'LofiGirl');
      if (url.isAbsolute) {
        var request = TokenRequest(_lastFmSessionKey, _listenBrainzToken);
        final body = json.encode(request.toJson());
        final response = await http.post(
          url,
          headers: <String, String>{
            'Content-Type': 'application/json; charset=UTF-8',
          },
          body: body,
        );
        if (response.statusCode == 200) {
          final token = json.decode(response.body)['token'];
          setState(() {
            _sessionToken = token;
            prefs.setString("sessionToken", token);
          });
          const snackBar = SnackBar(
            content: Text('Session token is set!'),
          );
          ScaffoldMessenger.of(context).showSnackBar(snackBar);
          // developer.log("$_sessionToken", name: 'Session token');
          return true;
        }
      }
    }
    return false;
  }

  void onSessionTokenDeleted() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _sessionToken = null;
      prefs.remove('sessionToken');
    });
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: DefaultTabController(
        length: 2,
        child: Scaffold(
          appBar: AppBar(
            bottom: const TabBar(
              tabs: [
                Tab(icon: Icon(Icons.music_note)),
                Tab(icon: Icon(Icons.settings)),
              ],
            ),
            title: const Text('LofiGirl Scrobbler Client'),
          ),
          body: TabBarView(
            children: [
              Scaffold(
                  body: (_isScrobbling)
                      ? ListView(
                          padding: const EdgeInsets.all(8),
                          children: [
                            ListeningInfo(_currentTrack),
                            Padding(
                                padding: EdgeInsets.only(top: 10),
                                child: ElevatedButton.icon(
                                  label: const Text('Stop scrobbling!'),
                                  icon: const Icon(Icons.stop),
                                  onPressed: () {
                                    setState(() {
                                      _isScrobbling = false;
                                      _currentTrack = null;
                                    });
                                  },
                                ))
                          ],
                        )
                      : ListView(
                          padding: const EdgeInsets.all(8),
                          children: (_sessionToken != null)
                              ? [
                                  Center(
                                      child: Text(
                                          "Which steam are you listening right now?",
                                          style: TextStyle(fontSize: 20))),
                                  RadioListTile(
                                    title: const Text('Chill Stream'),
                                    value: LofiStream.Chill,
                                    groupValue: _lofiStreamName,
                                    onChanged: (value) {
                                      setState(() {
                                        _lofiStreamName = LofiStream.Chill;
                                      });
                                    },
                                  ),
                                  RadioListTile(
                                    title: const Text('Sleep Stream'),
                                    value: LofiStream.Sleep,
                                    groupValue: _lofiStreamName,
                                    onChanged: (value) {
                                      setState(() {
                                        _lofiStreamName = LofiStream.Sleep;
                                      });
                                    },
                                  ),
                                  Padding(
                                      padding: EdgeInsets.only(top: 10),
                                      child: ElevatedButton.icon(
                                        label: const Text('Start scrobbling!'),
                                        icon: const Icon(Icons.play_arrow),
                                        onPressed: () {
                                          setState(() {
                                            _isScrobbling = true;
                                          });
                                        },
                                      ))
                                ]
                              : [
                                  Center(
                                      child: Text("Let's get started!",
                                          style: TextStyle(fontSize: 20))),
                                  SetSettingsButton()
                                ])),
              Scaffold(
                  body: ListView(padding: const EdgeInsets.all(8), children: [
                ServerSettings(_serverUrl, _sessionToken, onServerUrlChanged),
                ListenBrainzSettings(_listenBrainzToken, _sessionToken,
                    onListenBrainzTokenChanged),
                LastFmSettings(
                    _lastFmUsername,
                    _lastFmSessionKey,
                    _sessionToken,
                    onLastFmUsernameChanged,
                    onLastFmPasswordChanged,
                    onLastFmSessionKeyDeleted),
                LofiGirlToken(
                    _sessionToken,
                    onSessionTokenRequested,
                    onSessionTokenDeleted,
                    (((_lastFmSessionKey != null) ||
                            (_listenBrainzToken != null)) &&
                        (_serverUrl != null))),
              ]))
            ],
          ),
        ),
      ),
    );
  }
}

class SetSettingsButton extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Padding(
        padding: EdgeInsets.only(top: 10),
        child: ElevatedButton.icon(
            icon: Icon(Icons.settings),
            label: Text('Get connected!'),
            onPressed: () {
              DefaultTabController.of(context)!.animateTo(1);
            }));
  }
}

class ListeningInfo extends StatelessWidget {
  final Track? track;
  const ListeningInfo(this.track);

  @override
  Widget build(BuildContext context) {
    return track != null
        ? Column(
            children: [
              Center(
                  child: Text(
                "Now playing",
              )),
              Center(
                  child: Text(
                "${track!.artist} - ${track!.song}",
                style: TextStyle(
                  fontSize: 20,
                ),
              ))
            ],
          )
        : Column(
            children: [
              Center(
                  child: Text(
                "Getting song info...",
                style: TextStyle(
                  fontSize: 20,
                ),
              ))
            ],
          );
  }
}
