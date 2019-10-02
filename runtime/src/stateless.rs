/// PROJECT: Stateless Blockchain Experiment.
///
/// DESCRIPTION: This repository implements a UTXO-based stateless blockchain on Substrate using an
/// RSA accumulator. In this scheme, validators only need to track a single accumulator value and
/// users only need to store their own UTXOs and membership proofs. Unless a data service provider is
/// used, users must constantly watch for updates to the accumulator in order to update their proofs.
/// This particular implementation includes batching and aggregation techniques from this paper:
/// https://eprint.iacr.org/2018/1188.pdf.
///
/// NOTE: This repository is experimental and is not meant to be used in production. The design choices
/// made in this runtime are impractical from both a security and usability standpoint. Additionally,
/// the following code has not been checked for correctness nor has been optimized for efficiency.

use support::{decl_module, decl_storage, decl_event, ensure, StorageValue, dispatch::Result, traits::Get, print};
use system::{ensure_signed, ensure_root};
use primitive_types::{U256, H256};
use rstd::prelude::Vec;
use sr_primitives::{ApplyResult, ApplyOutcome};
use codec::{Encode, Decode};
use accumulator;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, Copy)]
pub struct UTXO {
    pub pub_key: H256,
    pub id: u64,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, Copy)]
pub struct Transaction {
    pub input: UTXO,
    pub output: UTXO,
    pub witness: U256,
    pub proof: U256,
    // Need to add signature here
}

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Stateless {
        State get(get_state): U256 = U256::from(2);  // Use 2 as an arbitrary generator with "unknown" order.
        SpentCoins get(get_spent_coins): Vec<(U256, U256)>;
        NewCoins get(get_new_coins): Vec<U256>
    }
}

decl_event!(
    pub enum Event {
        Deletion(U256, U256, U256),
        Addition(U256, U256, U256),
    }
);

decl_module! {
	/// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initialize generic event
        fn deposit_event() = default;
	    // Declare RSA modulus constant

        /// Receive request to execute a transaction.
        /// NOTE: Only works if one transaction per user per block is submitted.
        pub fn add_transaction(origin, transaction: Transaction) -> Result {
            ensure_signed(origin)?;
            Ok(())
        }

        /// Batch delete spent coins and add new coins on block finalization
        fn on_finalize() {
            // Delete spent coins from aggregator and distribute proof
            let (state, agg, proof) = Self::delete(&SpentCoins::get());
            Self::deposit_event(Event::Deletion(state, agg, proof));

            runtime_io::print(state.low_u64());

            // Add new coins to aggregator and distribute proof
            let (state, agg, proof) = Self::add(&NewCoins::get());
            Self::deposit_event(Event::Addition(state, agg, proof));

            runtime_io::print(state.low_u64());

            // Clear storage
            SpentCoins::mutate(|n| n.clear());
            NewCoins::mutate(|n| n.clear());
        }
    }
}

impl<T: Trait> Module<T> {
    /// Verify the contents of a transaction and temporarily add it to a queue of verified transactions.
    /// This function will evolve as more implementation details related to transactions are added.
    pub fn verify_transaction(transaction: Transaction) -> ApplyResult  {
        // Arbitrarily cap the number of pending transactions to 100
        // Also verify that the user is not spending to themselves
        if SpentCoins::get().len() > 100 || transaction.input.pub_key == transaction.output.pub_key {
            return Ok(ApplyOutcome::Fail);
        }

        // Verify witness
        let spent_elem = accumulator::subroutines::hash_to_prime(&transaction.input.encode());
        if !(accumulator::witnesses::verify_mem_wit(Self::get_state(),
                                                    spent_elem, transaction.witness, transaction.proof)) {
            return Ok(ApplyOutcome::Fail);
        }

        let new_elem = accumulator::subroutines::hash_to_prime(&transaction.output.encode());

        SpentCoins::mutate(|v| v.push((spent_elem, transaction.witness)));
        NewCoins::mutate(|v| v.push(new_elem));
        Ok(ApplyOutcome::Success)
    }

    /// Aggregates a set of accumulator elements + witnesses and batch deletes them from the accumulator.
    /// Returns the state after deletion, the product of the deleted elements, and a proof of exponentiation.
    pub fn delete(elems: &Vec<(U256, U256)>) -> (U256, U256, U256) {
        let (mut x_agg, mut new_state) = elems[0];
        for i in 1..elems.len() {
            let (x, witness) = elems[i];
            new_state = accumulator::subroutines::shamir_trick(new_state, witness, x_agg, x).unwrap();
            x_agg *= x;
        }
        let proof = accumulator::proofs::poe(new_state, x_agg, State::get());
        State::put(new_state);
        return (new_state, x_agg, proof);
    }

