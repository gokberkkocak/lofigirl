class Secret {
  static final Secret _instance = Secret._internal();
  String? aesKey;

  factory Secret() {
    return _instance;
  }

  Secret._internal();

  void initKey() {
    this.aesKey = "MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2Nzg=";
  }
}
