import 'package:encrypt/encrypt.dart';
import 'package:lofigirl_flutter_client/secret.dart';

class SecureString {
  final String value;

  SecureString(this.value);

  Map<String, dynamic> toJson() {
    final aesHelper = AesHelper();
    final (encryptedBase64, nonceBase64) = aesHelper.encrypt(value);
    return {
      'encrypted_base64': encryptedBase64,
      'nonce_base64': nonceBase64,
    };
  }

  factory SecureString.fromJson(Map<String, dynamic> json) {
    final aesHelper = AesHelper();
    final encryptedBase64 = json['encrypted_base64'] as String;
    final nonceBase64 = json['nonce_base64'] as String;
    final value = aesHelper.decrypt(encryptedBase64, nonceBase64);
    return SecureString(value);
  }
}

class AesHelper {
  (String, String) encrypt(String value) {
    final key = Key.fromBase64(Secret.aesKey);
    final nonce = IV.fromLength(12);
    final encrypter = Encrypter(AES(key, mode: AESMode.gcm));
    final encrypted = encrypter.encrypt(value, iv: nonce);
    return (encrypted.base64, nonce.base64);
  }

  String decrypt(String encryptedBase64, String nonceBase64) {
    final key = Key.fromBase64(Secret.aesKey);
    final nonce = IV.fromBase64(nonceBase64);
    final encrypter = Encrypter(AES(key, mode: AESMode.gcm));
    final encrypted = Encrypted.from64(encryptedBase64);
    return encrypter.decrypt(encrypted, iv: nonce);
  }
}
