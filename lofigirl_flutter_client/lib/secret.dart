class Secret {
  static final Secret _instance = Secret._internal();
  static final String aesKey = "MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2Nzg=";

  factory Secret() {
    return _instance;
  }

  Secret._internal();
}
