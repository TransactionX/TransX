
use frame_support::{decl_module, decl_storage, decl_event, decl_error, weights::{Weight}, ensure, debug, StorageMap, StorageValue};
use frame_system as system;
use system::{ensure_signed, ensure_root};
use sp_std::{prelude::*, result, cmp, collections::btree_set::BTreeSet};
use sp_std::{result::Result};
use pallet_balances as balances;
use sp_std::convert::{TryInto,TryFrom, Into};
use codec::{Encode, Decode};
use sp_runtime::{DispatchResult, DispatchError, traits::{Hash}};
use frame_support::traits::{Get,
	Currency, ReservableCurrency, OnUnbalanced, Contains, EnsureOrigin, IsDeadAccount,
	GetMembers, ReportedTxs, LockableCurrency,
};
use sp_runtime::{Permill, ModuleId};
use sp_runtime::traits::{
	Zero, StaticLookup, AccountIdConversion, Saturating,
};
use crate::register::{AllMiners, BlackList, Trait as RegisterTrait};
use crate::register::{self, PledgeAmount, REGISTER_ID};
use crate::constants::time::*;
use crate::mine_linked::{MineTag};
use crate::mine::{self, OwnerMineRecord};
use pallet_elections_phragmen as elections_phragmen;

const MODULE_ID: ModuleId = ModuleId(*b"py/trsry");

type BalanceOf<T> = <<T as register::Trait>::Currency1 as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> = <<T as register::Trait>::Currency1 as Currency<<T as frame_system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as register::Trait>::Currency1 as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VoteInfo<Bo, A, Ba> {
	start_vote_block: Bo, // 开始投票的区块高度
	symbol: Vec<u8>,   // 币种
	tx: Vec<u8>,   // 交易tx
	reporter: A,  // 举报人
	report_reason: Vec<u8>,  // 举报理由
	illegal_man: A,  // 作弊者
	transaction_amount: Vec<u8>,  // 交易币额
	usdt_amount: Ba,  // usdt数额
	decimals: u32,  // 精度
	approve_mans: Vec<A>,  // 投赞成票的人
	reject_mans: Vec<A>,  // 投反对票的人
}


#[derive(Encode, Decode, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive())]
pub enum VoteRewardPeriodEnum{
	Days(u32),
	Minutes(u32),
	Hours(u32),
}


// 是否被惩罚
#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum IsPunished{
	YES,
	NO,
}


#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum TreasuryNeed{
	SUB,
	ADD,
}


// 投票结果
#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum VoteResult{
	PASS,
	NoPASS,
}



pub trait Trait: balances::Trait + RegisterTrait + mine::Trait{

	// 议会成员
	type ConcilOrigin: EnsureOrigin<Self::Origin, Success=Self::AccountId>;

	type ConcilCount: GetMembers<Self::AccountId>;

	type ConcilMembers: Contains<Self::AccountId>;

	type ShouldAddOrigin: OnUnbalanced<PositiveImbalanceOf<Self>>;
	type ShouldSubOrigin: OnUnbalanced<NegativeImbalanceOf<Self>>;

	// 议案过期时间
	type ProposalExpire: Get<Self::BlockNumber>;

//	// 每隔多久集体奖励一次
//	type VoteRewardPeriod: Get<Self::BlockNumber>;

	// 举报抵押金额
	type ReportReserve: Get<BalanceOf<Self>>;

	// 举报奖励
	type ReportReward: Get<BalanceOf<Self>>;

	// 撤销举报  举报者的惩罚金额
	type CancelReportSlash: Get<BalanceOf<Self>>;

	// 对作弊者的惩罚金额
	type IllegalPunishment: Get<BalanceOf<Self>>;

