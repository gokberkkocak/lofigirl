import 'dart:async';
import 'dart:convert';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:lofigirl_flutter_client/config.dart';
import 'package:lofigirl_flutter_client/settings.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:http/http.dart' as http;
import 'package:web_socket_channel/web_socket_channel.dart';

void main() {
  runApp(const LofiGirlWithScaffold());
}

class LofiGirlWithScaffold extends StatelessWidget {
  const LofiGirlWithScaffold({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'LofiGirl Scrobbler Client',
      theme: ThemeData.light(),
      darkTheme: ThemeData.dark(),
      themeMode: ThemeMode.system,
      home: const Scaffold(
        body: LofiGirl(),
      ),
    );
  }
}

class LofiGirl extends StatefulWidget {
  const LofiGirl({super.key});

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
  String? _streamUrl;
  WebSocketChannel? _channel;
  StreamSubscription? _socketStreamHandle;
  Timer? _pingTimerHandle;

  @override
  void initState() {
    super.initState();
    _loadValues();
  }

  // Before removing the widget, we need to stop the timer.
  @override
  void dispose() {
    _pingTimerHandle?.cancel();
    _socketStreamHandle?.cancel();
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
      _streamUrl = prefs.getString("streamUrl");
    });
  }

  void _scrobble() async {
    // socket url parsing
    var socketUrl = Uri.parse('$_serverUrl/track_ws');
    if (socketUrl.scheme == "https") {
      socketUrl = socketUrl.replace(scheme: "wss");
    } else if (socketUrl.scheme == "http") {
      socketUrl = socketUrl.replace(scheme: "ws");
    } else {
      const snackBar = SnackBar(
        content: Text('Server url can only be http/https!'),
      );
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(snackBar);
      }
    }
    // establish socket
    _channel = WebSocketChannel.connect(
      socketUrl,
    );
    // send initial messageu
    _channel?.sink.add('$_streamUrl');
    // set periodic ping
    _pingTimerHandle = Timer.periodic(const Duration(seconds: 30), (timer) {
      developer.log('Pinging socket with binary data');
      _channel?.sink.add(utf8.encode("ping"));
    });
    _socketStreamHandle = _channel?.stream.listen(
      (dynamic message) {
        final nextTrack = Track.fromJson(json.decode(message));
        if (_currentTrack != null) {
          _sendInfo("Listened");
        }
        setState(() {
          _currentTrack = nextTrack;
        });
        _sendInfo("PlayingNow");
      },
      onDone: () {
        developer.log('ws channel closed');
      },
      onError: (error) {
        developer.log('ws error $error');
      },
    );
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
              content: Text('Server is set!'),
            );
            if (context.mounted) {
              ScaffoldMessenger.of(context).showSnackBar(snackBar);
            }
          });
        } else {
          const snackBar = SnackBar(
            content: Text('Server did not respond correctly!'),
          );
          if (context.mounted) {
            ScaffoldMessenger.of(context).showSnackBar(snackBar);
          }
        }
      });
    } else {
      const snackBar = SnackBar(
        content: Text('Server url is not valid!'),
      );
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(snackBar);
      }
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
    if (context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(snackBar);
    }
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
    if (context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(snackBar);
    }
  }

  void onStreamUrlChanged(String value) async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _streamUrl = value;
      prefs.setString("streamUrl", value);
    });
    const snackBar = SnackBar(
      content: Text('Stream url is set!'),
    );
    if (context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(snackBar);
    }
    // maybe websocket init?
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
          if (context.mounted) {
            ScaffoldMessenger.of(context).showSnackBar(snackBar);
          }
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
    var snackBar = const SnackBar(
      content: Text('Last.fm session key is deleted!'),
    );
    if (context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(snackBar);
    }
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
          if (context.mounted) {
            ScaffoldMessenger.of(context).showSnackBar(snackBar);
          }
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
    return DefaultTabController(
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
                              padding: const EdgeInsets.only(top: 10),
                              child: ElevatedButton.icon(
                                label: const Text('Stop scrobbling!'),
                                icon: const Icon(Icons.stop),
                                onPressed: () {
                                  setState(() {
                                    _isScrobbling = false;
                                    _currentTrack = null;
                                    _socketStreamHandle?.cancel();
                                    _socketStreamHandle = null;
                                    _pingTimerHandle?.cancel();
                                    _pingTimerHandle = null;
                                    _channel = null;
                                  });
                                },
                              ))
                        ],
                      )
                    : ListView(
                        padding: const EdgeInsets.all(8),
                        children: (_sessionToken != null)
                            ? [
                                TextField(
                                    decoration: const InputDecoration(
                                      labelText: 'LofiStream URL',
                                    ),
                                    onSubmitted: onStreamUrlChanged,
                                    readOnly: (_isScrobbling),
                                    controller: TextEditingController(
                                        text: _streamUrl)),
                                Padding(
                                    padding: const EdgeInsets.only(top: 10),
                                    child: ElevatedButton.icon(
                                      label: const Text('Start scrobbling!'),
                                      icon: const Icon(Icons.play_arrow),
                                      onPressed: (_streamUrl != null &&
                                              _streamUrl!.isNotEmpty)
                                          ? () {
                                              setState(() {
                                                _isScrobbling = true;
                                                _scrobble();
                                              });
                                            }
                                          : null,
                                    ))
                              ]
                            : [
                                const Center(
                                    child: Text("Let's get started!",
                                        style: TextStyle(fontSize: 20))),
                                const SetSettingsButton()
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
    );
  }
}

class SetSettingsButton extends StatelessWidget {
  const SetSettingsButton({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
        padding: const EdgeInsets.only(top: 10),
        child: ElevatedButton.icon(
            icon: const Icon(Icons.settings),
            label: const Text('Get connected!'),
            onPressed: () {
              DefaultTabController.of(context).animateTo(1);
            }));
  }
}

class ListeningInfo extends StatelessWidget {
  final Track? track;
  const ListeningInfo(this.track, {super.key});

  @override
  Widget build(BuildContext context) {
    return track != null
        ? Column(
            children: [
              const Center(
                  child: Text(
                "Now playing",
              )),
              Center(
                  child: Text(
                "${track!.artist} - ${track!.song}",
                style: const TextStyle(
                  fontSize: 20,
                ),
              ))
            ],
          )
        : const Column(
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
