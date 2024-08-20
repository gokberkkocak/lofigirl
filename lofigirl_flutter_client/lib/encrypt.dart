import 'package:encrypt/encrypt.dart';
import 'package:lofigirl_flutter_client/secret.dart';

class SecureString {
  final String value;
  final Secret secret;

  SecureString(this.value, this.secret);

  Future<(String, String)> encrypt() async {
    final key = Key.fromBase64(secret.aesKey);
    final nonce = IV.fromLength(12);
    final encrypter = Encrypter(AES(key, mode: AESMode.gcm));
    final encrypted = encrypter.encrypt(value, iv: nonce);
    return (encrypted.base64, nonce.base64);
  }

  Future<String> decrypt(String encryptedBase64, String nonceBase64) async {
    final key = Key.fromBase64(secret.aesKey);
    final nonce = IV.fromBase64(nonceBase64);
    final encrypter = Encrypter(AES(key, mode: AESMode.gcm));
    final encrypted = Encrypted.from64(encryptedBase64);
    return encrypter.decrypt(encrypted, iv: nonce);
  }

  Future<Map<String, dynamic>> toJson() async {
    final (encryptedBase64, nonceBase64) = await encrypt();
    return {
      'encrypted_base64': encryptedBase64,
      'nonce_base64': nonceBase64,
    };
  }
}