    /// Aggregates a set of accumulator elements + witnesses and batch adds them to the accumulator.
    /// Returns the state after addition, the product of the added elements, and a proof of exponentiation.
    pub fn add(elems: &Vec<U256>) -> (U256, U256, U256) {
        let mut x_agg = U256::from(1);
        for i in 0..elems.len() {
            x_agg *= elems[i];
        }

        let new_state = accumulator::subroutines::mod_exp(State::get(), x_agg, U256::from(accumulator::MODULUS));
        let proof = accumulator::proofs::poe(State::get(), x_agg, new_state);
        State::put(new_state);
        return (new_state, x_agg, proof);
    }

}

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
    }

    type Stateless = Module<Test>;
    type System = system::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

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
            // Collect witnesses for the added elements
            let witnesses = accumulator::witnesses::create_all_mem_wit(Stateless::get_state(), &elems);

            // Add elements
            Stateless::add(&elems);
            assert_eq!(Stateless::get_state(), U256::from(5));

            // Delete elements
            let deletions = vec![(elems[0], witnesses[0]), (elems[1], witnesses[1]), (elems[2], witnesses[2])];
            Stateless::delete(&deletions);
            assert_eq!(Stateless::get_state(), U256::from(2));
        });
    }

    #[test]
    fn test_block() {
        with_externalities(&mut new_test_ext(), || {
            // 1. Construct UTXOs.
            let utxo_0 = UTXO {
                pub_key: H256::from_low_u64_be(0),
                id: 0,
            };

            let utxo_1 = UTXO {
                pub_key: H256::from_low_u64_be(1),
                id: 1,
            };

            let utxo_2 = UTXO {
                pub_key: H256::from_low_u64_be(2),
                id: 2,
            };

            // 2. Hash each UTXO to a prime.
            let elem_0 = accumulator::subroutines::hash_to_prime(&utxo_0.encode());
            let elem_1 = accumulator::subroutines::hash_to_prime(&utxo_1.encode());
            let elem_2 = accumulator::subroutines::hash_to_prime(&utxo_2.encode());
            let elems = vec![elem_0, elem_1, elem_2];

            // 3. Produce witnesses for the added elements.
            let witnesses = accumulator::witnesses::create_all_mem_wit(Stateless::get_state(), &elems);

            // 4. Add elements to the accumulator.
            Stateless::add(&elems);

            // 5. Construct new UTXOs and derive integer representations.
            let utxo_3 = UTXO {
                pub_key: H256::from_low_u64_be(1),
                id: 0,
            };

            let utxo_4 = UTXO {
                pub_key: H256::from_low_u64_be(2),
                id: 1,
            };

            let utxo_5 = UTXO {
                pub_key: H256::from_low_u64_be(0),
                id: 2,
            };

            let elem_3 = accumulator::subroutines::hash_to_prime(&utxo_3.encode());
            let elem_4 = accumulator::subroutines::hash_to_prime(&utxo_4.encode());
            let elem_5 = accumulator::subroutines::hash_to_prime(&utxo_5.encode());

            // 6. Construct transactions.
            let tx_0 = Transaction {
                input: utxo_0,
                output: utxo_3,
                witness: witnesses[0],
                proof: accumulator::proofs::poe(witnesses[0], elem_0, Stateless::get_state()),
            };

            let tx_1 = Transaction {
                input: utxo_1,
                output: utxo_4,
                witness: witnesses[1],
                proof: accumulator::proofs::poe(witnesses[1], elem_1, Stateless::get_state()),
            };

            let tx_2 = Transaction {
                input: utxo_2,
                output: utxo_5,
                witness: witnesses[2],
                proof: accumulator::proofs::poe(witnesses[2], elem_2, Stateless::get_state()),
            };

            // 7. Verify transactions. Note that this logic will eventually be executed automatically
            // by the block builder API eventually.
            assert_eq!(Stateless::verify_transaction(tx_0).unwrap(), ApplyOutcome::Success);
            assert_eq!(Stateless::verify_transaction(tx_1).unwrap(), ApplyOutcome::Success);
            assert_eq!(Stateless::verify_transaction(tx_2).unwrap(), ApplyOutcome::Success);

            // 8. Finalize the block.
            Stateless::on_finalize(System::block_number());

            assert_eq!(Stateless::get_state(),
                       accumulator::subroutines::mod_exp(U256::from(2), elem_3 * elem_4 * elem_5, U256::from(accumulator::MODULUS)));

        });
    }




}