	// 奖励每个投票的议员多少
	type CouncilReward: Get<BalanceOf<Self>>;

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type DeadOrigin: IsDeadAccount<Self::AccountId>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as ReportModule {

		/// 所有还未奖励的投票的集合（一直到投票奖励完成才kill）
		pub Votes get(fn votes): map hasher(blake2_128_concat) Vec<u8> => VoteInfo<T::BlockNumber, T::AccountId, T::Balance>;

		/// 与自己有关的所有tx
		pub ManTxHashs get(fn mantxhashs): map hasher(blake2_128_concat) T::AccountId => Vec<Vec<u8>>;

		/// 已经通过但是还没有给予奖励的投票结果
		pub RewardList get(fn rewardlist): Vec<VoteInfo<T::BlockNumber, T::AccountId, T::Balance>>;


		pub VoteRewardPeriod get(fn vote_reward_period): T::BlockNumber;

		/// 正在进行投票的举报
		pub Voting get(fn voting): Vec<(Vec<u8>, VoteInfo<T::BlockNumber, T::AccountId, T::Balance>)>;

		/// 进入黑名单的所有信息 被永久保存  现在用tx做key
		pub AllPunishmentInfo get(fn allpunishmentinfo): map hasher(blake2_128_concat) Vec<u8> => VoteInfo<T::BlockNumber, T::AccountId, T::Balance>;

		pub ProposalExpire get(fn proposal_expire): T::BlockNumber;

		/// 个人正在被举报的tx
		pub BeingReportedTxsOf get(fn reported_txs_of): map hasher(blake2_128_concat) T::AccountId => BTreeSet<Vec<u8>>;

	}



}


decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {
		// 	在黑名单里
		InBlackList,

		// 还没有注册
		NotRegister,

		// 已经被举报
		BeingReported,

		// 金额太少
		BondTooLow,

		// 举报不存在
		ReportNotExists,

		// 不是本人
		NotSelf,

		// 已经结束的提案
		PassedProposal,

		// 自己是作弊方
		IllegalMan,

		// 重复投票
		RepeatVoteError,

		// 不是挖矿过的tx
		NotMinerTx,

		RepoterOrIllegalmanInBlackList,

		// 不在投票列表
		NotInVoteList,

		// 在被惩罚的队列里面
		InPunishmentList,

		// 自己是举报者不能参与投票
		Reporter,


	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

//		const VoteRewardPeriod: T::BlockNumber = T::VoteRewardPeriod::get();
		const ReportReserve: BalanceOf<T> = T::ReportReserve::get();
		const ReportReward: BalanceOf<T> = T::ReportReward::get();
		const CancelReportSlash: BalanceOf<T> = T::CancelReportSlash::get();
		const IllegalPunishment: BalanceOf<T> = T::IllegalPunishment::get();
		const CouncilReward: BalanceOf<T> = T::CouncilReward::get();

		type Error = Error<T>;
		pub fn deposit_event() = default;



		/// 设置统一奖励(或是惩罚)金额的周期
		#[weight = 500_000]
		fn set_vote_reward_duration(origin, time: T::BlockNumber) -> DispatchResult{

			ensure_root(origin)?;

			<VoteRewardPeriod<T>>::put(time);

			Ok(())
		}


		/// 设置投票过期时间
		#[weight = 500_000]
		fn set_proposal_expire_time(origin, time: T::BlockNumber) -> DispatchResult{

			ensure_root(origin)?;

			<ProposalExpire<T>>::put(time);

			Ok(())

		}


		/// 举报不良的挖矿
		#[weight = 500_000]
		pub fn report(origin, tx: Vec<u8>, mine_tag: MineTag, reason: Vec<u8>) -> DispatchResult{
			/// 举报不实交易
			let who = ensure_signed(origin)?;
			debug::warn!("-----report,account:{:?}------",who);
			let tx_info = if let Some(info) = <OwnerMineRecord<T>>::get(tx.clone(), mine_tag){
			info
			}
			else{
				return Err(Error::<T>::NotMinerTx)?;
			};
			#[cfg(feature = "std")]{
				debug::info!("tx_info:{:?}",tx_info);
			}

			let illegalman: T::AccountId = tx_info.miner_address;
			let decimals = tx_info.decimal;
			let symbol = tx_info.symbol;
			let usdt_amount: T::Balance = <<T as balances::Trait>::Balance as TryFrom::<u64>>::try_from(tx_info.usdt_amount).unwrap_or(T::Balance::default());

			let tx_amount = tx_info.sym_amount;

			// 如果作弊者和举报人有至少有一个在黑名单里， 则不给举报。
			ensure!(!<BlackList<T>>::contains_key(who.clone()) || !<BlackList<T>>::contains_key(illegalman.clone()), Error::<T>::RepoterOrIllegalmanInBlackList);

			// 被举报人必须是注册过的。
			ensure!(Self::is_register_member(illegalman.clone()), Error::<T>::NotRegister);

			// 根据tx判断这笔交易是否已经存在  已经存在的话不再添加进来
			ensure!(!<Votes<T>>::contains_key(&tx), Error::<T>::BeingReported);

			// 被举报人不能已经在被惩罚队列中
			ensure!(!<AllPunishmentInfo<T>>::contains_key(tx.clone()), Error::<T>::InPunishmentList);

			// 没有足够抵押资金，不给举报
			T::Currency1::reserve(&who, T::ReportReserve::get()).map_err(|_| Error::<T>::BondTooLow)?;

			// 获取当前区块高度
			let start_vote_block = <system::Module<T>>::block_number();
			let mut vote_info = VoteInfo{
				start_vote_block: start_vote_block.clone(),
				symbol: symbol.clone(),
				tx: tx.clone(),
				reporter: who.clone(),
				report_reason: reason.clone(),
				illegal_man: illegalman.clone(),
				transaction_amount: tx_amount.clone(),
				usdt_amount: usdt_amount.clone(),
				decimals: decimals.clone(),
				approve_mans: vec![],
				reject_mans:vec![],
			};
			// 判断投票者是否是议员
			if Self::is_concil_member(who.clone()) {
				vote_info.approve_mans.push(who.clone());
			}
			// 添加该投票的信息
			<Votes<T>>::insert(tx.clone(), vote_info.clone());
			// 添加人与相关交易映射
			Self::add_mantxhashs(who.clone(), tx.clone());
			Self::add_mantxhashs(illegalman.clone(), tx.clone());

			// 添加到正在进行投票的队列
			<Voting<T>>::mutate(|votes| votes.push((tx.clone(), vote_info.clone())));

			<BeingReportedTxsOf<T>>::mutate(illegalman.clone(), |h| h.insert(tx.clone()));

			Self::deposit_event(RawEvent::ReportEvent(start_vote_block, illegalman));
			Ok(())
		}


		/// 取消举报
		#[weight = 500_000]
		pub fn cancel_report(origin, tx: Vec<u8>) -> DispatchResult{
			/// 取消举报 只有该提案的举报者才有资格操作
			let who = ensure_signed(origin)?;
			// 交易不存在不能操作
			ensure!(<Votes<T>>::contains_key(tx.clone()), Error::<T>::ReportNotExists);
			let reporter = <Votes<T>>::get(tx.clone()).reporter;
			let illegalman = <Votes<T>>::get(tx.clone()).illegal_man;

			ensure!(!<BlackList<T>>::contains_key(who.clone()), Error::<T>::InBlackList);

			// 不是举报者本人则不能取消该举报提案
			ensure!((who.clone() == reporter.clone()), Error::<T>::NotSelf);

			// 如果提案已经结束 则不能再取消
			ensure!(Self::vote_result(<Votes<T>>::get(tx.clone())).0 == VoteResult::NoPASS, Error::<T>::PassedProposal);

			// 从Votes中删除该提案
			<Votes<T>>::remove(tx.clone());

			// 删除个人相关的tx
			Self::remove_mantxhashs(reporter.clone(),tx.clone());
			Self::remove_mantxhashs(illegalman.clone(),tx.clone());
			Self::deposit_event(RawEvent::RemoveManTxhashs(reporter.clone(), illegalman.clone()));

			// 归还举报者个人抵押
			T::Currency1::unreserve(&reporter, T::ReportReserve::get());

			// 惩罚举报者1个token
			T::ShouldSubOrigin::on_unbalanced(T::Currency1::slash(&reporter, T::CancelReportSlash::get()).0);
            // 从正在投票的队列中删除
			<Voting<T>>::mutate(|votes| votes.retain(|h| h.0 != tx.clone()));

			<BeingReportedTxsOf<T>>::mutate(illegalman.clone(), |h| h.remove(&tx));

			Self::deposit_event(RawEvent::CancelReportEvent(reporter.clone(), tx.clone()));

			Ok(())
		}


		/// 对举报提案集进行投票
		#[weight = 500_000]
		pub fn vote(origin, tx: Vec<u8>, yes_no: bool) -> DispatchResult{
			/// 投票 只有议员可以操作

			// 如果自己不是议会成员则不给操作
			let who = T::ConcilOrigin::ensure_origin(origin)?;
			// 判断这个tx是否存在于投票队列中，不存在则退出
			ensure!(<Votes<T>>::contains_key(&tx), Error::<T>::NotInVoteList);
			let illegalman = <Votes<T>>::get(&tx).illegal_man;
			let reporter = <Votes<T>>::get(&tx).reporter;

			// 自己是举报者不能参与投票
			ensure!(who.clone() != reporter, Error::<T>::Reporter);

			// 如果该投票的议员进入黑名单 则不能参与投票
			ensure!(!(<BlackList<T>>::contains_key(who.clone())), Error::<T>::InBlackList);

			// 如果这个议会成员是作弊者（被举报方），则禁止其投票。
			ensure!(!(illegalman.clone() == who.clone()), Error::<T>::IllegalMan);

			let vote_info = <Votes<T>>::get(&tx);

			// 如果举报者和作弊者有至少有一个在黑名单列表中， 则退出。
			if <BlackList<T>>::contains_key(reporter.clone()) || <BlackList<T>>::contains_key(illegalman.clone()){
				// 删除相关信息  并且退出
				Self::vote_expire_do(tx.clone(), vote_info);
				return Err(Error::<T>::InBlackList)?;
			}

			let now = <system::Module<T>>::block_number();
			// 过期删除相关信息  并且退出
			if now - vote_info.start_vote_block > <ProposalExpire<T>>::get(){
				Self::vote_expire_do(tx.clone(), vote_info);
				Self::deposit_event(RawEvent::RemoveManTxhashs(who.clone(), illegalman.clone()));
			}
			let mut voting = <Votes<T>>::get(&tx);
			let position_yes = voting.approve_mans.iter().position(|a| a == &who);
			let position_no = voting.reject_mans.iter().position(|a| a == &who);
			// 如果投赞成票
			if yes_no{
				if position_yes.is_none(){
					voting.approve_mans.push(who.clone());
				}
				else{
					Err(Error::<T>::RepeatVoteError)?
				}
				if let Some(pos) = position_no{
					voting.reject_mans.swap_remove(pos);
				}
			}
			// 如果投的是反对票
			else{
				if position_no.is_none(){
					voting.reject_mans.push(who.clone());
				}
				else{
					Err(Error::<T>::RepeatVoteError)?
				}
				if let Some(pos) = position_yes{
					voting.approve_mans.swap_remove(pos);
				}
			}
			<Votes<T>>::insert(tx.clone(), voting.clone());
			// 判断议案是否结束
			let vote_result = Self::vote_result(voting.clone());
			// 如果议案投票已经结束
			if vote_result.0 == VoteResult::PASS{

				// 把该投票结果存储到奖励名单
				<RewardList<T>>::mutate(|a| a.push(voting.clone()));
				Self::remove_mantxhashs(reporter.clone(),tx.clone());
				Self::remove_mantxhashs(illegalman.clone(),tx.clone());
				Self::deposit_event(RawEvent::RemoveManTxhashs(who.clone(), illegalman.clone()));
				// 如果作弊是真  把名字加入黑名单  并且从注册列表中删除  把该投票信息保存
				if vote_result.1 == IsPunished::YES{

					// 先于奖励加入黑名单
					<BlackList<T>>::insert(illegalman.clone(), tx.clone());
					Self::kill_register(illegalman.clone());
					Self::deposit_event(RawEvent::KillRegisterEvent(illegalman.clone()));

					// 永久保存该投票信息
					<AllPunishmentInfo<T>>::insert(tx.clone(), voting.clone());
				}

				// 从正在投票队列中移除
				Self::remove_voting(tx.clone())
			}

			else{
				// 修改正在投票的队列
				<Voting<T>>::mutate(|votes| votes.retain(|h| h.0 != tx));
				<Voting<T>>::mutate(|votes| votes.push((tx.clone(), voting.clone())));
			}

			Self::deposit_event(RawEvent::VoteEvent(illegalman.clone()));
			Ok(())
		}


		// 每次出块结束都要去计算一下是否是奖励时间 如果是则奖励
		fn on_finalize(n: T::BlockNumber){

			if !<VoteRewardPeriod<T>>::get().is_zero(){
				if (n % <VoteRewardPeriod<T>>::get()).is_zero() {  // 默认一天奖励一次
					Self::reward();  // 奖励的方法
			}

			// 奖励时间到优先奖励
			else{
				Self::remove_expire_voting(n);
			}

			}
			else{
				   assert!(1==2, "period is zero");

				}
			}
	}
}


decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId,
		<T as system::Trait>::BlockNumber,
		<T as balances::Trait>::Balance,
	 {

		// 开始的区块 被举报者姓名
		ReportEvent(BlockNumber, AccountId),

		// 取消提案
		CancelReportEvent(AccountId, Vec<u8>),

		//删除mantxhashs
		RemoveManTxhashs(AccountId, AccountId),

		// 正在投谁的票
		VoteEvent(AccountId),

		KillRegisterEvent(AccountId),

		// 谁的议案通过了
		VoteFinishEvent(AccountId),

		// 返回一个数组
		TreasuryEvent(bool, Balance),

		// 谁的票奖励结束了 tx哈希是多少
		RewardEvent(AccountId, Vec<u8>),

		SomethingStored(u32, AccountId),
	}
);

impl<T: Trait> Module <T> {


	pub fn remove_voting(tx: Vec<u8>){
		<Voting<T>>::mutate(|votes| votes.retain(|h| h.0 != tx.clone()));
	}


	pub fn remove_expire_voting(n: T::BlockNumber){

		<Voting<T>>::mutate(|votes| votes.retain(|vote| if n - <Votes<T>>::get(&vote.0).start_vote_block <= <ProposalExpire<T>>::get(){
			true
		}
			else {
				let vote_info = <Votes<T>>::get(&vote.0);
				Self::vote_expire_do(vote_info.clone().tx, vote_info.clone());
				false
			}

		));

	}


