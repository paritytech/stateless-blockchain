#![cfg_attr(not(feature = "std"), no_std)]

/// Account-Based Stateless Blockchain
/// ***DISCLOSURE*** This module is incomplete, untested, and completely experimental.

use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, StorageValue, traits::Get};
use system::ensure_signed;
use codec::{Encode, Decode};
use accumulator::*;
pub mod binary;
pub mod vc;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Transaction {
    sender_key: u8,
    sender_balance: u8,
    sender_elem: U2048,
    sender_opening: (Witness, Witness),
    receiver_key: u8,
    receiver_balance: u8,
    receiver_elem: U2048,
    receiver_opening: (Witness, Witness),
    amount: u8,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
    type KeySpace: Get<u8>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as StatelessAccounts {
        State get(get_state): U2048 = U2048::from(2);  // Use 2 as an arbitrary generator with "unknown" order.
        WitnessData get(get_witness_data): Vec<(U2048, U2048)>;
        NewKeyValuePairs: Vec<(u8, u8)>;
    }
}

decl_event!(
    pub enum Event {
        TokensMinted(U2048, U2048),
        Deletion(U2048, U2048, U2048),
        Addition(U2048, U2048, U2048),
    }
);

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
        const KeySpace: u8 = T::KeySpace::get();

        /// Arbitrarily add a new key-value store to the accumulator.
        /// NOTE: The key must not exist initially.
        pub fn mint(origin, key: u8, amount: u8) -> Result {
            ensure_signed(origin)?;
            let (state, product) = vc::commit(State::get(), &[key as usize], &[amount]);
            State::put(state);
            Self::deposit_event(Event::TokensMinted(state, product));
            Ok(())
        }

        /// Submit a transaction to the chain.
        /// NOTE: All transactions must be referenced from the same previous "state". In practice,
        /// this might be the state of the previous block for example. This is a workaround to
        /// prevent having to pass in the product of all of the elements in the accumulator.
        pub fn add_transaction(origin, transaction: Transaction, old_state: U2048) -> Result {
            ensure_signed(origin)?;
            // Get the opening of the sender
            let (pi_i_sender, pi_e_sender) = transaction.sender_opening;

            // Verify that it is valid
            ensure!(vc::verify_at_key(old_state, State::get(), transaction.sender_key as usize,
            transaction.sender_balance, pi_i_sender, pi_e_sender), "Opening is invalid.");

            // Ensure that the sender isn't spending more than balance
            ensure!(transaction.sender_balance >= transaction.amount, "User is trying to spend more than balance.");

            // Verify receiver opening
            let (pi_i_receiver, pi_e_receiver) = transaction.receiver_opening;
            ensure!(vc::verify_at_key(old_state, State::get(), transaction.receiver_key as usize,
                    transaction.receiver_balance, pi_i_receiver, pi_e_receiver), "Opening is invalid.");

            // Add membership proofs to temporary vector to be processed later
            if let Witness::MemWit(sender_witness) = pi_i_sender {
                WitnessData::append(&vec![(transaction.sender_elem, sender_witness)]);
            }

            if let Witness::MemWit(receiver_witness) = pi_i_receiver {
                WitnessData::append(&vec![(transaction.receiver_elem, receiver_witness)]);
            }

            // Currently omitting non-membership proofs for simplicity

            // Temporarily store the new key-value pairs
            NewKeyValuePairs::append(&vec![(transaction.sender_key, transaction.sender_balance-transaction.amount)]);
            NewKeyValuePairs::append(&vec![(transaction.receiver_key, transaction.receiver_balance+transaction.amount)]);
            Ok(())
        }

        fn on_finalize() {
            // Remove previous key-value commitment.
            let (state, product, proof) = accumulator::batch_delete(State::get(), &WitnessData::get());
            Self::deposit_event(Event::Deletion(state, product, proof));

            // Get the integer representations of the new key-value pairs.
            let elems: Vec<U2048> = NewKeyValuePairs::get()
                .into_iter()
                .enumerate()
                .map(|(_, (key, value))| -> U2048 {
                    let (binary_vec, indices) = vc::convert_key_value(&[key as usize], &[value]);
                    let (p_ones, _) = binary::get_bit_elems(&binary_vec, &indices);
                    return p_ones;
                })
                .collect();

            // Add updated key-value pairs.
            let (state, product, proof) = accumulator::batch_add(state, &elems);
            Self::deposit_event(Event::Addition(state, product, proof));

            // Update accumulator
            State::put(state);

            // Clear storage items
            WitnessData::kill();
            NewKeyValuePairs::kill();
        }
    }
}

/// Tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use runtime_io::with_externalities;
    use primitives::{H256, Blake2Hasher};
    use support::{impl_outer_origin, assert_ok, parameter_types};
    use sr_primitives::{
        traits::{BlakeTwo256, IdentityLookup, OnFinalize}, testing::Header, weights::Weight, Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
        pub const KeySpace: u8 = 255;
    }

    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }

    impl Trait for Test {
        type Event = ();
        type KeySpace = KeySpace;
    }

    type StatelessAccounts = Module<Test>;
    type System = system::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

    #[test]
    fn test_mint() {
        with_externalities(&mut new_test_ext(), || {
            let key: u8 = 1;
            let value: u8 = 10;
            StatelessAccounts::mint(Origin::signed(1), key, value);

            let (binary_vec, indices) = vc::convert_key_value(&[key as usize], &[value]);
            let (p_ones, _) = binary::get_bit_elems(&binary_vec, &indices);
            assert_eq!(StatelessAccounts::get_state(), subroutines::mod_exp(U2048::from(2), p_ones, U2048::from_dec_str(MODULUS).unwrap()));
        });
    }

    #[test]
    fn test_transaction() {
        with_externalities(&mut new_test_ext(), || {
            let generator = StatelessAccounts::get_state();

            // Define keys for alice and bob
            let alice_key: u8 = 12;
            let bob_key: u8 = 58;

            // Define balances for alice and bob
            let alice_balance: u8 = 10;
            let bob_balance: u8 = 5;

            // Mint tokens for each user
            StatelessAccounts::mint(Origin::signed(1), alice_key, alice_balance);
            StatelessAccounts::mint(Origin::signed(1), bob_key, bob_balance);

            // Derive integer representations for manual testing
            let alice_elem = vc::get_key_value_elem(alice_key as usize, alice_balance);  // This value would be received from the emitted event.
            let bob_elem = vc::get_key_value_elem(bob_key as usize, bob_balance);   // This value would be received from the emitted event.
            let product = alice_elem * bob_elem;

            // Get state after minting
            let state_after_mint = StatelessAccounts::get_state();

            // Get openings for each user
            let (alice_pi_i, alice_pi_e) = vc::open_at_key(generator, product, alice_key as usize, alice_balance);
            let (bob_pi_i, bob_pi_e) = vc::open_at_key(generator, product, bob_key as usize, bob_balance);

            // Construct transaction
            let transaction = Transaction {
                sender_key: alice_key,
                sender_balance: alice_balance,
                sender_elem: alice_elem,
                sender_opening: (alice_pi_i, alice_pi_e),
                receiver_key: bob_key,
                receiver_balance: bob_balance,
                receiver_elem: bob_elem,
                receiver_opening: (bob_pi_i, bob_pi_e),
                amount: 3,
            };

            // Submit transaction
            StatelessAccounts::add_transaction(Origin::signed(1), transaction, generator);

            // Manually get the state after deletion for manual testing
            let (state_after_del, _, _) = batch_delete(state_after_mint, &StatelessAccounts::get_witness_data());

            // Finalize block
            StatelessAccounts::on_finalize(System::block_number());

            // Get the new state
            let new_state = StatelessAccounts::get_state();

            // Derive integer representations for alice and bob's new key-value stores
            let new_alice_elem = vc::get_key_value_elem(alice_key as usize, alice_balance-3);  // This value would be received from the emitted event.
            let new_bob_elem = vc::get_key_value_elem(bob_key as usize, bob_balance+3);  // This value would be received from the emitted event.

            // Create openings with the new balances
            let (alice_pi_i_new, alice_pi_e_new) = vc::open_at_key(state_after_del, new_alice_elem*new_bob_elem, alice_key as usize, alice_balance-3);
            let (bob_pi_i_new, bob_pi_e_new) = vc::open_at_key(state_after_del, new_alice_elem*new_bob_elem, bob_key as usize, bob_balance+3);

            // Verify that the openings are valid
            assert_eq!(vc::verify_at_key(state_after_del, new_state, alice_key as usize, alice_balance-3, alice_pi_i_new, alice_pi_e_new), true);
            assert_eq!(vc::verify_at_key(state_after_del, new_state, bob_key as usize, bob_balance+3, bob_pi_i_new, bob_pi_e_new), true);
        });
    }
}
