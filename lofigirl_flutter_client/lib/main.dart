import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:http/http.dart' as http;
import 'dart:developer' as developer;

void main() {
  runApp(const LofiGirl());
}

class LofiGirl extends StatefulWidget {
  const LofiGirl({Key? key}) : super(key: key);

  @override
  State<LofiGirl> createState() => _LofiGirlState();
}

class _LofiGirlState extends State<LofiGirl> {
  String? _lastfmSessionKey;
  String? _listenBrainzToken;
  String? _serverUrl;
  String? _sessionToken;
  bool _isScrobbling = false;
  String? _currentTrack = '';

  @override
  void initState() {
    super.initState();
    _loadValues();
  }

  void _loadValues() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _lastfmSessionKey = prefs.getString('lastFmSessionKey');
      _listenBrainzToken = prefs.getString('listenBrainzToken');
      _sessionToken = prefs.getString('sessionToken');
      _serverUrl = prefs.getString('serverUrl');
    });
  }

  void onServerUrlChanged(String value) {
    final url = Uri.parse('$value/health/');
    if (url.isAbsolute) {
      http.get(Uri.parse('$value/health')).then((http.Response response) async {
        if (response.statusCode == 200) {
          final SharedPreferences prefs = await SharedPreferences.getInstance();
          setState(() {
            _serverUrl = value;
            prefs.setString("serverUrl", value);
          });
        } else {
          developer.log("server is not healthy", name: 'LofiGirl');
        }
      });
    }
  }

  void onServerUrlDeleted() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _serverUrl = null;
      prefs.remove("serverUrl");
    });
  }

  void onListenBrainzTokenChanged(String value) async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _listenBrainzToken = value;
      prefs.setString("listenBrainzToken", value);
    });
  }

  void onListenBrainzTokenDeleted() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    setState(() {
      _listenBrainzToken = null;
      prefs.remove("listenBrainzToken");
    });
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: DefaultTabController(
        length: 3,
        child: Scaffold(
          appBar: AppBar(
            bottom: const TabBar(
              tabs: [
                Tab(icon: Icon(Icons.music_note)),
                Tab(icon: Icon(Icons.self_improvement)),
                Tab(icon: Icon(Icons.settings)),
              ],
            ),
            title: const Text('LofiGirl Scrobbler Client'),
          ),
          body: TabBarView(
            children: [
              Icon(Icons.music_note),
              Icon(Icons.self_improvement),
              Container(
                  child: Column(children: [
                ServerSettings(this._serverUrl, this.onServerUrlChanged,
                    this.onServerUrlDeleted),
                ListenBrainzSettings(
                    this._listenBrainzToken,
                    this.onListenBrainzTokenChanged,
                    this.onListenBrainzTokenDeleted),
              ]))
            ],
          ),
        ),
      ),
    );
  }
}

class ServerSettings extends StatelessWidget {
  String? serverUrl;
  Function(String) onServerUrlChanged;
  Function() onServerUrlDeleted;
  ServerSettings(
      this.serverUrl, this.onServerUrlChanged, this.onServerUrlDeleted);

  @override
  Widget build(BuildContext context) {
    return Column(
        children: serverUrl != null
            ? [
                Text("Server URL has been set to: $serverUrl"),
                ElevatedButton(
                    onPressed: onServerUrlDeleted, child: Text("Delete"))
              ]
            : [
                TextField(
                  decoration: const InputDecoration(
                    labelText: 'Server URL',
                  ),
                  onSubmitted: onServerUrlChanged,
                  controller: TextEditingController(text: serverUrl),
                )
              ]);
  }
}

class ListenBrainzSettings extends StatelessWidget {
  String? listenBrainzToken;
  Function(String) onListenBrainzTokenChanged;
  Function() onListenBrainzTokenDeleted;
  ListenBrainzSettings(this.listenBrainzToken, this.onListenBrainzTokenChanged,
      this.onListenBrainzTokenDeleted);

  @override
  Widget build(BuildContext context) {
    return Column(
        children: listenBrainzToken != null
            ? [
                Text("Listenbrainz Token has been set to: $listenBrainzToken"),
                ElevatedButton(
                    onPressed: onListenBrainzTokenDeleted,
                    child: Text("Delete"))
              ]
            : [
                TextField(
                  decoration: const InputDecoration(
                    labelText: 'ListenBrainz Token',
                  ),
                  onSubmitted: onListenBrainzTokenChanged,
                  controller: TextEditingController(text: listenBrainzToken),
                )
              ]);
  }
}
