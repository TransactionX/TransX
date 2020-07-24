
type BalanceOf<T> = <<T as Trait>::Currency1 as Currency<<T as system::Trait>::AccountId>>::Balance;
use frame_support::traits::{Get,
	Currency, ReservableCurrency, LockIdentifier,
	WithdrawReasons, LockableCurrency,
};
use sp_std::{prelude::*, result::Result};
use frame_support::{debug, ensure, decl_module, decl_storage, decl_error, decl_event, weights::{Weight},
					StorageValue, StorageMap, StorageDoubleMap, Blake2_256};
use frame_system as system;
use system::{ensure_signed, ensure_root};
use sp_runtime::{DispatchResult, Perbill, Permill, Percent};
use pallet_timestamp;
use codec::{Encode, Decode};
use crate::constants::symbol::*;

pub const REGISTER_ID: LockIdentifier = *b"register";


// 机器状态
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum MinerStatus{
	Success,
	Invalid,
	Slashed,
}

impl Default for MinerStatus{
	fn default() -> Self {
		Self::Success
	}
}


#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
// 这个用于表述地址状态
pub enum AddressStatus{
	active,  // 已经激活
	inActive,  // 未激活
}

// enum中derive不了Default
impl Default for AddressStatus{
	fn default() -> Self {
		Self::inActive
	}
}


#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode, Clone, Default)]  // 应该是有了option就必须要实现Default
pub struct MinerInfo<A, M, S> {
	hardware_id: Vec<u8>,
	pub father_address: Option<A>,
	pub grandpa_address: Option<A>,
	register_time: M,
	pub machine_state: S,  // 暂时用字符串表示
	machine_owner: A,
}



pub trait Trait: pallet_timestamp::Trait + system::Trait{

	/// The overarching event type.
	type PledgeAmount: Get<BalanceOf<Self>>;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency1: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

	// 一条链允许换地址的最大次数
	type ChangeAddressMaxCount: Get<u32>;

	type TxsMaxCount: Get<u32>;

	// 解压需要的时间
	type UnBondingDuration: Get<Self::BlockNumber>;
}


decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		pub AllMiners get(fn allminers): map hasher(blake2_128_concat)  T::AccountId => MinerInfo<T::AccountId, T::Moment, MinerStatus>;

		// 用来临时存储查找的账户的所有地址信息
		// (token_address, AddressStatus, tx, symbol)
		pub AddressOf get(fn address_of): map hasher(blake2_128_concat)  T::AccountId =>  Vec<(Vec<u8>, AddressStatus, Vec<u8>, Vec<u8>)>;

		// AccountId, symbol => (token_address, AddressStatus, tx, symbol)
		//  激活地址后更改状态为激活
		pub TokenInfo: double_map hasher(blake2_128_concat) T::AccountId,  hasher(blake2_128_concat) Vec<u8> => (Vec<u8>, AddressStatus, Vec<u8>, Vec<u8>);

        // 设置状态位  u32表现形式1xxx.初始值为 1000,低3位分别表示 验证通过次数,验证失败次数,非正常情况返回次数, tx => 1xxx,AccountId, symbol
		pub TokenStatus get(fn tx_status): map hasher(blake2_128_concat) Vec<u8> => (u64,T::AccountId,Vec<u8>);
		// 记录 TokenStatus 的长度,防止队列过大
		pub TokenStatusLen: u32;

		AllRegisters get(fn allregisters):  map hasher(blake2_128_concat)  Vec<u8> => T::AccountId;

		// 已经被用来挖矿的各个链的地址
		// 这个要求全网唯一  不可多人共用
		// (symbol, address) => account_id
		// todo 激活地址后insert的信息
		pub AddressUsedForMiner get(fn address_used_for_miner): map hasher(blake2_128_concat)  (Vec<u8>, Vec<u8>) => T::AccountId;

        // 每一个symbol更改地址(真正被用）的次数 用来限制用户更改地址的次数
        // (account_id, symbol) => count
        // 不在这个模块中更改 因为这个模块不确保可用
        // todo 激活地址后需要 +1 信息
		pub ChangeAddressCount get(fn change_address_count): map hasher(blake2_128_concat)  (T::AccountId, Vec<u8>) => u32;

		// 每一个矿工正在使用的地址
		// 不是在这个模块添加 因为这个模块不保证可用
		//account_id => Vec<(symbol, address)>
		// todo 激活地址后需要append的信息
		pub PerMinerUsingAddress get(fn per_miner_using_address): map hasher(blake2_128_concat)  T::AccountId => Vec<(Vec<u8>, Vec<u8>)>;

		// 黑名单  只有举报模块调用option比较好 如果没有就返回None 而不是默认值
		// 使用
		pub BlackList get(fn blacklist): map hasher(blake2_128_concat)  T::AccountId => Option<Vec<u8>>;
		pub MinersCount: u64;

		// 用户的下家
		pub MinerChildren get(fn miner_children): map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;

		// ***治理参数***
		pub PledgeAmount get(fn pledge_amount): BalanceOf<T>;

		pub AddressDefaultStatus get(fn address_default_status): AddressStatus;

		// 所有矿工可解压的到期时间
		pub UnbondTimeOfMiners get(fn unbond_time_of_miners): map hasher(blake2_128_concat) T::AccountId => T::BlockNumber;


	}
	add_extra_genesis {
			config(donothing): Vec<T::AccountId>;
			build(|config| {
				<Module<T>>::initialize_mutable_parameter(&config.donothing);
				})
			}
}


decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {

		// 没有填写硬件id
		HardIdIsNone,

		// 已经注册（不能再注册）
		AlreadyRegisted,

		// 被举报进入黑名单的成员（不能注册)
		InBlackList,

		// 硬件id已经被使用
		HardIdBeenUsed,

		// 金额太低(不够抵押)
		BondTooLow,

		// 数目溢出
		Overflow,

		// 还没有注册
		NotRegister,

		// 上级没有注册过矿机
		FatherNotRegister,

		// 上级是自己
		FatherIsYourself,

		// vec数组为空
		VarEmpty,

		// address被别人使用
		InUseAddress,

		// 更改次数过多
		ChangeTooMore,

		// 该币种信息未存在
		SymbolNotExists,

		// tx 正在占用中,请稍后再试
		TxInUsing,

		// txsoverlimit
		OverMaximum,
		// 不支持的币种
		UnknownSymbol,

		// 没有注销过账号
		NotInWithdrawList,

		// 没有到释放锁的时间
		NotUnbondTime,
	}
}



decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		type Error = Error<T>;
		fn deposit_event() = default;


		#[weight = 500_000]
		pub fn register(origin, hardware_id: Vec<u8>, father_address: Option<T::AccountId>) -> DispatchResult{
			/// register the machine.
			let who = ensure_signed(origin)?;

			ensure!(!(hardware_id.len() == 0), Error::<T>::HardIdIsNone);
			// Vec<u8>类型参数不能为空

			ensure!(!<AllMiners<T>>::contains_key(who.clone()), Error::<T>::AlreadyRegisted);
			// 账户已经存在不需要注册！

			ensure!(!<BlackList<T>>::contains_key(who.clone()),  Error::<T>::InBlackList);
			// 如果账户已经进入黑名单， 则不能再注册

			ensure!(!<AllRegisters<T>>::contains_key(hardware_id.clone()), Error::<T>::HardIdBeenUsed);
			// 硬件已经被注册则不能再次注册。

			// 如果写了上级地址
			if let Some(father_address) = father_address.clone(){
				// 上级地址不能是自己
				ensure!(who.clone() != father_address.clone(), Error::<T>::FatherIsYourself,);
				// 上级必须是注册过矿机的
				ensure!(<AllMiners<T>>::contains_key(father_address.clone()), Error::<T>::FatherNotRegister);

			}

			let bond :BalanceOf<T> = <PledgeAmount<T>>::get();

			// 判断是否有足够钱可以抵押
			if !T::Currency1::can_reserve(&who, bond.clone()){
				return Err(Error::<T>::BondTooLow)?;
			}
			// 进行琐仓操作
			T::Currency1::set_lock(
				REGISTER_ID,
				&who,
				bond.clone(),
				WithdrawReasons::all(),
				);

			let register_time = <pallet_timestamp::Module<T>>::get();
			// 添加注册时间

			let mut minerinfo = MinerInfo{
				hardware_id:  hardware_id.clone(),
				father_address: None,  // 上级默认是None
				grandpa_address: None,  // 上上级默认是None
				register_time: register_time.clone(),
				machine_state: MinerStatus::Success,
				machine_owner: who.clone(),
			};

			if let Some(father_address) = father_address.clone(){
				minerinfo.father_address = Some(father_address.clone());

				// 我的上级添加我作为他的下级（记录，方便每个用户能够找到自己的下级）
				<MinerChildren<T>>::mutate(father_address.clone(), |v|  v.push(who.clone()));

				// 上级不能是自己  默认一定要填一个  填自己的话就返回none 如果填的那个人没有注册矿机 则返回上级是None

				if <AllMiners<T>>::contains_key(father_address.clone()){
					let tmpt =  <AllMiners<T>>::get(father_address.clone()).father_address.unwrap_or(who.clone());
					if who.clone() != tmpt {
						minerinfo.grandpa_address = Some(tmpt);
					}
				}
				// 上上级不能是自己

			}

			<AllMiners<T>>::insert(who.clone(), minerinfo.clone());
			// 添加矿机信息完毕

			<AllRegisters<T>>::insert(hardware_id.clone(), who.clone());
			// 添加映射 矿机id => 用户id

			let allminerscount = MinersCount::get();
			let new_allminerscount = allminerscount.checked_add(1).ok_or(Error::<T>::Overflow)?;
			MinersCount::put(new_allminerscount);
			// 矿机数加1

			Self::deposit_event(RawEvent::RegisterEvent(allminerscount, who.clone(), register_time.clone()));
			// 触发事件
			Ok(())
		}


		#[weight = 500_000]
		fn set_default_address_status(origin, status: AddressStatus){
			ensure_root(origin)?;
			<AddressDefaultStatus>::put(status);
		}


		#[weight = 500_000 ]
		pub fn withdraw(origin) -> DispatchResult {
			/// 注销注册的账户 并归还抵押金额
			let who = ensure_signed(origin)?;

			ensure!(<AllMiners<T>>::contains_key(who.clone()), Error::<T>::NotRegister);
			// 如果还没有注册， 则直接退出

			let now = <system::Module<T>>::block_number();
			Self::kill_man(who.clone());

			let bond :BalanceOf<T> = T::PledgeAmount::get();
			<UnbondTimeOfMiners<T>>::insert(who.clone(), now + T::UnBondingDuration::get());

			// 归还抵押
			Self::deposit_event(RawEvent::Withdraw(who.clone()));
			Ok(())
		}


		/// 自动注销账户时过一段时间自己解压
		#[weight = 500_000 ]
		pub fn withdraw_unbonded(origin) -> DispatchResult{
			let who = ensure_signed(origin)?;
			if <UnbondTimeOfMiners<T>>::contains_key(who.clone()){
				let now = <system::Module<T>>::block_number();
				let unbond_time = <UnbondTimeOfMiners<T>>::get(who.clone());
				if now > unbond_time{
					T::Currency1::remove_lock(REGISTER_ID, &who);
					<UnbondTimeOfMiners<T>>::remove(who.clone());
				}
				else{
					return Err(Error::<T>::NotUnbondTime)?;
				}
			}
			else{
				return Err(Error::<T>::NotInWithdrawList)?;
			}
			Self::deposit_event(RawEvent::WithdrawUnbond(who.clone()));
			Ok(())
		}



		#[weight = 500_000]
		pub fn add_token_info(origin, symbol: Vec<u8>, tokenaddress: Vec<u8>, tx: Vec<u8>) -> DispatchResult {  // 一个symbol值有一个地址
			/// 给注册过的用户添加token信息
			let who = ensure_signed(origin)?;

			//**********支持挖矿的币种********************
			let btc = BTC.as_bytes().to_vec();
			let eth = ETH.as_bytes().to_vec();
			let usdt = USDT.as_bytes().to_vec();
			let eos = EOS.as_bytes().to_vec();
			let ecap = ECAP.as_bytes().to_vec();
			//*************在这里添加********************

			 match symbol{
				_ if btc.clone() == symbol => debug::info!("add btc info"),
				_ if usdt.clone() == symbol => debug::info!("add usdt info"),
				_ if eth.clone() == symbol => debug::info!("add eth info"),
				_ if eos.clone() == symbol => debug::info!("add eos info"),
				_ if ecap.clone() == symbol => debug::info!("add ecap info"),
				_ => return Err(Error::<T>::UnknownSymbol)?
			};

			ensure!(!(symbol.len() == 0), Error::<T>::VarEmpty);
			ensure!(!(tokenaddress.len()==0), Error::<T>::VarEmpty);
			// Vec<u8>参数不能为空

			ensure!(<AllMiners<T>>::contains_key(who.clone()), Error::<T>::NotRegister);
			// 如果还没有注册， 则直接退出

			ensure!(!<AddressUsedForMiner<T>>::contains_key((symbol.clone(), tokenaddress.clone())), Error::<T>::InUseAddress);
			// 如果已经有人用了这个地址 则不能再使用

			ensure!(<ChangeAddressCount<T>>::get((who.clone(), symbol.clone())) < T::ChangeAddressMaxCount::get(), Error::<T>::ChangeTooMore);
			// 每条链的地址最多能改两次

			<TokenInfo<T>>::insert(who.clone(), symbol.clone(), (tokenaddress.clone(), <AddressDefaultStatus>::get(), tx.clone(), symbol.clone()));

			// 初始化状态码
			let curent_status = <TokenStatus<T>>::get(tx.clone()).0;
			// 等于0表示没被占用,继续向下执行
			ensure!(<TokenStatus<T>>::get(tx.clone()).0 == 0, Error::<T>::TxInUsing);
			debug::RuntimeLogger::init();
			debug::info!("当前长度: {:?}",TokenStatusLen::get());
			ensure!(TokenStatusLen::get() <= T::TxsMaxCount::get(), Error::<T>::OverMaximum);
			<TokenStatus<T>>::insert(tx.clone(),(1000,who.clone(), symbol.clone()));
			debug::info!("TokenStatus 初始化状态为:{:?}",<TokenStatus<T>>::get(tx.clone()).0);
			TokenStatusLen::mutate(|n|*n += 1);
			Self::deposit_event(RawEvent::AddTokenInfoEvent(who, symbol));
			Ok(())

			}


		#[weight = 10000_000 ]
		pub fn remove_token_info(origin, symbol: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!(symbol.len() == 0), Error::<T>::VarEmpty);

			ensure!(<AllMiners<T>>::contains_key(who.clone()), Error::<T>::NotRegister);
			// 不是已经注册的账户，不可查。

			ensure!(<TokenInfo<T>>::contains_key(who.clone(), symbol.clone()), Error::<T>::SymbolNotExists);
			// 如果本来就不存在， 则退出。

			// 获取地址
			let address = <TokenInfo<T>>::get(who.clone(), symbol.clone()).0;

			ensure!(!(<AddressUsedForMiner<T>>::contains_key((symbol.clone(), address.clone())) && <ChangeAddressCount<T>>::get((who.clone(), symbol.clone())) >= T::ChangeAddressMaxCount::get()), Error::<T>::ChangeTooMore);
			// 如果这个地址正在被占用，并且删除后超过最大可更改次数 则不能删除

			let address = <TokenInfo<T>>::get(who.clone(), symbol.clone()).0;

			 if <AddressUsedForMiner<T>>::contains_key((symbol.clone(), address.clone())){
			 	<AddressUsedForMiner<T>>::remove((symbol.clone(), address.clone()));

			 	let position = <PerMinerUsingAddress<T>>::get(who.clone()).iter().position(|x| x.clone().0 == symbol.clone());
			 	if let Some(x) = position{
			 		let mut miner_using_address = <PerMinerUsingAddress<T>>::get(who.clone());
			 		miner_using_address.swap_remove(x);
			 		if miner_using_address.is_empty(){
						<PerMinerUsingAddress<T>>::remove(who.clone());
					}
					else{
						<PerMinerUsingAddress<T>>::insert(who.clone(), miner_using_address);
					}
			 	}

			 }
			 // 把全网使用的这个token以及对应的address删除  以防下次不能使用

			<TokenInfo<T>>::remove(who.clone(), symbol.clone());
			// 删除该key

			let mut address_info = <AddressOf<T>>::get(who.clone());
			address_info.retain(|x| x.3 != symbol.clone());
			<AddressOf<T>>::insert(who.clone(), address_info);

			Self::deposit_event(RawEvent::RemoveTokenInfoEvent(who, symbol));
			Ok(())
		}


		#[weight = 500_000]
		fn set_pledgeamount(origin, bond: BalanceOf<T>){
			/// 设置注册抵押金额
			ensure_root(origin)?;
			<PledgeAmount<T>>::put(bond);

	}

	}
}

