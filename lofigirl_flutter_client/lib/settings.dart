import 'package:flutter/material.dart';
import 'dart:developer' as developer;

class ServerSettings extends StatelessWidget {
  final String? serverUrl;
  final String? sessionToken;
  final Function(String) onServerUrlChanged;
  const ServerSettings(
      this.serverUrl, this.sessionToken, this.onServerUrlChanged,
      {super.key});

  @override
  Widget build(BuildContext context) {
    return Column(children: [
      TextField(
        decoration: const InputDecoration(
          labelText: 'Server URL',
        ),
        onSubmitted: onServerUrlChanged,
        readOnly: (sessionToken != null),
        controller: TextEditingController(text: serverUrl),
      ),
    ]);
  }
}

class ListenBrainzSettings extends StatelessWidget {
  final String? listenBrainzToken;
  final String? sessionToken;
  final Function(String) onListenBrainzTokenChanged;
  const ListenBrainzSettings(this.listenBrainzToken, this.sessionToken,
      this.onListenBrainzTokenChanged,
      {super.key});

  @override
  Widget build(BuildContext context) {
    return Column(children: [
      TextField(
        decoration: const InputDecoration(
          labelText: 'ListenBrainz Token',
        ),
        onSubmitted: onListenBrainzTokenChanged,
        readOnly: (sessionToken != null),
        controller: TextEditingController(text: listenBrainzToken),
      )
    ]);
  }
}

class LastFmSettings extends StatelessWidget {
  final String? lastFmUsername;
  final String? lastFMSessionKey;
  final String? sessionToken;
  final Function(String) onLastFMUsernameChanged;
  final Function(String) onLastFMPasswordChanged;
  final Function() onLastFMSessionKeyDeleted;
  const LastFmSettings(
      this.lastFmUsername,
      this.lastFMSessionKey,
      this.sessionToken,
      this.onLastFMUsernameChanged,
      this.onLastFMPasswordChanged,
      this.onLastFMSessionKeyDeleted,
      {super.key});

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
                  readOnly: (sessionToken != null),
                  controller: TextEditingController(text: lastFmUsername),
                ),
                TextField(
                  decoration: const InputDecoration(
                    labelText: 'LastFM Password',
                  ),
                  obscureText: true,
                  onSubmitted: onLastFMPasswordChanged,
                  readOnly: (sessionToken != null),
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
                    padding: const EdgeInsets.only(top: 10),
                    child: ElevatedButton(
                      onPressed: (sessionToken == null)
                          ? onLastFMSessionKeyDeleted
                          : null,
                      child: const Text('Delete Session Key'),
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
      this.onSessionTokenDeleted, this.isActive,
      {super.key});

  @override
  Widget build(BuildContext context) {
    if (!isActive) {
      return const Padding(
          padding: EdgeInsets.only(top: 20),
          child: ElevatedButton(
            onPressed: null,
            child: Text('Connect!'),
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
                        padding: const EdgeInsets.only(top: 10),
                        child: ElevatedButton(
                          onPressed: onSessionTokenDeleted,
                          child: const Text('Disconnect!'),
                        ))
                  ]));
  }
}

class ConnectButton extends StatelessWidget {
  final Future<bool> Function() onSessionTokenRequest;

  const ConnectButton(this.onSessionTokenRequest, {super.key});

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      child: const Text('Connect!'),
      onPressed: () async {
        var ret = onSessionTokenRequest();
        ret.then((value) =>
            developer.log("App Connected: $value", name: 'ConnectButton'));
      },
    );
  }
}
