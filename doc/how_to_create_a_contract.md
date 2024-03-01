# How to create a contract to fuzz

There are a few things to do in order to fuzz a contract, be it in stateless or in stateful.

## In stateless

In stateless fuzzing, you're testing the contract without considering any previous state. This means that the contract's behavior doesn't depend on any past interactions.

You have to create the function you want to fuzz, the function needs to take only basic types as parameters.

To create a contract follow the sui tutorial [here](https://docs.sui.io/guides/developer/first-app/write-package).

Here is an example:

```rust
module fuzzinglabs_package::fuzzinglabs_module {
  use std::vector;

  // Crash if input is "fuzzinglabs"
  fun fuzzinglabs(str: vector<u8>) {
    if (vector::length(&str) == 11) {
      if (*vector::borrow(&str, 0) == 0x66) { // f
        if (*vector::borrow(&str, 1) == 0x75) { // u
          // Check for other characters in a similar manner
          // ...
          if (*vector::borrow(&str, 10) == 0x73) { // s
            assert!(1 == 0, 42); // Trigger assertion failure
          }
        }
      }
    }
  }
}
```

Once you've created the contract build it using the following command (while in the root directory of the contract):

```bash
$ sui move build
```

You can then start the fuzzer, follow the instructions in **./doc/how_to_use_stateless.md** to find out how.

> Stateless fuzzing of a contract is faster but less comprehensive as it operates without considering past states, whereas stateful fuzzing, by accounting for the contract's evolving state over multiple transactions, provides a more thorough exploration of potential vulnerabilities and behaviors, albeit with increased complexity.

## In stateful

Stateful fuzzing involves testing a contract while considering its state changes over time. This includes interactions with previous transactions and changes to contract state.

You can create a contract using the tutorial mentionned earlier, there are a few more steps to do in stateful as in stateless.

The complete code of the example below in *./examples/calculator_package*.

You need to create a **fuzz_init** function where every object used in the fuzzing targets should be created.

Here is an example:

```rust
public fun fuzz_init(ctx: &mut TxContext) {
    let calc = Calc {
        id: object::new(ctx),
        sum: 0
    };
    transfer::transfer(calc, tx_context::sender(ctx))
}
```

Create one or more functions prefixed with **fuzz_** that contains the fuzzing logic. These functions typically take the contract's state as an argument and perform checks or operations on it, like so:

```rust
public fun fuzz_check(calc: &mut Calc, _ctx: &mut TxContext) {
    assert!(calc.sum == 42, calc.sum);
}
```

You can then start the fuzzer and it will detect all the **fuzz_** functions. (Documentation: **./doc/how_to_use_stateful.md**)