// Copyright 2019-2020
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
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

//! # Generic Asset Module
//!
//! The Generic Asset module provides functionality for handling accounts and asset balances.
//!
//! ## Overview
//!
//! The Generic Asset module provides functions for:
//!
//! - Creating a new kind of asset.
//! - Setting permissions of an asset.
//! - Getting and setting free balances.
//! - Retrieving total, reserved and unreserved balances.
//! - Repatriating a reserved balance to a beneficiary account.
//! - Transferring a balance between accounts (when not reserved).
//! - Slashing an account balance.
//! - Managing total issuance.
//! - Setting and managing locks.
//!
//! ### Terminology
//!
//! - **Staking Asset:** The asset for staking, to participate as Validators in the network.
//! - **Spending Asset:** The asset for payment, such as paying transfer fees, gas fees, etc.
//! - **Permissions:** A set of rules for a kind of asset, defining the allowed operations to the asset, and which
//! accounts are allowed to possess it.
//! - **Total Issuance:** The total number of units in existence in a system.
//! - **Free Balance:** The portion of a balance that is not reserved. The free balance is the only balance that matters
//! for most operations. When this balance falls below the existential deposit, most functionality of the account is
//! removed. When both it and the reserved balance are deleted, then the account is said to be dead.
//! - **Reserved Balance:** Reserved balance still belongs to the account holder, but is suspended. Reserved balance
//! can still be slashed, but only after all the free balance has been slashed. If the reserved balance falls below the
//! existential deposit then it and any related functionality will be deleted. When both it and the free balance are
//! deleted, then the account is said to be dead.
//! - **Imbalance:** A condition when some assets were credited or debited without equal and opposite accounting
//! (i.e. a difference between total issuance and account balances). Functions that result in an imbalance will
//! return an object of the `Imbalance` trait that can be managed within your runtime logic. (If an imbalance is
//! simply dropped, it should automatically maintain any book-keeping such as total issuance.)
//! - **Lock:** A freeze on a specified amount of an account's free balance until a specified block number. Multiple
//! locks always operate over the same funds, so they "overlay" rather than "stack".
//!
//! ### Implementations
//!
//! The Generic Asset module provides `AssetCurrency`, which implements the following traits. If these traits provide
//! the functionality that you need, you can avoid coupling with the Generic Asset module.
//!
//! - `Currency`: Functions for dealing with a fungible assets system.
//! - `ReservableCurrency`: Functions for dealing with assets that can be reserved from an account.
//! - `LockableCurrency`: Functions for dealing with accounts that allow liquidity restrictions.
//! - `Imbalance`: Functions for handling imbalances between total issuance in the system and account balances.
//! Must be used when a function creates new assets (e.g. a reward) or destroys some assets (e.g. a system fee).
//!
//! The Generic Asset module provides two types of `AssetCurrency` as follows.
//!
//! - `StakingAssetCurrency`: Currency for staking.
//! - `SpendingAssetCurrency`: Currency for payments such as transfer fee, gas fee.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create`: Create a new kind of asset.
//! - `transfer`: Transfer some liquid free balance to another account.
//! - `update_permission`: Updates permission for a given `asset_id` and an account. The origin of this call
//! must have update permissions.
//! - `mint`: Mint an asset, increases its total issuance. The origin of this call must have mint permissions.
//! - `burn`: Burn an asset, decreases its total issuance. The origin of this call must have burn permissions.
//! - `create_reserved`: Create a new kind of reserved asset. The origin of this call must be root.
//!
//! ### Public Functions
//!
//! - `total_balance`: Get an account's total balance of an asset kind.
//! - `free_balance`: Get an account's free balance of an asset kind.
//! - `reserved_balance`: Get an account's reserved balance of an asset kind.
//! - `create_asset`: Creates an asset.
//! - `make_transfer`: Transfer some liquid free balance from one account to another.
//! This will not emit the `Transferred` event.
//! - `make_transfer_with_event`: Transfer some liquid free balance from one account to another.
//! This will emit the `Transferred` event.
//! - `reserve`: Moves an amount from free balance to reserved balance.
//! - `unreserve`: Move up to an amount from reserved balance to free balance. This function cannot fail.
//! - `mint_free`: Mint to an account's free balance.
//! - `burn_free`: Burn an account's free balance.
//! - `slash`: Deduct up to an amount from the combined balance of `who`, preferring to deduct from the
//!	free balance. This function cannot fail.
//! - `slash_reserved`: Deduct up to an amount from reserved balance of an account. This function cannot fail.
//! - `repatriate_reserved`: Move up to an amount from reserved balance of an account to free balance of another
//! account.
//! - `check_permission`: Check permission to perform burn, mint or update.
//! - `ensure_can_withdraw`: Check if the account is able to make a withdrawal of the given amount
//!	for the given reason.
//!
//! ### Usage
//!
//! The following examples show how to use the Generic Asset Pallet in your custom pallet.
//!
//! ### Examples from the FRAME pallet
//!
//! The Fees Pallet uses the `Currency` trait to handle fee charge/refund, and its types inherit from `Currency`:
//!
//! ```
//! use frame_support::{
//!
//! 	dispatch,
//! 	traits::{Currency, ExistenceRequirement, WithdrawReason, Contains},
//! };
//! # pub trait Trait: frame_system::Trait {
//! # 	type Currency: Currency<Self::AccountId>;
//! # }
//! type AssetOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
//!
//! fn charge_fee<T: Trait>(transactor: &T::AccountId, amount: AssetOf<T>) -> dispatch::DispatchResult {
//! 	// ...
//! 	T::Currency::withdraw(
//! 		transactor,
//! 		amount,
//! 		WithdrawReason::TransactionPayment.into(),
//! 		ExistenceRequirement::KeepAlive,
//! 	)?;
//! 	// ...
//! 	Ok(())
//! }
//!
//! fn refund_fee<T: Trait>(transactor: &T::AccountId, amount: AssetOf<T>) -> dispatch::DispatchResult {
//! 	// ...
//! 	T::Currency::deposit_into_existing(transactor, amount)?;
//! 	// ...
//! 	Ok(())
//! }
//!
//! # fn main() {}
//! ```
//!
//! ## Genesis config
//!
//! The Generic Asset Pallet depends on the [`GenesisConfig`](./struct.GenesisConfig.html).

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact, Input, Output, Error as CodecError};

use sp_runtime::{RuntimeDebug, DispatchResult, DispatchError, ModuleId, traits::{AccountIdConversion}};
use sp_runtime::traits::{
	CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Saturating, AtLeast32Bit, AtLeast32BitUnsigned,
	Zero, Bounded,
};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::prelude::*;
use sp_std::{cmp, result, fmt::Debug};
use pallet_nicks as nicks;
use nicks::AccountIdOf;
//use pallet_balances as balances;
use frame_support::{debug,
	weights::Weight,
	decl_event, decl_module, decl_storage, ensure, decl_error,
	traits::{
		Currency, ExistenceRequirement, Imbalance, LockIdentifier, LockableCurrency, ReservableCurrency,
		SignedImbalance, WithdrawReason, WithdrawReasons, TryDrop, BalanceStatus, Get, OnUnbalanced, Contains,
		EnsureOrigin, GetMembers,
	},
	Parameter, StorageMap,
};
use frame_system::{self as system, ensure_signed, ensure_root};

mod mock;
mod tests;

pub use self::imbalances::{NegativeImbalance, PositiveImbalance};
use crate::time::{DAYS, HOURS, MINUTES};
use frame_support::weights::RuntimeDbWeight;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

// 币的精度是14位
pub const DOLLARS: u128 = 10000_00000_00000;

pub mod time {
	type BlockNumber = u32;
	pub const MINUTES: BlockNumber = 60 / 4;  // todo 这里直接3是很危险的
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

#[derive(Encode, Decode, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive())]
pub enum AssetTime<BlockNumber>{
	MintVoteExists(BlockNumber),
	MintInterval(BlockNumber),
	BurnExistsHowLong(BlockNumber),
}

#[derive(Encode, Decode, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive())]
pub enum AssetAmount<Balance> {
	MintPledge(Balance),
	BurnPledge(Balance),
	MintMinAmount(Balance),
	BurnMinAmount(Balance),

}


pub trait Trait: system::Trait + nicks::Trait{

