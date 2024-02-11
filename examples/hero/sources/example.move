// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Example of a game character with basic attributes, inventory, and
/// associated logic.
module hero::example {
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use sui::event;
    use sui::object::{Self, ID, UID};
    use sui::math;
    use sui::sui::SUI;
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use std::option::{Self, Option};

    /// Our hero!
    struct Hero has key, store {
        id: UID,
        /// Game this hero is playing in.
        game_id: ID,
        /// Hit points. If they go to zero, the hero can't do anything
        health: u64,
        /// Experience of the hero. Begins at zero
        experience: u64,
        /// The hero's minimal inventory
        sword: Option<Sword>,
    }

    /// The hero's trusty sword
    struct Sword has key, store {
        id: UID,
        /// Game this sword is from.
        game_id: ID,
        /// Constant set at creation. Acts as a multiplier on sword's strength.
        /// Swords with high magic are rarer (because they cost more).
        magic: u64,
        /// Sword grows in strength as we use it
        strength: u64,
    }

    /// For healing wounded heroes
    struct Potion has key, store {
        id: UID,
        /// Game this potion is from.
        game_id: ID,
        /// Effectiveness of the potion
        potency: u64,
    }

    /// A creature that the hero can slay to level up
    struct Boar has key, store {
        id: UID,
        /// Game this boar is from.
        game_id: ID,
        /// Hit points before the boar is slain
        health: u64,
        /// Strength of this particular boar
        strength: u64,
    }

    /// Contains information about the game managed by a given `admin`.  Holds
    /// payments for player actions for the admin to collect.
    struct Game has key {
        id: UID,
        payments: Balance<SUI>,
    }

    /// Capability conveying the authority to create boars and potions, and take
    /// payments.
    struct Admin has key, store {
        id: UID,
        /// ID of the game this admin manages
        game_id: ID,
        /// Total number of boars the admin has created
        boars_created: u64,
        /// Total number of potions the admin has created
        potions_created: u64,
    }

    /// Event emitted each time a Hero slays a Boar
    struct BoarSlainEvent has copy, drop {
        /// Address of the user that slayed the boar
        slayer_address: address,
        /// ID of the now-deceased boar
        boar: ID,
        /// ID of the Hero that slayed the boar
        hero: ID,
        /// ID of the game where event happened
        game_id: ID,
    }

    // === Constants ===

    /// Upper bound on player's HP
    const MAX_HP: u64 = 1000;

    /// Upper bound on how magical a sword can be
    const MAX_MAGIC: u64 = 10;

    /// Minimum amount you can pay for a sword
    const MIN_SWORD_COST: u64 = 100;

    // === Error Codes ===

    /// Objects are from differing game instances.
    const EWrongGame: u64 = 0;

    /// The boar won the battle
    const EBoarWon: u64 = 1;

    /// The hero is too tired to fight
    const EHeroTired: u64 = 2;

    /// Trying to initialize from a non-admin account
    const ENotAdmin: u64 = 3;

    /// Not enough money to purchase the given item
    const EInsufficientFunds: u64 = 5;

    /// Trying to equip a sword but the hero already has one
    const EAlreadyEquipped: u64 = 6;

    /// Trying to remove a sword, but the hero does not have one
    const ENotEquipped: u64 = 7;

    // === Player Object creation ===

    /// It all starts with the sword. Anyone can buy a sword, and proceeds are
    /// stored in the `Game`. Amount of magic in the sword depends on how much
    /// you pay for it.
    public fun new_sword(
        game: &mut Game,
        payment: Coin<SUI>,
        ctx: &mut TxContext
    ): Sword {
        let value = coin::value(&payment);
        // ensure the user pays enough for the sword
        assert!(value >= MIN_SWORD_COST, EInsufficientFunds);

        // pay the game for this sword
        coin::put(&mut game.payments, payment);

        // magic of the sword is proportional to the amount you paid, up to
        // a max. one can only imbue a sword with so much magic
        let magic = (value - MIN_SWORD_COST) / MIN_SWORD_COST;
        Sword {
            id: object::new(ctx),
            magic: math::min(magic, MAX_MAGIC),
            strength: 1,
            game_id: object::id(game)
        }
    }

    /// Anyone can create a hero if they have a sword. All heroes start with the
    /// same attributes.
    public fun new_hero(sword: Sword, ctx: &mut TxContext): Hero {
        Hero {
            id: object::new(ctx),
            game_id: sword.game_id,
            health: 100,
            experience: 0,
            sword: option::some(sword),
        }
    }

    // === Admin Object creation ===

