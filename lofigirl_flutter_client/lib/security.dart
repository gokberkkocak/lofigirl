import 'package:encrypt/encrypt.dart';
import 'package:dart_jsonwebtoken/dart_jsonwebtoken.dart';

class Secret {
  static const String aesKey = "MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2Nzg=";
}

class SecureString {
  final String value;

  SecureString(this.value);

  Map<String, dynamic> toJson() {
    final (encryptedBase64, nonceBase64) = AesHelper.encrypt(value);
    return {
      'encrypted_base64': encryptedBase64,
      'nonce_base64': nonceBase64,
    };
  }

  factory SecureString.fromJson(Map<String, dynamic> json) {
    final encryptedBase64 = json['encrypted_base64'] as String;
    final nonceBase64 = json['nonce_base64'] as String;
    final value = AesHelper.decrypt(encryptedBase64, nonceBase64);
    return SecureString(value);
  }
}

class AesHelper {
  static (String, String) encrypt(String value) {
    final key = Key.fromBase64(Secret.aesKey);
    final nonce = IV.fromLength(12);
    final encrypter = Encrypter(AES(key, mode: AESMode.gcm));
    final encrypted = encrypter.encrypt(value, iv: nonce);
    return (encrypted.base64, nonce.base64);
  }

  static String decrypt(String encryptedBase64, String nonceBase64) {
    final key = Key.fromBase64(Secret.aesKey);
    final nonce = IV.fromBase64(nonceBase64);
    final encrypter = Encrypter(AES(key, mode: AESMode.gcm));
    final encrypted = Encrypted.from64(encryptedBase64);
    return encrypter.decrypt(encrypted, iv: nonce);
  }
}

class JWTClaims {
  final SecureString value;

  JWTClaims(this.value);

  String toJWT() {
    final secretKey = SecretKey(Secret.aesKey);
    final jsonValue = value.toJson();
    final jwt = JWT({"secure_token": jsonValue});
    final token = jwt.sign(secretKey,
        expiresIn: const Duration(hours: 1), notBefore: const Duration());
    return token;
  }
}