	// 用来判断是否是议会成员
	type CouncilMembers: Contains<Self::AccountId>;

	type MembersCount: GetMembers<Self::AccountId>;

	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	type Balance: Parameter
		+ Member
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug;
	type AssetId: Parameter + Member + AtLeast32Bit + Default + Copy;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	// 议会成员
	type CouncilOrigin: EnsureOrigin<Self::Origin, Success=Self::AccountId>;
	// 技术委员会
	type TechnicalOrigin: EnsureOrigin<Self::Origin, Success=Self::AccountId>;
	// transx基金会
	type TransxFoundation: EnsureOrigin<Self::Origin, Success=Self::AccountId>;

	type MaxLenOfMint: Get<u32>;

	type MaxLenOfBurn: Get<u32>;

	type TreasuryId: Get<ModuleId>;


}

pub trait Subtrait: Trait + system::Trait + nicks::Trait{

}

impl<T: Trait> Subtrait for T {

}

/// Asset creation options.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct AssetOptions<Balance: HasCompact, AccountId> {
	/// Initial issuance of this asset. All deposit to the creator of the asset.
	#[codec(compact)]
	pub initial_issuance: Balance,
	/// Which accounts are allowed to possess this asset.
	pub permissions: PermissionLatest<AccountId>,
}

/// Owner of an asset.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum Owner<AccountId> {
	/// No owner.
	None,
	/// Owned by an AccountId
	Address(AccountId),
}

impl<AccountId> Default for Owner<AccountId> {
	fn default() -> Self {
		Owner::None
	}
}

/// Asset permissions
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct PermissionsV1<AccountId> {
	/// Who have permission to update asset permission
	pub update: Owner<AccountId>,
	/// Who have permission to mint new asset
	pub mint: Owner<AccountId>,
	/// Who have permission to burn asset
	pub burn: Owner<AccountId>,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
#[repr(u8)]
enum PermissionVersionNumber {
	V1 = 0,
}

/// Versioned asset permission
#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PermissionVersions<AccountId> {
	V1(PermissionsV1<AccountId>),
}

/// Asset permission types
pub enum PermissionType {
	/// Permission to burn asset permission
	Burn,
	/// Permission to mint new asset
	Mint,
	/// Permission to update asset
	Update,
}

/// Alias to latest asset permissions
pub type PermissionLatest<AccountId> = PermissionsV1<AccountId>;

impl<AccountId> Default for PermissionVersions<AccountId> {
	fn default() -> Self {
		PermissionVersions::V1(Default::default())
	}
}

impl<AccountId: Encode> Encode for PermissionVersions<AccountId> {
	fn encode_to<T: Output>(&self, dest: &mut T) {
		match self {
			PermissionVersions::V1(payload) => {
				dest.push(&PermissionVersionNumber::V1);
				dest.push(payload);
			},
		}
	}
}

impl<AccountId: Encode> codec::EncodeLike for PermissionVersions<AccountId> {}

impl<AccountId: Decode> Decode for PermissionVersions<AccountId> {
	fn decode<I: Input>(input: &mut I) -> core::result::Result<Self, CodecError> {
		let version = PermissionVersionNumber::decode(input)?;
		Ok(
			match version {
				PermissionVersionNumber::V1 => PermissionVersions::V1(Decode::decode(input)?)
			}
		)
	}
}

impl<AccountId> Default for PermissionsV1<AccountId> {
	fn default() -> Self {
		PermissionsV1 {
			update: Owner::None,
			mint: Owner::None,
			burn: Owner::None,
		}
	}
}

impl<AccountId> Into<PermissionLatest<AccountId>> for PermissionVersions<AccountId> {
	fn into(self) -> PermissionLatest<AccountId> {
		match self {
			PermissionVersions::V1(v1) => v1,
		}
	}
}

/// Converts the latest permission to other version.
impl<AccountId> Into<PermissionVersions<AccountId>> for PermissionLatest<AccountId> {
	fn into(self) -> PermissionVersions<AccountId> {
		PermissionVersions::V1(self)
	}
}

// 铸币投票
#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode, Clone, Default)]  // 应该是有了option就必须要实现Default
pub struct MintVote<AC, AS, BA, BL>{
	pub start_block: BL,
	pub pass_block: Option<BL>,
	pub mint_block: Option<BL>,
	pub mint_man: AC,
	pub asset_id: AS,
	pub amount: BA,
	pub approve_list: Vec<AC>,
	pub reject_list: Vec<AC>,
	pub technical_reject: Option<AC>,
}

// 销毁信息
#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode, Clone, Default)]  // 应该是有了option就必须要实现Default
pub struct BurnInfo<AC, AS, BA, BL>{
	pub start_block: BL,
	pub burn_man: AC,
	pub asset_id: AS,
	pub amount: BA,
	pub foundation_tag_man: Option<AC>,  // 基金会打上标签的人
}

// 投反对或是赞成票
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum AssetVote {
	// 赞成
	Approve,
	// 反对
	Reject,
}

