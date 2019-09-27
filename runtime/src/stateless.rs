/// This runtime implements a stateless blockchain on Substrate using RSA accumulators. The code is meant to be
/// experimental and is far from production quality. The following code has not been checked for correctness nor has
/// been optimized for efficiency.

use support::{decl_module, decl_storage, decl_event, ensure, StorageValue, dispatch::Result, traits::Get, print};
use system::{ensure_signed, ensure_root};
use primitive_types::U256;
use rstd::prelude::Vec;
use sr_primitives::{ApplyResult, ApplyOutcome};
use accumulator;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type RsaModulus: Get<U256>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Stateless {
		State get(get_state): U256 = U256::from(2);  // Use 2 as an arbitrary generator with "unknown" order.
		SpentCoins get(get_spent_coins): Vec<(U256, U256)>;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initialize generic event
		fn deposit_event() = default;

		// Declare RSA modulus constant
		const RsaModulus: U256 = T::RsaModulus::get();

		/// Receive request to spend a coin.
		pub fn spend(origin, elem: U256, witness: U256) -> Result {
			ensure_signed(origin)?;
			Ok(())
		}

        /// Batch delete spent coins on block finalization
        fn on_finalize() {
            let (state, proof) = Self::delete(&SpentCoins::get());
            State::put(state);
            SpentCoins::mutate(|n| n.clear());
        }
	}
}

impl<T: Trait> Module<T> {
    // Temporarily add a coin.
    pub fn add_spent_coin(elem: U256, witness: U256) -> ApplyResult  {
        // Arbitrarily cap the number of pending transactions to 100
        // Add edge case for full block error
        if SpentCoins::get().len() > 100 || !(Self::verify_coin(elem, witness)) {
            return Ok(ApplyOutcome::Fail);
        }

        SpentCoins::mutate(|n| n.push((elem, witness)));
        Ok(ApplyOutcome::Success)
    }

    // Modify this include PoE
    pub fn verify_coin(elem: U256, witness: U256) -> bool {
        return accumulator::subroutines::mod_exp(witness, elem, T::RsaModulus::get()) == State::get();
    }

    // Aggregate a set of accumulator elements and witnesses in order to batch delete them from the accumulator.
    // Returns the state after deletion and proof of exponentiation.
    pub fn delete(elems: &Vec<(U256, U256)>) -> (U256, U256) {
        let (mut x_agg, mut new_state) = elems[0];
        for i in 1..elems.len() {
            let (x, witness) = elems[i];
            new_state = accumulator::subroutines::shamir_trick(new_state, witness, x_agg, x, T::RsaModulus::get()).unwrap();
            x_agg *= x;  // Should this be mod?
        }
        let proof = accumulator::proofs::poe(new_state, x_agg, State::get(), T::RsaModulus::get());
        return (new_state, proof);
    }

    // Aggregates a set of accumulator elements and witnesses in order to batch add them to the accumulator.
    // Returns the state after addition and a proof of exponentiation.
    pub fn add(elems: &Vec<U256>) -> (U256, U256) {
        let mut x_agg = U256::from(1);
        for i in 0..elems.len() {
            x_agg *= elems[i];
        }

        let new_state = accumulator::subroutines::mod_exp(State::get(), x_agg, T::RsaModulus::get());
        let proof = accumulator::proofs::poe(State::get(), x_agg, new_state, T::RsaModulus::get());
        State::put(new_state);
        return (new_state, proof);
    }
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		SomethingStored(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup, OnFinalize}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

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
		pub const RsaModulus: U256 = U256::from(13);
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
		type RsaModulus = RsaModulus;
	}

	type Stateless = Module<Test>;
    type System = system::Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	/// TEST ADDING/DELETING
	#[test]
	fn test_add() {
		with_externalities(&mut new_test_ext(), || {
			let elems = vec![U256::from(3), U256::from(5), U256::from(7)];
			Stateless::add(&elems);
			assert_eq!(Stateless::get_state(), U256::from(5));
		});
	}

	#[test]
	fn test_del() {
		with_externalities(&mut new_test_ext(), || {
			let elems = vec![U256::from(3), U256::from(5), U256::from(7)];
			Stateless::add(&elems);

            // Make sure to test wil invalid spends
			assert_ok!(Stateless::spend(Origin::signed(1), U256::from(3), U256::from(7)));
			assert_ok!(Stateless::spend(Origin::signed(1), U256::from(5), U256::from(5)));
			assert_ok!(Stateless::spend(Origin::signed(1), U256::from(7), U256::from(8)));

            Stateless::on_finalize(System::block_number());

            assert_eq!(Stateless::get_state(), U256::from(2));

		});
	}
}