    /// Create a new `Game` (shared) and an `Admin` (returned) to run it. Anyone
    /// can run a game, all objects spawned by the game will be associated with
    /// it.
    public fun new_game(ctx: &mut TxContext): Admin {
        let game = Game {
            id: object::new(ctx),
            payments: balance::zero(),
        };

        let admin = Admin {
            id: object::new(ctx),
            game_id: object::id(&game),
            boars_created: 0,
            potions_created: 0,
        };

        transfer::share_object(game);
        admin
    }

    /// Admin can create a potion with the given `potency` for `recipient`
    public fun new_potion(
        admin: &mut Admin,
        potency: u64,
        ctx: &mut TxContext
    ): Potion {
        admin.potions_created = admin.potions_created + 1;
        Potion { id: object::new(ctx), potency, game_id: admin.game_id }
    }

    /// Admin can create a boar with the given attributes
    public fun new_boar(
        admin: &mut Admin,
        health: u64,
        strength: u64,
        ctx: &mut TxContext
    ): Boar {
        admin.boars_created = admin.boars_created + 1;
        Boar { id: object::new(ctx), health, strength, game_id: admin.game_id }
    }

    // === Gameplay ===

    /// Slay the `boar` with the `hero`'s sword, get experience.
    /// Aborts if the hero has 0 HP or is not strong enough to slay the boar
    public fun slay(hero: &mut Hero, boar: Boar, ctx: &TxContext) {
        assert!(hero.game_id == boar.game_id, EWrongGame);

        let Boar {
            id: boar_id,
            strength: boar_strength,
            health: boar_health,
            game_id: _
        } = boar;

        // Hero gains experience proportional to the boar's health.
        let experience = boar_health;

        // Attack the boar with the sword until its HP goes to zero.
        loop {
            let hero_strength = hero_strength(hero);

            // First, the hero attacks.
            if (boar_health < hero_strength) {
                break
            } else {
                boar_health = boar_health - hero_strength;
            };

            // Then, the boar gets a turn to attack.  If the boar would kill the
            // hero, abort -- we can't let the boar win!
            assert!(hero.health >= boar_strength, EBoarWon);
            hero.health = hero.health - boar_strength;
        };

        // Boar slain, level up the hero, and their sword if they have one.
        hero.experience = hero.experience + experience;
        if (option::is_some(&hero.sword)) {
            level_up_sword(option::borrow_mut(&mut hero.sword), 1)
        };

        // Let the world know about the hero's triumph by emitting an event!
        event::emit(BoarSlainEvent {
            slayer_address: tx_context::sender(ctx),
            hero: object::id(hero),
            boar: object::uid_to_inner(&boar_id),
            game_id: hero.game_id,
        });
        object::delete(boar_id);
    }

    /// Strength of the hero when attacking -- aborts if the hero cannot fight.
    public fun hero_strength(hero: &Hero): u64 {
        // A hero with zero HP is too tired to fight.
        assert!(hero.health > 0, EHeroTired);

        // Hero can fight without a sword, but will not be very strong.
        let sword_strength = if (option::is_some(&hero.sword)) {
            sword_strength(option::borrow(&hero.sword))
        } else {
            0
        };

        // Hero is weaker if they have lower HP.
        (hero.experience * hero.health) + sword_strength
    }

    fun level_up_sword(sword: &mut Sword, amount: u64) {
        sword.strength = sword.strength + amount
    }

    /// Strength of a sword when attacking.
    public fun sword_strength(sword: &Sword): u64 {
        sword.magic + sword.strength
    }

    // === Inventory ===

    /// Heal the weary hero with a potion.
    public fun heal(hero: &mut Hero, potion: Potion) {
        let Potion { id, potency, game_id } = potion;
        object::delete(id);

        assert!(hero.game_id == game_id, EWrongGame);

        // cap hero's HP at MAX_HP to avoid int overflows
        hero.health = math::min(hero.health + potency, MAX_HP)
    }

    /// Add `new_sword` to the hero's inventory and return the old sword
    /// (if any)
    public fun equip(hero: &mut Hero, sword: Sword) {
        assert!(option::is_none(&hero.sword), EAlreadyEquipped);
        option::fill(&mut hero.sword, sword)
    }

    /// Disarm the hero by returning their sword.
    /// Aborts if the hero does not have a sword.
    public fun unequip(hero: &mut Hero): Sword {
        assert!(option::is_some(&hero.sword), ENotEquipped);
        option::extract(&mut hero.sword)
    }

    // === Payments ===

    /// The owner of the `Admin` object can extract payment from the `Game`.
    fun take_payment(
        admin: &Admin,
        game: &mut Game,
        ctx: &mut TxContext,
    ): Coin<SUI> {
        assert!(admin.game_id == object::id(game), ENotAdmin);
        coin::from_balance(balance::withdraw_all(&mut game.payments), ctx)
    }

}