decl_error! {
	/// Error for the generic-asset module.
	pub enum Error for Module<T: Trait> {
		/// No new assets id available.
		NoIdAvailable,
		/// Cannot transfer zero amount.
		ZeroAmount,
		/// The origin does not have enough permission to update permissions.
		NoUpdatePermission,
		/// The origin does not have permission to mint an asset.
		NoMintPermission,
		/// The origin does not have permission to burn an asset.
		NoBurnPermission,
		/// Total issuance got overflowed after minting.
		TotalMintingOverflow,
		/// Free balance got overflowed after minting.
		FreeMintingOverflow,
		/// Total issuance got underflowed after burning.
		TotalBurningUnderflow,
		/// Free balance got underflowed after burning.
		FreeBurningUnderflow,
		/// Asset id is already taken.
		IdAlreadyTaken,
		/// Asset id not available.
		IdUnavailable,
		/// The balance is too low to send amount.
		InsufficientBalance,
		/// The account liquidity restrictions prevent withdrawal.
		LiquidityRestrictions,
		/// 没有设置铸币抵押金额
		MintPledgeNone,
		/// 没有设置金额
		AmountNone,
		/// 投票错误
		VoteError,
		/// 没有设置销毁币抵押金额
		BurnPledgeNone,
		/// 抵押不够
		BondTooLow,
		/// 铸币金额太小
		AmountTooLow,
		/// 队列过长
		QueueTooLong,
		/// 已经存在的议案
		ExistsProposal,
		/// 不存在的议案
		NotExistsProposal,
		/// 数目溢出错误
		OverFlow,
		/// 不存在该用户
		NotExistsName,
		/// 除于0错误
		DivZero,
		/// balance转换错误
		BalanceChangeErr,
		/// 未知参数
		UnknownParm,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		/// 铸币提案数量上限
		const MaxLenOfMint: u32 = T::MaxLenOfMint::get();
		/// 销毁币提案数量上限
		const MaxLenOfBurn: u32 = T::MaxLenOfBurn::get();

		type Error = Error<T>;
		fn deposit_event() = default;


		/// 铸币（测试阶段保留的方法)
	 	#[weight = 500_000]
		fn create(origin, options: AssetOptions<T::Balance, T::AccountId>) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			Self::create_asset(None, Some(origin), options)
		}


		/// 转账
		#[weight = 500_000]
		pub fn transfer(origin, #[compact] asset_id: T::AssetId, to: T::AccountId, #[compact] amount: T::Balance) {
			let origin = ensure_signed(origin)?;
			ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);
			Self::make_transfer_with_event(&asset_id, &origin, &to, amount)?;
		}


		/// 根据nicks转账
		#[weight = 500_000]
		pub fn transfer_by_name(origin, #[compact] asset_id: T::AssetId, to: Vec<u8>, #[compact] amount: T::Balance) {
			let origin = ensure_signed(origin)?;
			ensure!(<AccountIdOf<T>>::contains_key(to.clone()), Error::<T>::NotExistsName);
			let to = <AccountIdOf<T>>::get(to.clone());
			ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);
			Self::make_transfer_with_event(&asset_id, &origin, &to, amount)?;
		}


		/// 矿工申请铸币
		#[weight = 500_000]
		fn need_mint(origin, asset_id: T::AssetId, amount: T::Balance) -> DispatchResult{
			let who = ensure_signed(origin)?;

			// 铸币金额太小 不给铸币
			let amount_1 = Self::balance_change_into_balanceof(amount)?;
			ensure!(<MintMinAmount<T>>::get() <= amount_1, Error::<T>::AmountTooLow);

			// 如果正在等待的铸币队列大于100， 则不给申请铸币
			let tempt_queue = Self::vote_queue();
			ensure!((tempt_queue.len() as u32) < T::MaxLenOfMint::get(), Error::<T>::QueueTooLong);

			// 同一种币同一时间只能提一个议案
			ensure!(!(<MintVoteInfo<T>>::contains_key((who.clone(), asset_id.clone()))), Error::<T>::ExistsProposal);

			// 抵押金额（不够不给操作）
			T::Currency::reserve(&who, <MintPledge<T>>::get())
			.map_err(|_| Error::<T>::BondTooLow)?;

			let start_block = <system::Module<T>>::block_number();

			// 如果是议会成员 自己默认一个赞成票
			let mut approve_list = vec![];
			if Self::is_council_member(who.clone()){
				approve_list = vec![who.clone()]
			}

			let vote = MintVote{
				start_block: start_block,
				pass_block: None,
				mint_block: None,
				mint_man: who.clone(),
				asset_id: asset_id.clone(),
				amount: amount.clone(),
				approve_list: approve_list,
				reject_list: vec![],
				technical_reject: None,
			};
			<MintVoteInfo<T>>::insert((who.clone(), asset_id.clone()), vote);
			<VoteQueue<T>>::mutate(|queue| queue.push((who.clone(), asset_id.clone())));
			Ok(())
		}


		/// 议会为铸币议案投票
		#[weight = 500_000]
		fn council_vote_for_mint(origin, who: T::AccountId, asset_id: T::AssetId, appro_or_reject: AssetVote) -> DispatchResult{
			// 议会成员才有投票的资格
			let origin = T::CouncilOrigin::ensure_origin(origin)?;

			let mut vote = Self::mint_vote_info((who.clone(), asset_id.clone())).ok_or(Error::<T>::NotExistsProposal)?;

			let position_approve = vote.approve_list.iter().position(|a| a == &origin);
			let position_reject = vote.reject_list.iter().position(|a| a == &origin);

			if appro_or_reject == AssetVote::Approve{
				if position_approve.is_none(){
					vote.approve_list.push(origin.clone());
				}
				else{
					return Err(Error::<T>::VoteError)?;
				}
				if let Some(pos) = position_reject{
					vote.reject_list.swap_remove(pos);
				}
			}
			else{
				if position_reject.is_none(){
					vote.reject_list.push(origin.clone());
				}
				else{
					return Err(Error::<T>::VoteError)?;
				}
				if let Some(pos) = position_approve{
					vote.approve_list.swap_remove(pos);
				}
			}

			let pass_block = <system::Module<T>>::block_number();

			let reject_votes = vote.reject_list.len() as u32;
			let app_votes =  vote.approve_list.len() as u32;

			// 过的票数大于等于10/13
			if app_votes * 13u32 >= Self::get_members_count() * 10u32{
				vote.pass_block = Some(pass_block);
			}
			else{
				vote.pass_block = None;
			}

			// 反对的票数大于3/13
			if reject_votes * 13u32 > 3u32 * Self::get_members_count(){

				Self::mint_last_do(vote, false)?;
			}
			else{
				<MintVoteInfo<T>>::insert((who.clone(), asset_id.clone()), vote);
			}
			Self::deposit_event(RawEvent::CouncilVoted);
			Ok(())
		}


		/// 技术委员会一票否决铸币
		#[weight = 500_000]
		fn technical_reject_mint(origin, who: T::AccountId, asset_id: T::AssetId) -> DispatchResult{
			// 技术委员会成员才有一票否决的资格
			let origin = T::TechnicalOrigin::ensure_origin(origin)?;

			// 这个议案要存在
			let mut vote = Self::mint_vote_info((who.clone(), asset_id.clone())).ok_or(Error::<T>::NotExistsProposal)?;
			vote.technical_reject = Some(origin.clone());

			Self::mint_last_do(vote.clone(), false)?;
			Self::deposit_event(RawEvent::TechnicalReject);
			Ok(())

		}


		/// 销毁币
		#[weight = 500_000]
		fn burn(origin, #[compact] asset_id: T::AssetId, amount: T::Balance) -> DispatchResult {
			let to = ensure_signed(origin)?;
			// todo 这里代码是错误的 没有核查这个币是否存在

			let original_free_balance = Self::free_balance(&asset_id, &to);

			let current_total_issuance = <TotalIssuance<T>>::get(asset_id);
			let new_total_issuance = current_total_issuance.checked_sub(&amount)
				.ok_or_else(|| Error::<T>::OverFlow)?;
			let value = original_free_balance.checked_sub(&amount)
				.ok_or_else(|| Error::<T>::OverFlow)?;

			// 同一时间一个人不能销毁相同assetid的资产
			ensure!(!(<BurnStatics<T>>::contains_key((to.clone(), asset_id.clone()))), Error::<T>::ExistsProposal);

			// 同一时间 不能超过100个销毁申请
			let queue = <BurnQueue<T>>::get();
			let queue_len = queue.len() as u32;
			ensure!(queue_len < T::MaxLenOfBurn::get(), Error::<T>::QueueTooLong);

			// 销毁币金额太小 不给销毁币
			let amount_1 = Self::balance_change_into_balanceof(amount)?;
			ensure!(<BurnMinAmount<T>>::get() <= amount_1, Error::<T>::AmountTooLow);

			// 抵押金额不够不给操作
			T::Currency::reserve(&to, <BurnPledge<T>>::get())
			.map_err(|_| Error::<T>::BondTooLow)?;

			let start_block = <system::Module<T>>::block_number();
			let burn_info = BurnInfo{
				start_block: start_block.clone(),
				burn_man: to.clone(),
				asset_id: asset_id,
				amount: amount,
				foundation_tag_man: None,
			};

			<BurnStatics<T>>::insert((to.clone(), asset_id.clone()), burn_info);
			<BurnQueue<T>>::mutate(|a| a.push((to.clone(), asset_id.clone())));

			<TotalIssuance<T>>::insert(asset_id, new_total_issuance);

			Self::set_free_balance(&asset_id, &to, value);

			Self::deposit_event(RawEvent::Burned(asset_id, to, amount));

			Ok(())

		}


		/// 基金会确认销毁币
		#[weight = 500_000]
		fn foundation_tag_for_burn(origin, who: T::AccountId, asset_id: T::AssetId) -> DispatchResult{
			// 只有基金会成员才有权限
			let origin = T::TransxFoundation::ensure_origin(origin)?;
			let mut burn_info = Self::burn_statisc((who.clone(), asset_id.clone())).ok_or(Error::<T>::NotExistsProposal)?;
			burn_info.foundation_tag_man = Some(origin.clone());

			// 惩罚掉抵押
			let real_bond = <BurnPledge<T>>::get();

			// slash掉申请铸币者的抵押币
			T::Currency::slash_reserved(&who, real_bond.clone());

			// 奖励抵押金额给基金会贴上标签的人
			T::Currency::deposit_creating(&origin, real_bond);

			Self::burn_last_do(burn_info, true)?;
			Self::deposit_event(RawEvent::FoundationTaged);
			Ok(())
		}


		/// 设置时间相关参数
		#[weight = 500_000]
		fn set_time(origin, time: AssetTime<T::BlockNumber>) {
			ensure_root(origin)?;
			match time {
				AssetTime::MintVoteExists(x) => <MintVoteExists<T>>::put(x),
				AssetTime::MintInterval(x) => <MintInterval<T>>::put(x),
				AssetTime::BurnExistsHowLong(x) => <BurnExistsHowLong<T>>::put(x),
				_ => return Err(Error::<T>::UnknownParm)?,
			}
			Self::deposit_event(RawEvent::SetTime);
		}


		/// 设置金额相关参数
		#[weight = 500_000]
		fn set_amount(origin, amount: AssetAmount<BalanceOf<T>>) {
			ensure_root(origin)?;
			match amount {
				AssetAmount::MintPledge(x) => <MintPledge<T>>::put(x),
				AssetAmount::BurnPledge(x) => <BurnPledge<T>>::put(x),
				AssetAmount::MintMinAmount(x) => <MintMinAmount<T>>::put(x),
				AssetAmount::BurnMinAmount(x) => <BurnMinAmount<T>>::put(x),
				_ => return Err(Error::<T>::UnknownParm)?,
			}
			Self::deposit_event(RawEvent::SetAmount);
		}

		fn on_finalize(n: T::BlockNumber) {

			// 寻找过期的铸币议案，没有通过的就要废弃
			Self::find_expire_mint_vote(n);

			// 寻找通过的投票并处理
			if (n % <MintInterval<T>>::get()).is_zero() {
				Self::find_pass_vote_and_mint(n);
			}

			// 寻找过期的销毁币的议案并将之从队列中删除
			// todo 这里剩下的都是还没有过的
			Self::find_expire_burn_info(n);
		}

	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceLock<Balance> {
	pub id: LockIdentifier,
	pub amount: Balance,
	pub reasons: WithdrawReasons,
}



