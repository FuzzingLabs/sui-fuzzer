module arithmetic_errors_package::arithmetic_errors_module {

  public entry fun fuzz_u64_overflow(a: u64, b: u64) {
    a * b;
  }

}