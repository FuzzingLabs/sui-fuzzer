module calculator_package::calculator_module {

  use sui::object::{Self, UID};
  use sui::tx_context::{Self, TxContext};
  use sui::transfer;

  struct Calc has key, store {
    id: UID,
    sum: u64
  }

  public fun fuzz_init(ctx: &mut TxContext) {
    let calc = Calc {
      id: object::new(ctx),
      sum: 0
    };
    transfer::transfer(calc, tx_context::sender(ctx))
  }

  public fun add(a: u64, calc: &mut Calc, _ctx: &mut TxContext) {
    calc.sum = calc.sum + a;
  }

  public fun sub(a: u64, calc: &mut Calc, _ctx: &mut TxContext) {
    calc.sum = calc.sum - a;
  }

  public fun fuzz_check(calc: &mut Calc, _ctx: &mut TxContext) {
    assert!(calc.sum == 42, calc.sum);
  }

}