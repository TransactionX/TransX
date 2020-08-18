// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! # Nicks Module
//!
//! - [`nicks::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//!
//! Nicks is a trivial module for keeping track of account names on-chain. It makes no effort to
//! create a name hierarchy, be a DNS replacement or provide reverse lookups.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `set_name` - Set the associated name of an account; a small deposit is reserved if not already
//!   taken.
//! * `clear_name` - Remove an account's associated name; the deposit is returned.
//! * `kill_name` - Forcibly remove the associated name; the deposit is lost.
//!
//! [`Call`]: ./enum.Call.html
//! [`Trait`]: ./trait.Trait.html

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_runtime::{
	traits::{StaticLookup, Zero}
};
use frame_support::{
	decl_module, decl_event, decl_storage, ensure, decl_error,
	traits::{Currency, ReservableCurrency, OnUnbalanced, Get, EnsureOrigin},
	weights::Weight,
};
use frame_system::{self as system, ensure_signed, ensure_root};

type BalanceOf<T> = <<T as Trait>::Currency_n as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Trait>::Currency_n as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The currency trait.
	type Currency_n: ReservableCurrency<Self::AccountId>;

	/// Reservation fee.
	type ReservationFee: Get<BalanceOf<Self>>;  // 1 token

	/// What to do with slashed funds.
	type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

	/// The origin which may forcibly set or remove a name. Root can always do this.
	type ForceOrigin: EnsureOrigin<Self::Origin>;

	/// The minimum length a name may be.
	type MinLength: Get<usize>;

	/// The maximum length a name may be.
	type MaxLength: Get<usize>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Sudo {

		/// 每个account_id对应的名字
		NameOf: map hasher(blake2_128_concat) T::AccountId => Option<(Vec<u8>, BalanceOf<T>)>;

		/// 每个名字对应的account_id
		pub AccountIdOf: map hasher(blake2_128_concat) Vec<u8> => T::AccountId;
	}
}

decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {
		/// 名字太短
		NameTooShort,

		/// 名字太长
		NameTooLong,

		/// 已经存在的名字
		ExistsName,

		/// 不是已经存在的名字
		NotExistsName,

		/// 不是允许的源
		BabOrigin,

		///已经设置名字
		AlreadySetName,

	}
	}


decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, Balance = BalanceOf<T> {
		/// A name was set.
		NameSet(AccountId),
		/// A name was forcibly set.
		NameForced(AccountId),
		/// A name was changed.
		NameChanged(AccountId),
		/// A name was cleared, and the given balance returned.
		NameCleared(AccountId, Balance),
		/// A name was removed and the given balance slashed.
		NameKilled(AccountId, Balance),
	}
);