decl_event!(
	pub enum Event<T> where
	 <T as system::Trait>::AccountId,
	 <T as pallet_timestamp::Trait>::Moment {
		// Just a dummy event.

		RegisterEvent(u64, AccountId, Moment),
		AddTokenInfoEvent(AccountId, Vec<u8>),
		RemoveTokenInfoEvent(AccountId, Vec<u8>),
		KillRegisterEvent(AccountId),
		WithdrawUnbond(AccountId),
		Withdraw(AccountId),
	}
);

impl <T: Trait> Module <T> {


	pub fn kill_man(who: T::AccountId) {

			let hardware_id = <AllMiners<T>>::get(who.clone()).hardware_id;
			// 获取硬件id

			let miner_info = <AllMiners<T>>::get(who.clone());
			if let Some(father_address) = miner_info.father_address{
				<MinerChildren<T>>::mutate(father_address.clone(), |v| v.retain(|h| h  != &who));
				// 上级删除掉我的记录
			};

			<MinerChildren<T>>::take(who.clone());
			// 删除掉自己的下级记录

			<AllMiners<T>>::remove(who.clone());
			// 从矿机列表删除该账户

			<AllRegisters<T>>::remove(hardware_id.clone());
			// 从AllRegisters列表中删除记录

			<PerMinerUsingAddress<T>>::get(who.clone()).retain(|x| {
				if <AddressUsedForMiner<T>>::contains_key(x.clone()){
					<AddressUsedForMiner<T>>::remove(x.clone());
				}
				if <ChangeAddressCount<T>>::contains_key((who.clone(), x.clone().0)){
					<ChangeAddressCount<T>>::remove((who.clone(), x.clone().0))

				}
				false  // 不管怎样均删除掉
			}
			);
			<PerMinerUsingAddress<T>>::remove(who.clone());
			// 删除掉AddressUsedForMiner与ChangeAddressCount中相应记录

			let minercount = MinersCount::get();
			let new_minercount = minercount - 1;
			MinersCount::put(new_minercount);
			// 矿机数减掉1

			<AddressOf<T>>::remove(who.clone());

			<TokenInfo<T>>::remove_prefix(who.clone());
			//删除掉相关的tokeninfo

			Self::deposit_event(RawEvent::KillRegisterEvent(who.clone()));

	}

	fn initialize_mutable_parameter(params: &[T::AccountId]){
		<PledgeAmount<T>>::put(T::PledgeAmount::get());
		<AddressDefaultStatus>::put(AddressStatus::default());
	}


}


