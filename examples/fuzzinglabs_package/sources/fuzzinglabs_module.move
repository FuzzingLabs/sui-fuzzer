module fuzzinglabs_package::fuzzinglabs_module {
  use std::vector;

  // Crash if input is fuzzinglabs
  fun fuzzinglabs(str: vector<u8>) {
    if (vector::length(&str) == 11) {
      if (*vector::borrow(&str, 0) == 0x66) { // f
        if (*vector::borrow(&str, 1) == 0x75) { // u
          if (*vector::borrow(&str, 2) == 0x7a) { // z
            if (*vector::borrow(&str, 3) == 0x7a) { // z
              if (*vector::borrow(&str, 4) == 0x69) { // i
                if (*vector::borrow(&str, 5) == 0x6e) { // n
                  if (*vector::borrow(&str, 6) == 0x67) { // g
                    if (*vector::borrow(&str, 7) == 0x6c) { // l
                      if (*vector::borrow(&str, 8) == 0x61) { // a
                        if (*vector::borrow(&str, 9) == 0x62) { // b
                          if (*vector::borrow(&str, 10) == 0x73) { // s
                            assert!(1 == 0, 42)
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
