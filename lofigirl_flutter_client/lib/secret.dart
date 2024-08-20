import 'dart:async' show Future;
import 'dart:convert' show json;
import 'package:flutter/services.dart' show rootBundle;

class Secret {
  final String aesKey;
  Secret(this.aesKey);
  factory Secret.fromJson(Map<String, dynamic> jsonMap) {
    return Secret(jsonMap["aes_key"]);
  }
}

class SecretLoader {
  final String secretPath;

  SecretLoader(this.secretPath);
  Future<Secret> load() {
    return rootBundle.loadStructuredData<Secret>(secretPath,
        (jsonStr) async {
      final secret = Secret.fromJson(json.decode(jsonStr));
      return secret;
    });
  }
}
