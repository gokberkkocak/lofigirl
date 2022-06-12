import 'package:flutter/material.dart';
import 'dart:developer' as developer;

class ServerSettings extends StatelessWidget {
  String? serverUrl;
  Function(String) onServerUrlChanged;
  ServerSettings(this.serverUrl, this.onServerUrlChanged);

  @override
  Widget build(BuildContext context) {
    return Column(children: [
      TextField(
        decoration: const InputDecoration(
          labelText: 'Server URL',
        ),
        onSubmitted: onServerUrlChanged,
        controller: TextEditingController(text: serverUrl),
      ),
    ]);
  }
}

class ListenBrainzSettings extends StatelessWidget {
  String? listenBrainzToken;
  Function(String) onListenBrainzTokenChanged;
  ListenBrainzSettings(this.listenBrainzToken, this.onListenBrainzTokenChanged);

  @override
  Widget build(BuildContext context) {
    return Column(children: [
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

class LastFmSettings extends StatelessWidget {
  String? lastFmUsername;
  String? lastFMSessionKey;
  Function(String) onLastFMUsernameChanged;
  Function(String) onLastFMPasswordChanged;
  Function() onLastFMSessionKeyDeleted;
  LastFmSettings(
      this.lastFmUsername,
      this.lastFMSessionKey,
      this.onLastFMUsernameChanged,
      this.onLastFMPasswordChanged,
      this.onLastFMSessionKeyDeleted);

  @override
  Widget build(BuildContext context) {
    return Column(
        children: (lastFMSessionKey == null)
            ? [
                TextField(
                  decoration: const InputDecoration(
                    labelText: 'LastFM Username',
                  ),
                  onSubmitted: onLastFMUsernameChanged,
                  controller: TextEditingController(text: lastFmUsername),
                ),
                TextField(
                  decoration: const InputDecoration(
                    labelText: 'LastFM Password',
                  ),
                  obscureText: true,
                  onSubmitted: onLastFMPasswordChanged,
                  controller: TextEditingController(text: ''),
                )
              ]
            : [
                TextField(
                  decoration: const InputDecoration(
                    labelText: 'LastFM Username',
                  ),
                  onSubmitted: onLastFMUsernameChanged,
                  readOnly: true,
                  controller: TextEditingController(text: lastFmUsername),
                ),
                TextField(
                  decoration: const InputDecoration(
                    labelText: 'LastFM Session Key',
                  ),
                  readOnly: true,
                  controller: TextEditingController(text: lastFMSessionKey),
                ),
                Padding(
                    padding: EdgeInsets.only(top: 10),
                    child: ElevatedButton(
                      child: const Text('Delete Session Key'),
                      onPressed: onLastFMSessionKeyDeleted,
                    ))
              ]);
  }
}

class LofiGirlToken extends StatelessWidget {
  final String? sessionToken;
  final Future<bool> Function() onSessionTokenRequest;
  final Function() onSessionTokenDeleted;
  final bool isActive;

  const LofiGirlToken(this.sessionToken, this.onSessionTokenRequest,
      this.onSessionTokenDeleted, this.isActive);

  @override
  Widget build(BuildContext context) {
    if (!isActive) {
      return Padding(
          padding: const EdgeInsets.only(top: 20),
          child: ElevatedButton(
            child: const Text('Connect!'),
            onPressed: null,
          ));
    }
    return Padding(
        padding: const EdgeInsets.only(top: 20),
        child: Column(
            children: (sessionToken == null)
                ? [ConnectButton(onSessionTokenRequest)]
                : [
                    TextField(
                      decoration: const InputDecoration(
                        labelText: 'App Session Token',
                      ),
                      readOnly: true,
                      controller: TextEditingController(text: sessionToken),
                    ),
                    Padding(
                        padding: EdgeInsets.only(top: 10),
                        child: ElevatedButton(
                          child: const Text('Disconnect!'),
                          onPressed: onSessionTokenDeleted,
                        ))
                  ]));
  }
}

class ConnectButton extends StatelessWidget {
  final Future<bool> Function() onSessionTokenRequest;

  const ConnectButton(this.onSessionTokenRequest);

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      child: const Text('Connect!'),
      onPressed: () async {
        // Go to play page
        await onSessionTokenRequest();
      },
    );
  }
}
