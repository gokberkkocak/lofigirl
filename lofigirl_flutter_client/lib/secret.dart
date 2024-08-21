import 'dart:async' show Future;
import 'dart:convert' show json;
import 'package:flutter/services.dart' show rootBundle;

class Secret {
  static final Secret _instance = Secret._internal();
  String? aesKey;

  factory Secret() {
    return _instance;
  }

  Secret._internal();

  void setKeyFromJson(Map<String, dynamic> jsonMap) {
    this.aesKey = jsonMap["aes_key"];
  }
}

class SecretLoader {
  final String secretPath;

  SecretLoader(this.secretPath);
  Future<void> load() {
    return rootBundle.loadStructuredData<void>(secretPath, (jsonStr) async {
      final secret = Secret();
      secret.setKeyFromJson(json.decode(jsonStr));
    });
  }
}
