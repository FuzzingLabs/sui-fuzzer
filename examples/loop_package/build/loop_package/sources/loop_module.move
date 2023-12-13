module loop_package::loop_module {

  public entry fun fuzz_loop(value: u16) {
        let i = 0;
        while (i < 10) {
            i = i + 1;
        };
        let i = 0;
        while (i < value) {
            i = i + 1;
        };
  }

}