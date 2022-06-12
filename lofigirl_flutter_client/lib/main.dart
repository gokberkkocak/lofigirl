import 'dart:async';
import 'dart:convert';

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
  String? _currentTrack;
  String? _lastFmUsername;
  String? _lofiStreamName;

  @override
  void initState() {
    Timer.periodic(Duration(seconds: 15), (timer) {
      if (_isScrobbling) {
        _scrobble();
      }
    });
    super.initState();
    _loadValues();
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

  void _scrobble() async {}

  void onServerUrlChanged(String value) {
    final url = Uri.parse('$value/health');
    if (url.isAbsolute) {
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

  void onSessionTokenRequested() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    if (_lastFmSessionKey != null || _listenBrainzToken != null) {
      final url = Uri.parse('$_serverUrl/token');
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
        }
      }
    }
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
                  body: Column(
                      children: (_sessionToken != null)
                          ? [
                              ListTile(
                                title: Text("Chill"),
                                leading: Radio(
                                    value: "chill",
                                    groupValue: _lofiStreamName,
                                    onChanged: (value) {
                                      setState(() {
                                        _lofiStreamName = value.toString();
                                      });
                                    }),
                              ),
                              ListTile(
                                title: Text("Sleep"),
                                leading: Radio(
                                    value: "sleep",
                                    groupValue: _lofiStreamName,
                                    onChanged: (value) {
                                      setState(() {
                                        _lofiStreamName = value.toString();
                                      });
                                    }),
                              ),
                              ElevatedButton(
                                child: const Text('GO!'),
                                onPressed: () {
                                  setState(() {
                                    _isScrobbling = true;
                                  });
                                },
                              )
                            ]
                          : [SetSettingsButton()])),
              Scaffold(
                  body: Column(children: [
                ServerSettings(_serverUrl, onServerUrlChanged),
                ListenBrainzSettings(
                    _listenBrainzToken, onListenBrainzTokenChanged),
                LastFmSettings(
                    _lastFmUsername,
                    _lastFmSessionKey,
                    onLastFmUsernameChanged,
                    onLastFmPasswordChanged,
                    onLastFmSessionKeyDeleted),
                LofiGirlToken(
                    _sessionToken,
                    onSessionTokenRequested,
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
    return Center(
        child: ElevatedButton(
            child: Text('Get connected!'),
            onPressed: () {
              DefaultTabController.of(context)!.animateTo(1);
            }));
  }
}