decl_storage! {
	trait Store for Module<T: Trait> as GenericAsset {
		/// Total issuance of a given asset.
		pub TotalIssuance get(fn total_issuance) build(|config: &GenesisConfig<T>| {
			let issuance = config.initial_balance * (config.endowed_accounts.len() as u32).into();
			config.assets.iter().map(|id| (id.clone(), issuance)).collect::<Vec<_>>()
		}): map hasher(twox_64_concat) T::AssetId => T::Balance;

		/// The free balance of a given asset under an account.
		pub FreeBalance:
			double_map hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => T::Balance;

		/// The reserved balance of a given asset under an account.
		pub ReservedBalance:
			double_map hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => T::Balance;

		/// Next available ID for user-created asset.
		pub NextAssetId get(fn next_asset_id) config(): T::AssetId;

		/// Permission options for a given asset.
		pub Permissions get(fn get_permission):
			map hasher(twox_64_concat) T::AssetId => PermissionVersions<T::AccountId>;

		/// Any liquidity locks on some account balances.
		pub Locks get(fn locks):
			map hasher(blake2_128_concat) T::AccountId => Vec<BalanceLock<T::Balance>>;

		/// The identity of the asset which is the one that is designated for the chain's staking system.
		pub StakingAssetId get(fn staking_asset_id) config(): T::AssetId;

		/// The identity of the asset which is the one that is designated for paying the chain's transaction fee.
		pub SpendingAssetId get(fn spending_asset_id) config(): T::AssetId;

		/// 正在投票的队列（方便知道目前有多少铸币议案等待通过）
		pub VoteQueue get(fn vote_queue): Vec<(T::AccountId, T::AssetId)>;

		/// 当前铸币议案的具体信息
		pub MintVoteInfo get(fn mint_vote_info): map hasher(blake2_128_concat) (T::AccountId, T::AssetId) => Option<MintVote<T::AccountId,
		T::AssetId, T::Balance, T::BlockNumber>>;

		/// 一个二级map，用于存储所有铸币议案的最终信息（永久存储）
		pub MintVoteInfo2 get(fn mint_vote_info_2): double_map hasher(twox_64_concat) T::AccountId,  hasher(blake2_128_concat) T::AssetId => Vec<MintVote<
		T::AccountId,T::AssetId, T::Balance, T::BlockNumber>>;

		/// 当前等待银行打标签确认的销毁资金的议案
		pub BurnQueue get(fn mint_queue): Vec<(T::AccountId, T::AssetId)>;

		/// 当前正在等待打标签的议案， 打完标签即删除（为了方便银行和基金会查看）
		pub BurnStatics get(fn burn_statisc): map hasher(blake2_128_concat) (T::AccountId, T::AssetId) => Option<BurnInfo<T::AccountId,
		T::AssetId, T::Balance, T::BlockNumber>>;

		/// 永久存储销毁议案的信息
		pub BurnStatics2 get(fn burn_statisc_2): double_map hasher(twox_64_concat) T::AccountId, hasher(blake2_128_concat) T::AssetId => Vec<BurnInfo<
		T::AccountId,T::AssetId, T::Balance, T::BlockNumber>>;

		/// 铸币需要抵押的金额
		MintPledge get(fn mint_pledge): BalanceOf<T> = <BalanceOf<T> as TryFrom::<u128>>::try_from(10 * DOLLARS).ok().unwrap();

		/// 销毁币需要抵押的金额
		BurnPledge get(fn burn_pledge): BalanceOf<T> = <BalanceOf<T> as TryFrom::<u128>>::try_from(10 * DOLLARS).ok().unwrap();

		/// 铸币最小金额
		MintMinAmount get(fn mint_min_amount): BalanceOf<T> = <BalanceOf<T> as TryFrom::<u128>>::try_from(100_0000 * DOLLARS).ok().unwrap();

		/// 销毁币最小金额
		BurnMinAmount get(fn burn_min_amount): BalanceOf<T> = <BalanceOf<T> as TryFrom::<u128>>::try_from(100_0000 * DOLLARS).ok().unwrap();

		/// 铸币提案存在的最长时间
		MintVoteExists get(fn mint_exists_how_long): T::BlockNumber = T::BlockNumber::from(7 * DAYS);

		/// 多久集体铸币一次
		MintInterval get(fn mint_period): T::BlockNumber = T::BlockNumber::from(1 * DAYS);

		/// 销毁币提案存在的最长时间
		BurnExistsHowLong get(fn burn_exists_how_long): T::BlockNumber = T::BlockNumber::from(7 * DAYS);

		// ******************************
		// todo 测试专用
		pub MintTest get(fn mint_test): (T::BlockNumber, T::BlockNumber, T::BlockNumber);

		pub ManCount get(fn man_count): u32;

		pub BurnQueueLen get(fn queue_len)	: u32;

		pub BurnTest get(fn burn_test): (T::BlockNumber, T::BlockNumber, T::BlockNumber);
		}
	add_extra_genesis {
		config(assets): Vec<T::AssetId>;
		config(initial_balance): T::Balance;
		config(endowed_accounts): Vec<T::AccountId>;

		build(|config: &GenesisConfig<T>| {

			config.assets.iter().for_each(|asset_id| {
				config.endowed_accounts.iter().for_each(|account_id| {
					<FreeBalance<T>>::insert(asset_id, account_id, &config.initial_balance);
				});
			});
		});
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AssetId,
		AssetOptions = AssetOptions<<T as Trait>::Balance, <T as frame_system::Trait>::AccountId>
	{
		/// Asset created (asset_id, creator, asset_options).
		Created(AssetId, AccountId, AssetOptions),
		/// Asset transfer succeeded (asset_id, from, to, amount).
		Transferred(AssetId, AccountId, AccountId, Balance),
		/// Asset permission updated (asset_id, new_permissions).
		PermissionUpdated(AssetId, PermissionLatest<AccountId>),
		/// New asset minted (asset_id, account, amount).
		Minted(AssetId, AccountId, Balance),
		/// Asset burned (asset_id, account, amount).
		Burned(AssetId, AccountId, Balance),

		TechnicalReject,

		CouncilVoted,

		FoundationTaged,

		SetParamsed,

		SetTime,

		SetAmount,
	}
);

impl<T: Trait> Module<T> {
	// PUBLIC IMMUTABLES

	// 寻找过期的mint投票并删除
	// fixme 过期但是没有通过
	fn find_expire_mint_vote(n: T::BlockNumber){
		let vote_queue_1 = <VoteQueue<T>>::get();
		if vote_queue_1.len() != 0{
			<VoteQueue<T>>::mutate(|votes| votes.retain(
			|vote1| {
				// 已经过期 并且没有通过
				if let Some(vote) = <MintVoteInfo<T>>::get(vote1){
					 <MintTest<T>>::put((n, vote.start_block.clone(), <MintVoteExists<T>>::get()));
				if n - vote.start_block.clone() >= <MintVoteExists<T>>::get(){
					// 赞成票数达不到10/13
					if (vote.approve_list.len() as u32) * 13u32 < 10u32 * Self::get_members_count() {
						if Self::mint_last_do(vote.clone(), false).is_ok(){
							false
						}
						else{
							// 一般也不会执行
							true
						}
					}
					else{
						true
					}
					}
					else{
						true
					}
			}
			else{
			// 这个不会执行
			true
			}}
			))
		}
	}


	// 统一时间处理铸币议案（这个时候通过才会马上铸币）
	fn find_pass_vote_and_mint(n: T::BlockNumber){
		// 投票时间已经过 并且赞成 那么给铸币
		// 虽然投票已经超过10票 但是没有到过期时间 不给铸币
		let vote_queue = <VoteQueue<T>>::get();
		if vote_queue.len() != 0{
			<VoteQueue<T>>::mutate(|votes| votes.retain(
			|vote1| {
				if let  Some(vote) = <MintVoteInfo<T>>::get(vote1){
				// 如果已经过期(这里剩下的过期的其实都是可以铸币的）
				if n - vote.start_block.clone() >= <MintVoteExists<T>>::get(){
					// 这里不能终结
					if Self::mint(vote.asset_id.clone(), vote.mint_man.clone(), vote.amount).is_ok() && Self::mint_last_do(vote.clone(), true).is_ok(){
						false
					}
					else{
						true
					}
				}
				else{true}
			} else{
				//  这个不会执行
				true
			}
			}
			))
		}
			}

	// 获取国库id
	pub fn treasury_id() -> T::AccountId {
		T::TreasuryId::get().into_account()
	}

	// 获取议会成员数目
	fn get_members_count() -> u32{
		T::MembersCount::get_members_len()
	}

	// 寻找过期的销毁币的议案并将之从队列中删除
	fn find_expire_burn_info(n: T::BlockNumber){
		let burn_queue = <BurnQueue<T>>::get();
		<BurnQueueLen>::put(burn_queue.len() as u32);
		if burn_queue.len() != 0{
		<BurnQueue<T>>::mutate(|burns| burns.retain(
		|burn1|{
			if let Some(burn) = <BurnStatics<T>>::get(burn1){
				<BurnTest<T>>::put((n, burn.start_block.clone(), <BurnExistsHowLong<T>>::get()));
				if n - burn.start_block.clone() >= <BurnExistsHowLong<T>>::get(){
					// 过期删除掉 银行没贴标签，失败需要重新给该用户铸币（因为用户已经先销毁）
					if Self::burn_last_do(burn.clone(), false).is_ok() && Self::mint(burn.asset_id.clone(), burn.burn_man.clone(), burn.amount.clone()).is_ok(){
						false
					}
					else{
						true
					}
				}
				else{
					true
				}
			}
			else{
				false
			}
		}
		))
		}
	}

	// 铸币最终一步（不管成功或是失败均会执行)
	pub fn mint_last_do(mut vote: MintVote<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>, is_mint: bool) -> result::Result<(),&'static str>{
		// 把u128转换成BalanceOf
		let real_bond = <MintPledge<T>>::get();
		// slash掉申请铸币者的抵押币
		T::Currency::slash_reserved(&vote.mint_man, real_bond.clone());

		// 获取参与投票的议会成员
		let all_mans = vote.reject_list.iter().chain(vote.approve_list.iter());
		let mut all_mans_cp = all_mans.clone();
		let mans_count =  all_mans.count() as u32;
		<ManCount>::put(mans_count);
		let mans_count_1 = <BalanceOf<T> as TryFrom<u32>>::try_from(mans_count.clone());

		let man_real_count = if let Some(t) = mans_count_1.ok(){
			t
		}
		else{
			return Err("council number err");
		};

		if man_real_count.clone() != <BalanceOf<T>>::from(0){
			// 获取每一个议会成员的奖励
			let per_man_reward = real_bond.clone() / man_real_count.clone();
			// 奖励每一个议会成员
			for i in 0..mans_count{
				if let Some(man) = all_mans_cp.next(){
					T::Currency::deposit_creating(&man, per_man_reward);
				}
			}

		}
			// 没有人参与投票则给国库(为了金额总数平衡）
		else{
			T::Currency::deposit_creating(&Self::treasury_id(), real_bond);
		}

		// 删除vec里面的信息
		let who = vote.mint_man.clone();
		let asset_id = vote.asset_id.clone();
		<VoteQueue<T>>::mutate(|votes| votes.retain(|h| h != &(who.clone(), asset_id.clone())));

		// 删除一级map里面的信息
		<MintVoteInfo<T>>::remove((who.clone(), asset_id.clone()));

		// 添加二级map的信息
		if is_mint{
			let mint_block = <system::Module<T>>::block_number();
			vote.mint_block = Some(mint_block);
		}
		if <MintVoteInfo2<T>>::contains_key(who.clone(), asset_id.clone()){
			let mut vote_vec = <MintVoteInfo2<T>>::get(who.clone(), asset_id);
			vote_vec.push(vote.clone());
			<MintVoteInfo2<T>>::insert(who.clone(), asset_id.clone(), vote_vec);
		}
		else{
			let vote_vec = vec![vote.clone()];
			<MintVoteInfo2<T>>::insert(who.clone(), asset_id.clone(), vote_vec);
		}
		Ok(())
	}

	// 把balance转换成balanceof
	fn balance_change_into_balanceof(balance: T::Balance) -> Result<BalanceOf<T>, DispatchError> {
		let balance_1 =  <<T as Trait>::Balance as TryInto::<u128>>::try_into(balance).map_err(|_| Error::<T>::BalanceChangeErr)?;
		let balance_2 = <BalanceOf<T> as TryFrom::<u128>>::try_from(balance_1).map_err(|_| Error::<T>::BalanceChangeErr)?;
		Ok(balance_2)
	}


	fn burn_last_do(burn_info: BurnInfo<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>, is_burn: bool) -> DispatchResult{

		<BurnQueue<T>>::mutate(|queue| queue.retain(|h| h != &(burn_info.burn_man.clone(), burn_info.asset_id.clone())));
		<BurnStatics<T>>::remove((burn_info.burn_man.clone(), burn_info.asset_id.clone()));

		if <BurnStatics2<T>>::contains_key(burn_info.burn_man.clone(), burn_info.asset_id.clone()){
			let mut vote_vec = <BurnStatics2<T>>::get(burn_info.burn_man.clone(), burn_info.asset_id.clone());
			vote_vec.push(burn_info.clone());
			<BurnStatics2<T>>::insert(burn_info.burn_man.clone(), burn_info.asset_id.clone(), vote_vec);
		}
		else{
			let vote_vec = vec![burn_info.clone()];
			<BurnStatics2<T>>::insert(burn_info.burn_man.clone(), burn_info.asset_id.clone(), vote_vec);
		}

		//如果没有销毁成功 则惩罚掉销毁币抵押金额 并把它转移到国库
		if !is_burn{
			let real_bond = <BurnPledge<T>>::get();

			T::Currency::slash_reserved(&burn_info.burn_man, real_bond.clone());
			// 转移到国库
			T::Currency::deposit_creating(&Self::treasury_id(), real_bond);

		}

		Ok(())
	}


	fn is_council_member(who: T::AccountId) -> bool{
		T::CouncilMembers::contains(&who)
	}

	fn mint(asset_id: T::AssetId, to: T::AccountId, amount: T::Balance) -> DispatchResult {
		let original_free_balance = Self::free_balance(&asset_id, &to);
		let current_total_issuance = <TotalIssuance<T>>::get(asset_id);
		let new_total_issuance = current_total_issuance.checked_add(&amount)
			.ok_or_else(|| Error::<T>::OverFlow)?;
		let value = original_free_balance.checked_add(&amount)
			.ok_or_else(|| Error::<T>::OverFlow)?;

		<TotalIssuance<T>>::insert(asset_id.clone(), new_total_issuance);
		Self::set_free_balance(&asset_id, &to, value);

		Self::deposit_event(RawEvent::Minted(asset_id.clone(), to, amount));

		Ok(())
		}


	/// Get an account's total balance of an asset kind.
	pub fn total_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
		Self::free_balance(asset_id, who) + Self::reserved_balance(asset_id, who)
	}

	/// Get an account's free balance of an asset kind.
	pub fn free_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
		<FreeBalance<T>>::get(asset_id, who)
	}

	/// Get an account's reserved balance of an asset kind.
	pub fn reserved_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
		<ReservedBalance<T>>::get(asset_id, who)
	}

	/// Mint to an account's free balance, without event
	pub fn mint_free(
		asset_id: &T::AssetId,
		who: &T::AccountId,
		to: &T::AccountId,
		amount: &T::Balance,
	) -> DispatchResult {
		if Self::check_permission(asset_id, who, &PermissionType::Mint) {
			let original_free_balance = Self::free_balance(&asset_id, &to);
			let current_total_issuance = <TotalIssuance<T>>::get(asset_id);
			let new_total_issuance = current_total_issuance.checked_add(&amount)
				.ok_or(Error::<T>::TotalMintingOverflow)?;
			let value = original_free_balance.checked_add(&amount)
				.ok_or(Error::<T>::FreeMintingOverflow)?;

			<TotalIssuance<T>>::insert(asset_id, new_total_issuance);
			Self::set_free_balance(&asset_id, &to, value);
			Ok(())
		} else {
			Err(Error::<T>::NoMintPermission)?
		}
	}

	/// Burn an account's free balance, without event
	pub fn burn_free(
		asset_id: &T::AssetId,
		who: &T::AccountId,
		to: &T::AccountId,
		amount: &T::Balance,
	) -> DispatchResult {
		if Self::check_permission(asset_id, who, &PermissionType::Burn) {
			let original_free_balance = Self::free_balance(asset_id, to);

			let current_total_issuance = <TotalIssuance<T>>::get(asset_id);
			let new_total_issuance = current_total_issuance.checked_sub(amount)
				.ok_or(Error::<T>::TotalBurningUnderflow)?;
			let value = original_free_balance.checked_sub(amount)
				.ok_or(Error::<T>::FreeBurningUnderflow)?;

			<TotalIssuance<T>>::insert(asset_id, new_total_issuance);
			Self::set_free_balance(asset_id, to, value);
			Ok(())
		} else {
			Err(Error::<T>::NoBurnPermission)?
		}
	}

	/// Creates an asset.
	///
	/// # Arguments
	/// * `asset_id`: An ID of a reserved asset.
	/// If not provided, a user-generated asset will be created with the next available ID.
	/// * `from_account`: The initiator account of this call
	/// * `asset_options`: Asset creation options.
	///
	pub fn create_asset(
		asset_id: Option<T::AssetId>,
		from_account: Option<T::AccountId>,
		options: AssetOptions<T::Balance, T::AccountId>,
	) -> DispatchResult {
		let asset_id = if let Some(asset_id) = asset_id {
			ensure!(!<TotalIssuance<T>>::contains_key(&asset_id), Error::<T>::IdAlreadyTaken);
			ensure!(asset_id < Self::next_asset_id(), Error::<T>::IdUnavailable);
			asset_id
		} else {
			let asset_id = Self::next_asset_id();
			let next_id = asset_id
				.checked_add(&One::one())
				.ok_or(Error::<T>::NoIdAvailable)?;
			<NextAssetId<T>>::put(next_id);
			asset_id
		};

		let account_id = from_account.unwrap_or_default();
		let permissions: PermissionVersions<T::AccountId> = options.permissions.clone().into();

		<TotalIssuance<T>>::insert(asset_id, &options.initial_issuance);
		<FreeBalance<T>>::insert(&asset_id, &account_id, &options.initial_issuance);
		<Permissions<T>>::insert(&asset_id, permissions);

		Self::deposit_event(RawEvent::Created(asset_id, account_id, options));

		Ok(())
	}

	/// Transfer some liquid free balance from one account to another.
	/// This will not emit the `Transferred` event.
	pub fn make_transfer(
		asset_id: &T::AssetId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: T::Balance
	) -> DispatchResult {
		let new_balance = Self::free_balance(asset_id, from)
			.checked_sub(&amount)
			.ok_or(Error::<T>::InsufficientBalance)?;
		Self::ensure_can_withdraw(asset_id, from, amount, WithdrawReason::Transfer.into(), new_balance)?;

		if from != to {
			<FreeBalance<T>>::mutate(asset_id, from, |balance| *balance -= amount);
			<FreeBalance<T>>::mutate(asset_id, to, |balance| *balance += amount);
		}

		Ok(())
	}

	/// Transfer some liquid free balance from one account to another.
	/// This will emit the `Transferred` event.
	pub fn make_transfer_with_event(
		asset_id: &T::AssetId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		Self::make_transfer(asset_id, from, to, amount)?;

		if from != to {
			Self::deposit_event(RawEvent::Transferred(*asset_id, from.clone(), to.clone(), amount));
		}

		Ok(())
	}

	/// Move `amount` from free balance to reserved balance.
	///
	/// If the free balance is lower than `amount`, then no funds will be moved and an `Err` will
	/// be returned. This is different behavior than `unreserve`.
	pub fn reserve(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance)
		-> DispatchResult
	{
		// Do we need to consider that this is an atomic transaction?
		let original_reserve_balance = Self::reserved_balance(asset_id, who);
		let original_free_balance = Self::free_balance(asset_id, who);
		if original_free_balance < amount {
			Err(Error::<T>::InsufficientBalance)?
		}
		let new_reserve_balance = original_reserve_balance + amount;
		Self::set_reserved_balance(asset_id, who, new_reserve_balance);
		let new_free_balance = original_free_balance - amount;
		Self::set_free_balance(asset_id, who, new_free_balance);
		Ok(())
	}

	/// Moves up to `amount` from reserved balance to free balance. This function cannot fail.
	///
	/// As many assets up to `amount` will be moved as possible. If the reserve balance of `who`
	/// is less than `amount`, then the remaining amount will be returned.
	/// NOTE: This is different behavior than `reserve`.
	pub fn unreserve(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> T::Balance {
		let b = Self::reserved_balance(asset_id, who);
		let actual = sp_std::cmp::min(b, amount);
		let original_free_balance = Self::free_balance(asset_id, who);
		let new_free_balance = original_free_balance + actual;
		Self::set_free_balance(asset_id, who, new_free_balance);
		Self::set_reserved_balance(asset_id, who, b - actual);
		amount - actual
	}

	/// Deduct up to `amount` from the combined balance of `who`, preferring to deduct from the
	/// free balance. This function cannot fail.
	///
	/// As much funds up to `amount` will be deducted as possible. If this is less than `amount`
	/// then `Some(remaining)` will be returned. Full completion is given by `None`.
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	pub fn slash(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> Option<T::Balance> {
		let free_balance = Self::free_balance(asset_id, who);
		let free_slash = sp_std::cmp::min(free_balance, amount);
		let new_free_balance = free_balance - free_slash;
		Self::set_free_balance(asset_id, who, new_free_balance);
		if free_slash < amount {
			Self::slash_reserved(asset_id, who, amount - free_slash)
		} else {
			None
		}
	}

	/// Deducts up to `amount` from reserved balance of `who`. This function cannot fail.
	///
	/// As much funds up to `amount` will be deducted as possible. If the reserve balance of `who`
	/// is less than `amount`, then a non-zero second item will be returned.
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	pub fn slash_reserved(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> Option<T::Balance> {
		let original_reserve_balance = Self::reserved_balance(asset_id, who);
		let slash = sp_std::cmp::min(original_reserve_balance, amount);
		let new_reserve_balance = original_reserve_balance - slash;
		Self::set_reserved_balance(asset_id, who, new_reserve_balance);
		if amount == slash {
			None
		} else {
			Some(amount - slash)
		}
	}

	/// Move up to `amount` from reserved balance of account `who` to balance of account
	/// `beneficiary`, either free or reserved depending on `status`.
	///
	/// As much funds up to `amount` will be moved as possible. If this is less than `amount`, then
	/// the `remaining` would be returned, else `Zero::zero()`.
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	pub fn repatriate_reserved(
		asset_id: &T::AssetId,
		who: &T::AccountId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
		status: BalanceStatus,
	) -> T::Balance {
		let b = Self::reserved_balance(asset_id, who);
		let slash = sp_std::cmp::min(b, amount);

		match status {
			BalanceStatus::Free => {
				let original_free_balance = Self::free_balance(asset_id, beneficiary);
				let new_free_balance = original_free_balance + slash;
				Self::set_free_balance(asset_id, beneficiary, new_free_balance);
			}
			BalanceStatus::Reserved => {
				let original_reserved_balance = Self::reserved_balance(asset_id, beneficiary);
				let new_reserved_balance = original_reserved_balance + slash;
				Self::set_reserved_balance(asset_id, beneficiary, new_reserved_balance);
			}
		}

		let new_reserve_balance = b - slash;
		Self::set_reserved_balance(asset_id, who, new_reserve_balance);
		amount - slash
	}

	/// Check permission to perform burn, mint or update.
	///
	/// # Arguments
	/// * `asset_id`:  A `T::AssetId` type that contains the `asset_id`, which has the permission embedded.
	/// * `who`: A `T::AccountId` type that contains the `account_id` for which to check permissions.
	/// * `what`: The permission to check.
	///
	pub fn check_permission(asset_id: &T::AssetId, who: &T::AccountId, what: &PermissionType) -> bool {
		let permission_versions: PermissionVersions<T::AccountId> = Self::get_permission(asset_id);
		let permission = permission_versions.into();

		match (what, permission) {
			(
				PermissionType::Burn,
				PermissionLatest {
					burn: Owner::Address(account),
					..
				},
			) => account == *who,
			(
				PermissionType::Mint,
				PermissionLatest {
					mint: Owner::Address(account),
					..
				},
			) => account == *who,
			(
				PermissionType::Update,
				PermissionLatest {
					update: Owner::Address(account),
					..
				},
			) => account == *who,
			_ => false,
		}
	}

	/// Return `Ok` iff the account is able to make a withdrawal of the given amount
	/// for the given reason.
	///
	/// `Err(...)` with the reason why not otherwise.
	pub fn ensure_can_withdraw(
		asset_id: &T::AssetId,
		who: &T::AccountId,
		_amount: T::Balance,
		reasons: WithdrawReasons,
		new_balance: T::Balance,
	) -> DispatchResult {
		if asset_id != &Self::staking_asset_id() {
			return Ok(());
		}

		let locks = Self::locks(who);
		if locks.is_empty() {
			return Ok(());
		}
		if Self::locks(who)
			.into_iter().all(|l| new_balance >= l.amount || !l.reasons.intersects(reasons))
		{
			Ok(())
		} else {
			Err(Error::<T>::LiquidityRestrictions)?
		}
	}

	// PRIVATE MUTABLES

	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_reserved_balance(asset_id: &T::AssetId, who: &T::AccountId, balance: T::Balance) {
		<ReservedBalance<T>>::insert(asset_id, who, &balance);
	}

	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_free_balance(asset_id: &T::AssetId, who: &T::AccountId, balance: T::Balance) {
		<FreeBalance<T>>::insert(asset_id, who, &balance);
	}

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		reasons: WithdrawReasons,
	) {
		let mut new_lock = Some(BalanceLock {
			id,
			amount,
			reasons,
		});
		let mut locks = <Module<T>>::locks(who)
			.into_iter()
			.filter_map(|l| {
				if l.id == id {
					new_lock.take()
				} else {
					Some(l)
				}
			})
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		<Locks<T>>::insert(who, locks);
	}

	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		reasons: WithdrawReasons,
	) {
		let mut new_lock = Some(BalanceLock {
			id,
			amount,
			reasons,
		});
		let mut locks = <Module<T>>::locks(who)
			.into_iter()
			.filter_map(|l| {
				if l.id == id {
					new_lock.take().map(|nl| BalanceLock {
						id: l.id,
						amount: l.amount.max(nl.amount),
						reasons: l.reasons | nl.reasons,
					})
				} else {
					Some(l)
				}
			})
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		<Locks<T>>::insert(who, locks);
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		let mut locks = <Module<T>>::locks(who);
		locks.retain(|l| l.id != id);
		<Locks<T>>::insert(who, locks);
	}
}

