#![cfg_attr(not(feature = "std"), no_std)]

/// Account-Based Stateless Blockchain
/// ***DISCLOSURE*** This module is incomplete, untested, and completely experimental.

use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, StorageValue, traits::Get};
use system::ensure_signed;
use codec::{Encode, Decode};
use accumulator::{U2048, Witness};
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
		WitnessData: Vec<(U2048, U2048)>;
        NewKeyValuePairs: Vec<(u8, u8)>
	}
}

decl_event!(
	pub enum Event {
		TokensMinted(U2048, U2048),
        Deletion(U2048, U2048, U2048),
        Addition(U2048, U2048, U2048),
	}
);


// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
		const KeySpace: u8 = T::KeySpace::get();

		pub fn mint(origin, key: u8, amount: u8) -> Result {
		    ensure_signed(origin)?;
		    let (state, product) = vc::commit(State::get(), &[key as usize], &[amount]);
		    State::put(state);
		    Self::deposit_event(Event::TokensMinted(state, product));
		    Ok(())
		}

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

            // Temporarily store the existing key-value pairs
            NewKeyValuePairs::append(&vec![(transaction.sender_key, transaction.sender_balance)]);
            NewKeyValuePairs::append(&vec![(transaction.receiver_key, transaction.receiver_balance)]);

		    Ok(())
		}

        fn on_finalize() {
            // Remove previous key-value commitment.
            let (state, product, proof) = accumulator::batch_delete(State::get(), &WitnessData::get());
            Self::deposit_event(Event::Deletion(state, product, proof));

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

    use primitives::{H256, Blake2Hasher};
    use support::{impl_outer_origin, assert_ok, parameter_types};
    use sr_primitives::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, weights::Weight, Perbill,
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

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }




}