decl_module! {
	// Simple declaration of the `Module` type. Lets the macro know what it's working on.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;
		fn deposit_event() = default;

		/// Reservation fee.
		const ReservationFee: BalanceOf<T> = T::ReservationFee::get();

		/// The minimum length a name may be.
		const MinLength: u32 = T::MinLength::get() as u32;

		/// The maximum length a name may be.
		const MaxLength: u32 = T::MaxLength::get() as u32;

		/// Set an account's name. The name should be a UTF-8-encoded string by convention, though
		/// we don't check it.
		///
		/// The name may not be more than `T::MaxLength` bytes, nor less than `T::MinLength` bytes.
		///
		/// If the account doesn't already have a name, then a fee of `ReservationFee` is reserved
		/// in the account.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - At most one balance operation.
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[weight = 50_000]
		fn set_name(origin, name: Vec<u8>) {
			let sender = ensure_signed(origin)?;

			ensure!(name.len() >= T::MinLength::get(), Error::<T>::NameTooShort);
			ensure!(name.len() <= T::MaxLength::get(), Error::<T>::NameTooLong);

			// 名字不能用相同
			ensure!(!<AccountIdOf<T>>::contains_key(name.clone()),Error::<T>::ExistsName);
			ensure!(!<NameOf<T>>::contains_key(&sender), Error::<T>::AlreadySetName);

			let deposit = T::ReservationFee::get();
			T::Currency_n::reserve(&sender, deposit.clone())?;
			Self::deposit_event(RawEvent::NameChanged(sender.clone()));

			<NameOf<T>>::insert(&sender, (name.clone(), deposit));
			<AccountIdOf<T>>::insert(name.clone(), sender.clone());
		}


		/// Remove an account's name and take charge of the deposit.
		///
		/// Fails if `who` has not been named. The deposit is dealt with through `T::Slashed`
		/// imbalance handler.
		///
		/// The dispatch origin for this call must be _Root_ or match `T::ForceOrigin`.
		///
		/// # <weight>
		/// - O(1).
		/// - One unbalanced handler (probably a balance transfer)
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[weight = 70_000]
		// 这个方法估计是只有议会成员与root才能执行
		// 惩罚掉并且扣除他的押金
		fn kill_name(origin, target: <T::Lookup as StaticLookup>::Source) {

			ensure_root(origin)?;

			// Figure out who we're meant to be clearing.
			let target = T::Lookup::lookup(target)?;
			// Grab their deposit (and check that they have one).
			let account_info = <NameOf<T>>::take(&target).ok_or(Error::<T>::NotExistsName)?;
			let deposit = account_info.1;
			let name = account_info.0;
			<AccountIdOf<T>>::remove(name);

			// Slash their deposit from them.
			T::Slashed::on_unbalanced(T::Currency_n::slash_reserved(&target, deposit.clone()).0);

			Self::deposit_event(RawEvent::NameKilled(target, deposit));
		}

		/// Set a third-party account's name with no deposit.
		///
		/// No length checking is done on the name.
		///
		/// The dispatch origin for this call must be _Root_ or match `T::ForceOrigin`.
		///
		/// # <weight>
		/// - O(1).
		/// - At most one balance operation.
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[weight = 70_000]
		// 这个方法估计是只有议会成员与root才能执行
		// 不需要对名字长度进行检查  并且对方如果之前有命名过才会抵押他的金额 否则强制命名
		fn force_name(origin, target: <T::Lookup as StaticLookup>::Source, name: Vec<u8>) {

			ensure_root(origin)?;

			let target = T::Lookup::lookup(target)?;

			let deposit = <NameOf<T>>::get(&target).map(|x| x.1).unwrap_or_else(Zero::zero);
			<NameOf<T>>::insert(&target, (name.clone(), deposit));

			// 如果这个名字已经被占用
			if let old_id = <AccountIdOf<T>>::get(name.clone()) {
					// 如果不是他本人占用
					if old_id.clone() != target.clone(){
						if let Some(account_info) = <NameOf<T>>::get(&old_id){

						let old_name = account_info.clone().0;
						let old_deposit = account_info.clone().1;

						<AccountIdOf<T>>::remove(old_name.clone());

						// 归还old_name抵押
						let _ = T::Currency_n::unreserve(&old_id, old_deposit.clone());

						 <NameOf<T>>::remove(&old_id);

			}
					}

			}

			<AccountIdOf<T>>::insert(name.clone(), target.clone());

			Self::deposit_event(RawEvent::NameForced(target));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use support::{assert_ok, assert_noop, impl_outer_origin, parameter_types, weights::Weight};
	use primitives::H256;
	use system::EnsureSignedBy;
	// The testing primitives are very useful for avoiding having to work with signatures
	// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.
	use sp_runtime::{
		Perbill, testing::Header, traits::{BlakeTwo256, IdentityLookup},
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
		pub const AvailableBlockRatio: Perbill = Perbill::one();
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Call = ();
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	parameter_types! {
		pub const ExistentialDeposit: u64 = 0;
		pub const TransferFee: u64 = 0;
		pub const CreationFee: u64 = 0;
	}
	impl balances::Trait for Test {
		type Balance = u64;
		type OnFreeBalanceZero = ();
		type OnNewAccount = ();
		type Event = ();
		type TransferPayment = ();
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type TransferFee = TransferFee;
		type CreationFee = CreationFee;
	}
	parameter_types! {
		pub const ReservationFee: u64 = 2;
		pub const MinLength: usize = 3;
		pub const MaxLength: usize = 16;
		pub const One: u64 = 1;
	}
	impl Trait for Test {
		type Event = ();
		type Currency_n = Balances;
		type ReservationFee = ReservationFee;
		type Slashed = ();
		type ForceOrigin = EnsureSignedBy<One, u64>;
		type MinLength = MinLength;
		type MaxLength = MaxLength;
	}
	type Balances = balances::Module<Test>;
	type Nicks = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
		// We use default for brevity, but you can configure as desired if needed.
		balances::GenesisConfig::<Test> {
			balances: vec![
				(1, 10),
				(2, 10),
			],
			vesting: vec![],
		}.assimilate_storage(&mut t).unwrap();
		t.into()
	}

	#[test]
	fn kill_name_should_work() {
		new_test_ext().execute_with(|| {
			assert_ok!(Nicks::set_name(Origin::signed(2), b"Dave".to_vec()));
			assert_eq!(Balances::total_balance(&2), 10);
			assert_ok!(Nicks::kill_name(Origin::signed(1), 2));
			assert_eq!(Balances::total_balance(&2), 8);
			assert_eq!(<NameOf<Test>>::get(2), None);
		});
	}

	#[test]
	fn force_name_should_work() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Nicks::set_name(Origin::signed(2), b"Dr. David Brubeck, III".to_vec()),
				"Name too long"
			);

			assert_ok!(Nicks::set_name(Origin::signed(2), b"Dave".to_vec()));
			assert_eq!(Balances::reserved_balance(&2), 2);
			assert_ok!(Nicks::force_name(Origin::signed(1), 2, b"Dr. David Brubeck, III".to_vec()));
			assert_eq!(Balances::reserved_balance(&2), 2);
			assert_eq!(<NameOf<Test>>::get(2).unwrap(), (b"Dr. David Brubeck, III".to_vec(), 2));
		});
	}

	#[test]
	fn normal_operation_should_work() {
		new_test_ext().execute_with(|| {
			assert_ok!(Nicks::set_name(Origin::signed(1), b"Gav".to_vec()));
			assert_eq!(Balances::reserved_balance(&1), 2);
			assert_eq!(Balances::free_balance(&1), 8);
			assert_eq!(<NameOf<Test>>::get(1).unwrap().0, b"Gav".to_vec());

			assert_ok!(Nicks::set_name(Origin::signed(1), b"Gavin".to_vec()));
			assert_eq!(Balances::reserved_balance(&1), 2);
			assert_eq!(Balances::free_balance(&1), 8);
			assert_eq!(<NameOf<Test>>::get(1).unwrap().0, b"Gavin".to_vec());

			assert_ok!(Nicks::clear_name(Origin::signed(1)));
			assert_eq!(Balances::reserved_balance(&1), 0);
			assert_eq!(Balances::free_balance(&1), 10);
		});
	}

	#[test]
	fn error_catching_should_work() {
		new_test_ext().execute_with(|| {
			assert_noop!(Nicks::clear_name(Origin::signed(1)), "Not named");

			assert_noop!(Nicks::set_name(Origin::signed(3), b"Dave".to_vec()), "not enough free funds");

			assert_noop!(Nicks::set_name(Origin::signed(1), b"Ga".to_vec()), "Name too short");
			assert_noop!(
				Nicks::set_name(Origin::signed(1), b"Gavin James Wood, Esquire".to_vec()),
				"Name too long"
			);
			assert_ok!(Nicks::set_name(Origin::signed(1), b"Dave".to_vec()));
			assert_noop!(Nicks::kill_name(Origin::signed(2), 1), "bad origin");
			assert_noop!(Nicks::force_name(Origin::signed(2), 1, b"Whatever".to_vec()), "bad origin");
		});
	}
}