	// 获取议会成员数目
	fn get_members_count() -> u32{
		T::ConcilCount::get_members_len()
	}


	// 判断投票过期后做的
	pub fn vote_expire_do(tx: Vec<u8>,  vote_info: VoteInfo<T::BlockNumber, T::AccountId, T::Balance>){
		let reporter = vote_info.reporter;
		let illegalman = vote_info.illegal_man;
		<Votes<T>>::remove(&tx);
		// TODO 添加和删除方法均已经实现， 注意查看代码是否正确
		// 把举报者的抵押归还
		T::Currency1::unreserve(&reporter, T::ReportReserve::get());
		// 删除相关的man thhashs信息
		Self::remove_mantxhashs(reporter.clone(), tx.clone());
		Self::remove_mantxhashs(illegalman.clone(), tx.clone());

		<BeingReportedTxsOf<T>>::mutate(illegalman.clone(), |h| h.remove(&tx));
		// 从正在投票队列中移除
		Self::remove_voting(tx.clone());


	}


	pub fn reward() -> Result<(), DispatchError> {
		// 计算国库还有多少钱
		let mut useable_balance = Self::treasury_useable_balance();
		// 获取国库id
		let treasury_id = Self::get_treasury_id();

		// 这一步按照两个步骤来走
		for i in 0..2 {
			<RewardList<T>>::mutate(|v| {
			v.retain(|voteinfo| {
				let is_punish = Self::vote_result(voteinfo.clone()).1;
				let treasury_result = Self::treasury_imbalance(is_punish.clone(), voteinfo.clone());
				let sub_or_add = treasury_result.0;
				let imbalances = treasury_result.1;

				// 如果国库需要添加金额
				if sub_or_add == TreasuryNeed::ADD{
					// 给国库增加金额
					useable_balance += imbalances;
					T::Currency1::make_free_balance_be(&treasury_id, useable_balance + T::Currency1::minimum_balance());
					Self::everyone_balance_oprate(is_punish.clone(), voteinfo.clone());

					let illegalman = voteinfo.illegal_man.clone();
					<BeingReportedTxsOf<T>>::mutate(illegalman, |h|  h.remove(&voteinfo.tx));

					// 彻底删掉投票信息
					<Votes<T>>::remove(voteinfo.clone().tx);

					false
				}
					// 如果国库需要减掉金额
				else{
					if useable_balance >= imbalances{
						// 给国库减掉金额
						useable_balance -= imbalances;
						T::Currency1::make_free_balance_be(&treasury_id, useable_balance + T::Currency1::minimum_balance());

						let illegalman = voteinfo.illegal_man.clone();
						<BeingReportedTxsOf<T>>::mutate(illegalman, |h|  h.remove(&voteinfo.tx));

						// 彻底删掉投票信息
						<Votes<T>>::remove(voteinfo.clone().tx);
						Self::everyone_balance_oprate(is_punish.clone(), voteinfo.clone());
						false
					}
						// 金额不够 暂时不执行
					else{
						true
					}
				}
			});
		});
		}
		Ok(())
	}