pub trait AssetIdProvider {
	type AssetId;
	fn asset_id() -> Self::AssetId;
}

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
	use super::{
		result, AssetIdProvider, Imbalance, Saturating, StorageMap, Subtrait, Zero, TryDrop
	};
	use sp_std::mem;

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been created without any equal and opposite accounting.
	#[must_use]
	pub struct PositiveImbalance<T: Subtrait, U: AssetIdProvider<AssetId = T::AssetId>>(
		T::Balance,
		sp_std::marker::PhantomData<U>,
	);
	impl<T, U> PositiveImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		pub fn new(amount: T::Balance) -> Self {
			PositiveImbalance(amount, Default::default())
		}
	}

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been destroyed without any equal and opposite accounting.
	#[must_use]
	pub struct NegativeImbalance<T: Subtrait, U: AssetIdProvider<AssetId = T::AssetId>>(
		T::Balance,
		sp_std::marker::PhantomData<U>,
	);
	impl<T, U> NegativeImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		pub fn new(amount: T::Balance) -> Self {
			NegativeImbalance(amount, Default::default())
		}
	}

	impl<T, U> TryDrop for PositiveImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		fn try_drop(self) -> result::Result<(), Self> {
			self.drop_zero()
		}
	}

	impl<T, U> Imbalance<T::Balance> for PositiveImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		type Opposite = NegativeImbalance<T, U>;

		fn zero() -> Self {
			Self::new(Zero::zero())
		}
		fn drop_zero(self) -> result::Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self::new(first), Self::new(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self::new(a - b))
			} else {
				Err(NegativeImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T, U> TryDrop for NegativeImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		fn try_drop(self) -> result::Result<(), Self> {
			self.drop_zero()
		}
	}

	impl<T, U> Imbalance<T::Balance> for NegativeImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		type Opposite = PositiveImbalance<T, U>;

		fn zero() -> Self {
			Self::new(Zero::zero())
		}
		fn drop_zero(self) -> result::Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self::new(first), Self::new(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self::new(a - b))
			} else {
				Err(PositiveImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T, U> Drop for PositiveImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<super::ElevatedTrait<T>>>::mutate(&U::asset_id(), |v| *v = v.saturating_add(self.0));
		}
	}

	impl<T, U> Drop for NegativeImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<super::ElevatedTrait<T>>>::mutate(&U::asset_id(), |v| *v = v.saturating_sub(self.0));
		}
	}
}

