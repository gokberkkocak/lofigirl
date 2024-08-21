import 'package:dart_jsonwebtoken/dart_jsonwebtoken.dart';
import 'package:lofigirl_flutter_client/encrypt.dart';
import 'package:lofigirl_flutter_client/secret.dart';

class JWTClaims {
  final SecureString value;

  JWTClaims(this.value);

  Future<String> generate() async {
    final secretKey = SecretKey(Secret.aesKey);
    final jsonValue = await value.toJson();
    final jwt = JWT({"secure_token": jsonValue});
    final token = jwt.sign(secretKey,
        expiresIn: const Duration(hours: 1), notBefore: const Duration());
    return token;
  }
}