	// 这个方法用来判断是否是议会成员
	pub fn is_concil_member(who: T::AccountId) -> bool {
		if T::ConcilMembers::contains(&who){
			true
		}
		else {
			false
		}
	}


	// 是否在矿机的注册名单里面
	pub fn is_register_member(who: T::AccountId) -> bool {
		if <AllMiners<T>>::contains_key(&who){
			true
		}
		else {
			false
		}
	}


	// 把该名单从注册列表删除
	pub fn kill_register(who: T::AccountId) {
		<register::Module<T>>::kill_man(who.clone());

	}


	//获取国库id
	pub fn get_treasury_id() -> T::AccountId {
		MODULE_ID.into_account()
	}


	pub fn treasury_useable_balance() -> BalanceOf<T> {
		T::Currency1::free_balance(&Self::get_treasury_id())
			// Must never be less than 0 but better be safe.
			.saturating_sub(T::Currency1::minimum_balance())
	}


	// 添加man hashs映射添加相关信息
	pub fn add_mantxhashs(who: T::AccountId, tx: Vec<u8>) {
		let mut vec_txhash = vec![];
		if <ManTxHashs<T>>::contains_key(&who) {
			vec_txhash = <ManTxHashs<T>>::get(&who);
			vec_txhash.push(tx);
		} else {
			vec_txhash.push(tx)
		}
		<ManTxHashs<T>>::insert(&who, &vec_txhash);
	}


	// 删除man txhashs相关信息
	pub fn remove_mantxhashs(who: T::AccountId, tx: Vec<u8>) {
		let mut vec_txhash = vec![];
		vec_txhash = <ManTxHashs<T>>::get(&who);
		if let Some(pos) = vec_txhash.iter().position(|a| <Votes<T>>::contains_key(&tx)) {
			vec_txhash.swap_remove(pos);
		};
		if vec_txhash.len() == 0 {
			<ManTxHashs<T>>::remove(&who)
		} else {
			<ManTxHashs<T>>::insert(&who, &vec_txhash);
		}
	}


	// 这个方法用于验证投票是否结束（是否有一方胜出）
	pub fn vote_result(vote_info: VoteInfo<T::BlockNumber, T::AccountId, T::Balance>)
		-> (VoteResult, IsPunished) {

		let approve_len = vote_info.approve_mans.len() as u32;
		let reject_len = vote_info.reject_mans.len() as u32;
		// 胜出两票或是有一方先过半 那么就结束

		let n = cmp::max(approve_len, reject_len) - cmp::min(approve_len, reject_len);
		let thredshould = (Self::get_members_count() + 1u32)/2u32;
		// 相等情况下赞同优先
		if (thredshould > 0u32) && (n >= 2 || approve_len >= thredshould || reject_len >= thredshould){
			if approve_len >= reject_len {
				(VoteResult::PASS, IsPunished::YES)
			} else {
				(VoteResult::PASS, IsPunished::NO)
			}

		} else {
			(VoteResult::NoPASS, IsPunished::NO)
		}

	}