// TODO: #2052
// Somewhat ugly hack in order to gain access to module's `increase_total_issuance_by`
// using only the Subtrait (which defines only the types that are not dependent
// on Positive/NegativeImbalance). Subtrait must be used otherwise we end up with a
// circular dependency with Trait having some types be dependent on PositiveImbalance<Trait>
// and PositiveImbalance itself depending back on Trait for its Drop impl (and thus
// its type declaration).
// This works as long as `increase_total_issuance_by` doesn't use the Imbalance
// types (basically for charging fees).
// This should eventually be refactored so that the two type items that do
// depend on the Imbalance type (TransactionPayment, DustRemoval)
// are placed in their own pallet.
struct ElevatedTrait<T: Subtrait>(T);
impl<T: Subtrait> Clone for ElevatedTrait<T> {
	fn clone(&self) -> Self {
		unimplemented!()
	}
}
impl<T: Subtrait> PartialEq for ElevatedTrait<T> {
	fn eq(&self, _: &Self) -> bool {
		unimplemented!()
	}
}
impl<T: Subtrait> Eq for ElevatedTrait<T> {}
impl<T: Subtrait> frame_system::Trait for ElevatedTrait<T> {
	type BaseCallFilter = T::BaseCallFilter;
	type MaximumExtrinsicWeight = T::MaximumBlockWeight;
	type SystemWeightInfo = ();