	// 计算国库盈余或是亏损多少  第一个参数返回true是盈余  返回false是亏损
	pub fn treasury_imbalance(is_punish: IsPunished, vote:
	VoteInfo<T::BlockNumber, T::AccountId, T::Balance>) -> (TreasuryNeed, BalanceOf<T>) {

		let reporter = vote.clone().reporter;
		let illegalman = vote.clone().illegal_man;
		let mut postive: BalanceOf<T> = 0.into();
		let mut negative: BalanceOf<T> = 0.into();
		// 真的作弊
		if is_punish == IsPunished::YES {
			if T::Currency1::total_balance(&illegalman) >= T::IllegalPunishment::get(){
				postive = T::IllegalPunishment::get();
			}
			else{
				postive = T::Currency1::total_balance(&illegalman);
			}

			// 奖励举报者的总金额(如果这个人已经在黑名单， 则不给奖励)
			if !<BlackList<T>>::contains_key(reporter.clone()) {
				// 数额巨大 不需要考虑存活问题
				negative = T::ReportReward::get();
			}
		}

		// 虚假举报
		else {
			// 账户必须要存在(不管你是否在黑名单 均会扣除费用)
			if !(T::DeadOrigin::is_dead_account(&reporter)){
				if T::Currency1::total_balance(&reporter) >= T::ReportReserve::get(){
					postive += T::ReportReserve::get().clone();
				}
				else{
					postive += T::Currency1::total_balance(&reporter);
				}

			}
		}

		// 议员总奖励金额
		let mut all_mans =
			vote.reject_mans.iter().chain(vote.approve_mans.iter());

		for i in 0..all_mans.clone().count() {
			if let Some(peaple) = all_mans.next() {
				// 如果已经进入黑名单 则不给奖励
				if !<BlackList<T>>::contains_key(&peaple){
					// 如果账户还存在
					if !(T::DeadOrigin::is_dead_account(&peaple)){
						negative += T::CouncilReward::get().clone();
					}

					// 如果账户已经不存在
					else{
						if T::ReportReward::get() >= T::Currency1::minimum_balance(){
							negative += T::CouncilReward::get().clone();
						}
					}
				}

			};
		}

		// 惩罚金额如果还有 那就直接给国库
		if postive > negative {
			(TreasuryNeed::ADD, postive - negative)

		// 惩罚金额没有剩余 从国库扣除
		} else {
			(TreasuryNeed::SUB, negative - postive)
		}
	}


	// 这个方法用来操作除了国库之外的跟此次投票有关的人员的金额
	pub fn everyone_balance_oprate(is_punish: IsPunished,
								   vote: VoteInfo<T::BlockNumber, T::AccountId, T::Balance>){

		let reporter = vote.clone().reporter;
		let illegalman = vote.clone().illegal_man;
		// 真的作弊
		if is_punish == IsPunished::YES {

			// 释放锁并且扣除金额
			T::Currency1::remove_lock(REGISTER_ID, &illegalman);
			T::Currency1::slash(&illegalman, T::IllegalPunishment::get());

			// 数额巨大 不需要考虑存活问题
			T::Currency1::unreserve(&reporter, T::ReportReserve::get());

			// 举报者如果已经在黑名单 则不给奖励
			if !<BlackList<T>>::contains_key(&reporter){
				T::Currency1::deposit_creating(&reporter, T::ReportReward::get());
			}

		}

		// 虚假举报
		else {
			// 惩罚举报者的金额
				// 账户必须要存在
			if !(T::DeadOrigin::is_dead_account(&reporter)){

				T::Currency1::slash_reserved(&reporter, T::ReportReserve::get());

			}

			}

		// 议员总奖励金额
		let mut all_mans =
			vote.reject_mans.iter().chain(vote.approve_mans.iter());

		for i in 0..all_mans.clone().count() {
			if let Some(peaple) = all_mans.next() {
				if !<BlackList<T>>::contains_key(&peaple){
					if !(T::DeadOrigin::is_dead_account(&peaple)){
					T::Currency1::deposit_creating(&peaple, T::CouncilReward::get());
					}

					// 如果账户已经不存在
					else{
						if T::ReportReward::get() >= T::Currency1::minimum_balance(){
							T::Currency1::deposit_creating(&peaple, T::CouncilReward::get());
						}
					}
				}

			};
		}
	}


//	fn initialize_mutable_parameter(params: &[T::AccountId]){
//		<VoteRewardPeriod<T>>::put(T::VoteRewardPeriod::get());
//		<ProposalExpire<T>>::put(T::ProposalExpire::get());
//	}


}


impl<T: Trait> ReportedTxs<T::AccountId> for Module<T>{
	fn is_reported(who: T::AccountId) -> bool{
		!<BeingReportedTxsOf <T>>::get(who).is_empty()
	}
}


// ***************************************对数值进行限制*********************************************
trait RewardPeriodBound{
	fn get_bound_period(x: u32) -> result::Result<u32, &'static str>;
}

impl RewardPeriodBound for u32{
	fn get_bound_period(x: u32) -> result::Result<u32, &'static str> {
		match x {
			0 => Err("输入非法数据0"),
			_ => Ok(x)
		}
	}
}