	type Origin = T::Origin;
	type Call = T::Call;
	type Index = T::Index;
	type BlockNumber = T::BlockNumber;
	type Hash = T::Hash;
	type Hashing = T::Hashing;
	type AccountId = T::AccountId;
	type Lookup = T::Lookup;
	type Header = T::Header;
	type Event = ();
	type BlockHashCount = T::BlockHashCount;
	type MaximumBlockWeight = T::MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumBlockLength = T::MaximumBlockLength;
	type AvailableBlockRatio = T::AvailableBlockRatio;
	type Version = T::Version;
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();


}
impl<T: Subtrait> Trait for ElevatedTrait<T> {

	type CouncilMembers = T::CouncilMembers;
	type MembersCount = T::MembersCount;
	type Currency = T::Currency;
	type Balance = T::Balance;
	type AssetId = T::AssetId;
	type Event = ();
	type CouncilOrigin = T::CouncilOrigin;
	type TechnicalOrigin = T::TechnicalOrigin;
	type TransxFoundation = T::TransxFoundation;
	type MaxLenOfMint  = T::MaxLenOfMint;

	type MaxLenOfBurn = T::MaxLenOfBurn;

	type TreasuryId = T::TreasuryId;
}

impl<T: Subtrait> nicks::Trait for ElevatedTrait<T> {
	type Event = ();
	type Currency_n = T::Currency_n;
	type ReservationFee = T::ReservationFee;
	type Slashed = T::Slashed;
	type ForceOrigin = T::ForceOrigin;
	type MinLength = T::MinLength;
	type MaxLength = T::MaxLength;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct AssetCurrency<T, U>(sp_std::marker::PhantomData<T>, sp_std::marker::PhantomData<U>);

impl<T, U> Currency<T::AccountId> for AssetCurrency<T, U>
where
	T: Trait,
	U: AssetIdProvider<AssetId = T::AssetId>,
{
	type Balance = T::Balance;
	type PositiveImbalance = PositiveImbalance<T, U>;
	type NegativeImbalance = NegativeImbalance<T, U>;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		Self::free_balance(&who) + Self::reserved_balance(&who)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		<Module<T>>::free_balance(&U::asset_id(), &who)
	}

	/// Returns the total staking asset issuance
	fn total_issuance() -> Self::Balance {
		<Module<T>>::total_issuance(U::asset_id())
	}

	fn minimum_balance() -> Self::Balance {
		Zero::zero()
	}

	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
		_: ExistenceRequirement, // no existential deposit policy for generic asset
	) -> DispatchResult {
		<Module<T>>::make_transfer(&U::asset_id(), transactor, dest, value)
	}

	fn ensure_can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
		reasons: WithdrawReasons,
		new_balance: Self::Balance,
	) -> DispatchResult {
		<Module<T>>::ensure_can_withdraw(&U::asset_id(), who, amount, reasons, new_balance)
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		reasons: WithdrawReasons,
		_: ExistenceRequirement, // no existential deposit policy for generic asset
	) -> result::Result<Self::NegativeImbalance, DispatchError> {
		let new_balance = Self::free_balance(who)
			.checked_sub(&value)
			.ok_or(Error::<T>::InsufficientBalance)?;
		Self::ensure_can_withdraw(who, value, reasons, new_balance)?;
		<Module<T>>::set_free_balance(&U::asset_id(), who, new_balance);
		Ok(NegativeImbalance::new(value))
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> result::Result<Self::PositiveImbalance, DispatchError> {
		// No existential deposit rule and creation fee in GA. `deposit_into_existing` is same with `deposit_creating`.
		Ok(Self::deposit_creating(who, value))
	}

	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		let imbalance = Self::make_free_balance_be(who, Self::free_balance(who) + value);
		if let SignedImbalance::Positive(p) = imbalance {
			p
		} else {
			// Impossible, but be defensive.
			Self::PositiveImbalance::zero()
		}
	}

	fn make_free_balance_be(
		who: &T::AccountId,
		balance: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		let original = <Module<T>>::free_balance(&U::asset_id(), who);
		let imbalance = if original <= balance {
			SignedImbalance::Positive(PositiveImbalance::new(balance - original))
		} else {
			SignedImbalance::Negative(NegativeImbalance::new(original - balance))
		};
		<Module<T>>::set_free_balance(&U::asset_id(), who, balance);
		imbalance
	}

	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		<Module<T>>::free_balance(&U::asset_id(), &who) >= value
	}

	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		let remaining = <Module<T>>::slash(&U::asset_id(), who, value);
		if let Some(r) = remaining {
			(NegativeImbalance::new(value - r), r)
		} else {
			(NegativeImbalance::new(value), Zero::zero())
		}
	}

	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		<TotalIssuance<T>>::mutate(&U::asset_id(), |issued|
			issued.checked_sub(&amount).unwrap_or_else(|| {
				amount = *issued;
				Zero::zero()
			})
		);
		PositiveImbalance::new(amount)
	}

	fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
		<TotalIssuance<T>>::mutate(&U::asset_id(), |issued|
			*issued = issued.checked_add(&amount).unwrap_or_else(|| {
				amount = Self::Balance::max_value() - *issued;
				Self::Balance::max_value()
			})
		);
		NegativeImbalance::new(amount)
	}
}

impl<T, U> ReservableCurrency<T::AccountId> for AssetCurrency<T, U>
where
	T: Trait,
	U: AssetIdProvider<AssetId = T::AssetId>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		Self::free_balance(who)
			.checked_sub(&value)
			.map_or(false, |new_balance|
				<Module<T>>::ensure_can_withdraw(
					&U::asset_id(), who, value, WithdrawReason::Reserve.into(), new_balance
				).is_ok()
			)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		<Module<T>>::reserved_balance(&U::asset_id(), &who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		<Module<T>>::reserve(&U::asset_id(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Module<T>>::unreserve(&U::asset_id(), who, value)
	}

	fn slash_reserved(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		let b = Self::reserved_balance(&who.clone());
		let slash = cmp::min(b, value);

		<Module<T>>::set_reserved_balance(&U::asset_id(), who, b - slash);
		(NegativeImbalance::new(slash), value - slash)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		Ok(<Module<T>>::repatriate_reserved(&U::asset_id(), slashed, beneficiary, value, status))
	}
}

pub struct StakingAssetIdProvider<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait> AssetIdProvider for StakingAssetIdProvider<T> {
	type AssetId = T::AssetId;
	fn asset_id() -> Self::AssetId {
		<Module<T>>::staking_asset_id()
	}
}

pub struct SpendingAssetIdProvider<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait> AssetIdProvider for SpendingAssetIdProvider<T> {
	type AssetId = T::AssetId;
	fn asset_id() -> Self::AssetId {
		<Module<T>>::spending_asset_id()
	}
}

impl<T> LockableCurrency<T::AccountId> for AssetCurrency<T, StakingAssetIdProvider<T>>
where
	T: Trait,
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	type Moment = T::BlockNumber;

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		reasons: WithdrawReasons,
	) {
		<Module<T>>::set_lock(id, who, amount, reasons)
	}

	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		reasons: WithdrawReasons,
	) {
		<Module<T>>::extend_lock(id, who, amount, reasons)
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		<Module<T>>::remove_lock(id, who)
	}
}

pub type StakingAssetCurrency<T> = AssetCurrency<T, StakingAssetIdProvider<T>>;
pub type SpendingAssetCurrency<T> = AssetCurrency<T, SpendingAssetIdProvider<T>>;
