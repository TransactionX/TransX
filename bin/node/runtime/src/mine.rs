//! ## Genesis config
use frame_support::{debug,decl_storage, decl_module,decl_event, decl_error, StorageValue, StorageMap,Parameter, IterableStorageMap,
			   weights::{Weight},Blake2_256, ensure,dispatch::Vec,traits::Currency, StorageDoubleMap, IterableStorageDoubleMap};
use frame_support::traits::{Get, ReservableCurrency, OnUnbalanced, GetMembers, ReportedTxs, EnsureOrigin};
use frame_system as system;
use system::{ensure_signed, ensure_root};
use pallet_balances as balances;
use sp_std::convert::{TryInto,TryFrom, Into};
use sp_runtime::traits::{Hash, AtLeast32Bit, Bounded, One, Member, CheckedAdd, CheckedMul, CheckedDiv, Zero, AccountIdConversion, Saturating, CheckedConversion};
use sp_runtime::{Permill, ModuleId, DispatchResult, DispatchError, Percent};
use codec::{Encode, Decode};
use crate::mine_linked::{PersonMineWorkForce,PersonMine,MineParm,PersonMineRecord, MineTag};
use crate::register::{self,MinersCount,AllMiners, TokenInfo, AddressOf, Trait as RegisterTrait, AddressStatus};
use crate::mine_power::{PowerInfo, MinerPowerInfo, TokenPowerInfo, PowerInfoStore, MinerPowerInfoStore, TokenPowerInfoStore};
use node_primitives::{Count, USD, Balance};
use sp_std::{result, collections::btree_set::BTreeSet};
use num_traits::float::FloatCore;
use pallet_timestamp as timestamp;
use crate::constants::{symbol::{USDT, BTC, EOS, ETH, ECAP}, currency::*, genesis_params::*, time::*};

use crate::report::{self, VoteRewardPeriodEnum, BeingReportedTxsOf};
use crate::constants::{time::{MINUTES, DAYS, HOURS}, genesis_params::*};
use sp_std::prelude::*;

const MODULE_ID: ModuleId = ModuleId(*b"py/trsry");

// 算力相对于金额或是次数的倍数（为了让计算更加精确）
// 具体的算力数值大概也是金额与次数的Multiple倍
pub const Multiple: u64 = 1_0000;

// 第一年每天的奖励
const FirstYearPerDayMineRewardToken: Balance = 2100_0000*DOLLARS/2/(SubHalfDuration as u128)/36525*100;

// 4年减半
const SubHalfDuration: u64 = 4;

type PositiveImbalanceOf<T> = <<T as Trait>::Currency3 as Currency<<T as frame_system::Trait>::AccountId>>::PositiveImbalance;
type StdResult<T> = core::result::Result<T, &'static str>;
type BalanceOf<T> = <<T as Trait>::Currency3 as Currency<<T as system::Trait>::AccountId>>::Balance;
type BlockNumberOf<T> = <T as system::Trait>::BlockNumber;  // u32
type OwnerMineWorkForce<T> = PersonMineWorkForce<<T as system::Trait>::BlockNumber>;
type OwnerWorkForceItem<T> = PersonMine<OwnedDayWorkForce<T>, <T as system::Trait>::AccountId,<T as system::Trait>::BlockNumber>;
pub type OwnerMineRecordItem<T> = PersonMineRecord<<T as timestamp::Trait>::Moment,
	<T as system::Trait>::BlockNumber,BalanceOf<T>, <T as system::Trait>::AccountId>;
type PowerInfoItem<T> = PowerInfo<<T as system::Trait>::BlockNumber>;
type TokenPowerInfoItem<T> = TokenPowerInfo<<T as system::Trait>::BlockNumber>;
type MinerPowerInfoItem<T> = MinerPowerInfo<<T as system::Trait>::AccountId, <T as system::Trait>::BlockNumber>;
type PowerInfoStoreItem<T> = PowerInfoStore<PowerInfoList<T>, <T as system::Trait>::BlockNumber>;
type TokenPowerInfoStoreItem<T> = TokenPowerInfoStore<TokenPowerInfoList<T>, <T as system::Trait>::BlockNumber>;
type MinerPowerInfoStoreItem<T> = MinerPowerInfoStore<MinerPowerInfoDict<T>, <T as system::Trait>::AccountId, <T as system::Trait>::BlockNumber>;


#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum MLA {
	BtcAmount(u64),
	EthAmount(u64),
	UsdtAmount(u64),
	EosAmount(u64),
	EcapAmount(u64),

}


#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum LA {
	BtcAmount(u64),
	EthAmount(u64),
	UsdtAmount(u64),
	EosAmount(u64),
	EcapAmount(u64),

}


#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum TLA {
	BtcAmount(u64),
	EthAmount(u64),
	UsdtAmount(u64),
	EosAmount(u64),
	EcapAmount(u64),

}

#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum LC {
	BtcCount(u64),
	EthCount(u64),
	UsdtCount(u64),
	EosCount(u64),
	EcapCount(u64),

}

#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum TLC {
	BtcCount(u64),
	EthCount(u64),
	UsdtCount(u64),
	EosCount(u64),
	EcapCount(u64),

}


#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum MR {
	Btc(Permill),
	Eth(Permill),
	Usdt(Permill),
	Eos(Permill),
	Ecap(Permill),

}


// 继承 register 模块,方便调用register里面的 store
pub trait Trait: balances::Trait + RegisterTrait {

	type ReportedTxs: ReportedTxs<Self::AccountId>;

	type TechMmebersOrigin: GetMembers<Self::AccountId>;

	type ShouldAddOrigin: OnUnbalanced<PositiveImbalanceOf<Self>>;

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Currency3: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	type MineIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;

	type ArchiveDuration: Get<Self::BlockNumber>;

	type RemovePersonRecordDuration: Get<Self::BlockNumber>;

	// 上级和上上级的膨胀算力占比
	type SuperiorInflationRatio: Get<Permill>;

	type FatherInflationRatio: Get<Permill>;

	type AR: Get<Permill>;

	// 创始团队成员的分润占比（20% 写20； 25% 写25；以此类推）
	type FoundationShareRatio: Get<Permill>;

	// ***注意 下面的值不是占比 占比在相应方法中计算  如果矿工是100， 上级是50， 上上级是25， 那么
	// 矿工的分润比就是 100 / （100 + 50 + 25）
	// 矿工奖励部分
	type MinerSharePortion: Get<u32>;

	// 上级的奖励部分
	type FatherSharePortion: Get<u32>;

	// 上上级的奖励部分
	type SuperSharePortion: Get<u32>;

	// 客户端挖矿算力占比
	type ClientRatio: Get<Permill>;

	// 单日全网最低的奖励数量
	type PerDayMinReward: Get<BalanceOf<Self>>;

	// 默认金额
	type ZeroDayAmount: Get<u64>;

	// 默认次数
	type ZeroDayCount: Get<u64>;

	type DeclineExp: Get<u64>;

	type MineSetOrigin: EnsureOrigin<Self::Origin>;

}



decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
		<T as system::Trait>::BlockNumber,
    {
        Created(AccountId, Hash),
        Mined(AccountId,u64),  // 挖矿成功的事件
        PowerInfoArchived(BlockNumber),
        TokenPowerInfoArchived(BlockNumber),
        MinerPowerInfoArchived(BlockNumber),
        SetChangeableParam,
        StartMine,
        SetFounders,
        SetMaxMineCount,
        SetMLA,
        SetLA,
        SetLC,
        SetTLC,
        SetTLA,
        SetMR,

    }
);

decl_storage! {
    trait Store for Module<T: Trait> as MineStorage {

    	/// tx与minetype作为key对应的挖矿记录
    	pub OwnerMineRecord get(fn mine_record): double_map hasher(blake2_128_concat) Vec<u8>, hasher(blake2_128_concat) MineTag => Option<OwnerMineRecordItem<T>>;

        /// 设置状态位  u32表现形式1xxx.初始值为 1000,低3位分别表示 验证通过次数,验证失败次数,非正常情况返回次数, tx => 1xxx,AccountId, symbol
		pub TxVerifyMap get(fn tx_verify_map): map hasher(blake2_128_concat) (Vec<u8>,MineTag) => u64;

		/// 记录 TxVerifyMap 的长度
		pub LenOfTxVerify : u32;

    	/// 个人挖矿数据每天汇总
    	OwnedDayWorkForce get(fn person_workforce): map  hasher(blake2_128_concat) (T::AccountId,BlockNumberOf<T>) => Option<OwnerMineWorkForce<T>>;

    	/// 矿工每天挖矿次数
    	OwnedMineIndex: map hasher(blake2_128_concat) (T::AccountId,BlockNumberOf<T>) => u64;

		/// 全网挖矿总和数据
        PowerInfoList get(fn power_info): map hasher(blake2_128_concat) u32 => Option<PowerInfoItem<T>>;

        /// 有关币种的挖矿数据汇总
        TokenPowerInfoList get(fn token_power_info): map hasher(blake2_128_concat) u32 => Option<TokenPowerInfoItem<T>>;

		/// 个人挖矿的详细汇总(具体到每个币种)
        MinerPowerInfoDict get(fn miner_power_info): double_map  hasher(blake2_128_concat) u32, hasher(blake2_128_concat) T::AccountId => Option<MinerPowerInfoItem<T>>;

        // `MinerPowerInfoPrevPoint`用来区分存储前一天矿工算力信息的。
        // = 0，表示第一天挖矿，矿工还不存在前一日算力信息。
        // = 1，表示前一天挖矿信息保存在`MinerPowerInfoDict(1, AccountId)`中。
        // = 2，表示前一天挖矿信息保存在`MinerPowerInfoDict(2, AccountId)`中。
        /// 汇总数据的指针
        MinerPowerInfoPrevPoint: u32;

		/// 矿工挖矿的日期记录
		MinerDays get(fn minertxdays): map hasher(blake2_128_concat) T::AccountId => Vec<T::BlockNumber>;

		/// 个人挖矿的所有tx
		MinerAllDaysTx get(fn mineralldaystx): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::BlockNumber => Vec<Vec<u8>>;

		/// 币种btc的最大算力占比
		MRbtc get(fn btc_max_portion): Permill = Permill::from_percent(70);
		/// 币种eth的最大算力占比
		MReth get(fn eth_max_portion): Permill = Permill::from_percent(10);
		/// 币种eos的最大算力占比
		MReos get(fn eos_max_portion): Permill = Permill::from_percent(8);
		/// 币种usdt的最大算力占比
		MRusdt get(fn usdt_max_portion): Permill = Permill::from_percent(50);
		/// 币种ecap的最大算力占比
		MRecap get(fn ecap_max_portion): Permill = Permill::from_percent(50);

		/// 矿工的奖励部分
		MinerSharePortion get(fn miner_share_portion): u32 = 100;
		/// 上级的奖励部分
		FatherSharePortion get(fn father_share_portion): u32 = 50;
		/// 上上级的奖励部分
		SuperSharePortion get(fn super_share_portion): u32 = 25;

		/// 币种eth的次数算力硬顶
		TLCeth get(fn eth_limit_count): Count = 10000 * Multiple;
		/// 币种btc的次数算力硬顶
		TLCbtc get(fn btc_limit_count): Count = 10000 * Multiple;
		/// 币种usdt的次数算力硬顶
		TLCusdt get(fn usdt_limit_count): Count = 10000 * Multiple;
		/// 币种ecap的次数算力硬顶
		TLCecap get(fn ecap_limit_count): Count = 10000 * Multiple;
		/// 币种eos的次数算力硬顶
		TLCeos get(fn eos_limit_count): Count = 10000 * Multiple;

		/// 币种btc的金额算力硬顶
		TLAbtc get(fn btc_limit_amount): USD = 10000 * INIT_AMOUNT_POWER * USDT_DECIMALS * Multiple;
		/// 币种eth的金额算力硬顶
		TLAeth get(fn eth_limit_amount): USD = 10000 * INIT_AMOUNT_POWER * USDT_DECIMALS * Multiple;
		/// 币种eos的金额算力硬顶
		TLAeos get(fn eos_limit_amount): USD = 10000 * INIT_AMOUNT_POWER * USDT_DECIMALS * Multiple;
		/// 币种usdt的金额算力硬顶
		TLAusdt get(fn usdt_limit_amount): USD = 10000 * INIT_AMOUNT_POWER * USDT_DECIMALS * Multiple;
		/// 币种ecap的金额算力硬顶
		TLAecap get(fn ecap_limit_amount): USD = 10000 * INIT_AMOUNT_POWER * USDT_DECIMALS * Multiple;

		/// btc单次转账的金额硬顶
		MLAbtc get(fn mla_btc): USD = 10_0000 * USDT_DECIMALS * Multiple;
		/// usdt单次转账的金额硬顶
		MLAusdt get(fn mla_usdt): USD = 5000 * USDT_DECIMALS * Multiple;
		/// eos单次转账的金额硬顶
		MLAeos get(fn mla_eos): USD = 1_0000 * USDT_DECIMALS * Multiple;
		/// eth单次转账的金额硬顶
		MLAeth get(fn mla_eth): USD = 4_0000 * USDT_DECIMALS * Multiple;
		/// ecap单次转账的金额硬顶
		MLAecap get(fn mla_ecap): USD = 5000 * 2 * USDT_DECIMALS * Multiple;

		/// 本周期的奖励总金额
		ThisArchiveDurationTotalReward get(fn this_duration_reward): BalanceOf<T>;

		/// 目前为止全网挖矿奖励总金额
		HistoryTotalReward get(fn history_total_reward): BalanceOf<T>;

		/// 历史所有周期以及对应的总奖励
		HistorySpecificReward get(fn history_specific_reward): Vec<(u32, BalanceOf<T>)>;

		/// 个人当天挖矿次数硬顶
		MiningMaxNum get(fn mining_max_num): u64 = 10000;

		/// 个人btc当天挖矿金额算力硬顶
		LAbtc get(fn la_btc): u64 = 100_0000_00000 * Multiple;
		/// 个人eth当天挖矿金额算力硬顶
		LAeth get(fn la_eth): u64 = 100_0000_00000 * Multiple;
		/// 个人usdt当天挖矿金额算力硬顶
		LAusdt get(fn la_usdt): u64 = 100_0000_00000 * Multiple;
		/// 个人eos当天挖矿金额算力硬顶
		LAeos get(fn la_eos): u64 = 100_0000_00000 * Multiple;
		/// 个人ecap当天挖矿金额算力硬顶
		LAecap get(fn la_ecap): u64 = 100_0000_00000 * Multiple;

		/// 个人btc当天挖矿次数算力硬顶
		LCbtc get(fn lc_btc): u64 = 100_0000 * Multiple;
		/// 个人eth当天挖矿次数算力硬顶
		LCeth get(fn lc_eth): u64 = 100_0000 * Multiple;
		/// 个人usdt当天挖矿次数算力硬顶
		LCusdt get(fn lc_usdt): u64 = 100_0000 * Multiple;
		/// 个人eos当天挖矿次数算力硬顶
		LCeos get(fn lc_eos): u64 = 100_0000 * Multiple;
		/// 个人ecap当天挖矿次数算力硬顶
		LCecap get(fn lc_ecap): u64 = 100_0000 * Multiple;

		/// 创始人
		Founders get(fn founders): Vec<T::AccountId>;

		/// 上个周期挖矿的所有矿工
		LastTimeMiners get(fn last_time_miners): BTreeSet<T::AccountId>;

		/// 上次挖矿的金额算力与参与挖矿的矿工数（不为0的那次）
		LastTotolAmountPowerAndMinersCount get(fn last_total_amount_power_and_miners_count): (u64, u64) = (INIT_AMOUNT_POWER * USDT_DECIMALS * INIT_COUNT_POWER * Multiple * INIT_MINER_COUNT, INIT_MINER_COUNT);

		/// 上次挖矿的次数算力与参与挖矿的矿工数（不为0的那次）
		LastTotolCountPowerAndMinersCount get(fn last_total_count_power_and_miners_count): (u64, u64) = (1 * INIT_COUNT_POWER * Multiple * INIT_MINER_COUNT, INIT_MINER_COUNT);

		/// 个人挖矿奖励记录  (历史总金额， 最近一次的金额， 最后一次的时间)
		CommissionAmount get(fn commission_amount): map hasher(blake2_128_concat) T::AccountId => (BalanceOf<T>, BalanceOf<T>, T::Moment);

		// todo 测试专用
		ThisTimeReward get( fn this_time_reward): BalanceOf<T>;
		ThisDayReward get(fn this_day_reward): BalanceOf<T>;

		//本次算力 平均算力 倍数
		FinalCalculateExceptTag: Vec<(u64, u64, u64, u64)>;

		/// todo 计算算力占比时的数据  这时候的算力不做挖矿类型计算

		Ratio get(fn ratio): (u64, u64,  u64, u64);

		SetText get(fn set_text): BTreeSet<T::AccountId>;

		MinerCount get(fn miner_count): u64;
		PowerTest get(fn power_test): (u64, u64, u64, u64);
		MineReward get(fn mine_reward): (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>);

	}

	add_extra_genesis {

			config(founders): Vec<T::AccountId>;
			build(|config| {
				// 初始化创始人
				<Module<T>>::initialize_founders(&config.founders);

				})
			}
    }

decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {

		///除以0
		DivZero,

		/// 本次算力相对平均算力过大
		PowerTooLarge,

		/// 前一天挖矿初始化错误
		PrePowerErr,

		/// 不是注册过的
		NotRegister,

		/// 不存在这个参数
		NotExistsParam,

		/// 范围错误
		BoundError,

		/// 金额输入错误
		AmountError,

		/// 不是本人的挖矿地址
		NotYourTokenAddress,

		/// 挖矿地址没有被激活
		InActiveAddress,

		/// 金额太少
		BondTooLow,

		/// 自己的交易（自己转账给自己
		TransferToYourself,

		/// 数目溢出
		Overfolw,

		/// 这个tx正在被使用（已经进入挖矿）
		InUseTx,

		/// 超过自己的配额比例
		MoreThanProportion,

		/// 挖矿次数过多
		MineCountTooMore,

		/// 金额或是算力达到最大
		AmountOrCountToMax,

		/// 未知的币种
		UnknownSymbol,

		/// MLA 设置太大
		MLAError,

		/// txsoverlimit
		OverMaximum,

		/// usdt金额太小
		AmountTooLow,

		/// tx和挖矿类型被占用
		InUsingTxAndMinetype,

		/// 正在举报队列中
		BeingReported,

		/// 未知类型挖矿
		UnknownMineType,

		/// 输入了空值
		EmptyParam,

	}
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		/// 多久归档算力一次
    	const ArchiveDuration: T::BlockNumber = T::ArchiveDuration::get();

    	/// 最多保留多长时间的数据
    	const RemovePersonRecordDuration: T::BlockNumber = T::RemovePersonRecordDuration::get();

    	/// 金额算力的奖励占比
    	const AR: Permill = T::AR::get();

    	/// 客户端挖矿的奖励占比
    	const ClientRatio: Permill = T::ClientRatio::get();

    	/// 单日全网最低的奖励金额
		const PerDayMinReward: BalanceOf<T> = T::PerDayMinReward::get();

    	/// 钝化指数
    	const DeclineExp: u64 = T::DeclineExp::get();

    	/// 有上上级的算力膨胀比例
    	const SuperiorInflationRatio: Permill = T::SuperiorInflationRatio::get();

    	/// 有上级的膨胀比例
    	const FatherInflationRatio: Permill = T::FatherInflationRatio::get();

    	/// 创始团队成员的分润比例
    	const FoundationShareRatio: Permill = T::FoundationShareRatio::get();

    	/// 每个人的初始化金额算力
    	const ZeroDayAmount: u64 = T::ZeroDayAmount::get();

    	/// 每个人的初始化次数算力
    	const ZeroDayCount: u64 = T::ZeroDayCount::get();

    	type Error = Error<T>;
        fn deposit_event() = default;


        /// 设置创始人 (原则上给一个账号就可以，用于接收挖矿奖励)
		#[weight = 50_000]
		fn set_founders(origin, who: Vec<T::AccountId>) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;
			// 输入的值不能为空
			ensure!(who.len() != 0, Error::<T>::EmptyParam);
			<Founders<T>>::put(who);
			Self::deposit_event(RawEvent::SetFounders);
			Ok(())

		}


		/// 设置个人最大的挖矿次数
		#[weight = 50_000]
		fn set_max_mine_count(origin, count: u64) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;

			<MiningMaxNum>::put(count);
			Self::deposit_event(RawEvent::SetMaxMineCount);
			Ok(())

		}


		/// 设置单次转账金额硬顶
		#[weight = 50_000]
		fn set_mla(origin, amount: MLA) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;

			match amount {
				MLA::BtcAmount(x) => <MLAbtc>::put(x),
				MLA::UsdtAmount(x) => <MLAusdt>::put(x),
				MLA::EosAmount(x) => <MLAeos>::put(x),
				MLA::EthAmount(x) => <MLAeth>::put(x),
				MLA::EcapAmount(x) => <MLAecap>::put(x),
				_ => return Err(Error::<T>::UnknownSymbol)?,
			}
			Self::deposit_event(RawEvent::SetMLA);
			Ok(())
		}


		/// 设置个人当天的金额算力硬顶
		#[weight = 50_000]
		fn set_la(origin, amount: LA) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;

			match amount {
				LA::BtcAmount(x) => <LAbtc>::put(x),
				LA::UsdtAmount(x) => <LAusdt>::put(x),
				LA::EosAmount(x) => <LAeos>::put(x),
				LA::EthAmount(x) => <LAeth>::put(x),
				LA::EcapAmount(x) => <LAecap>::put(x),
				_ => return Err(Error::<T>::UnknownSymbol)?,
			}
			Self::deposit_event(RawEvent::SetLA);
			Ok(())

		}


		/// 设置个人当天的次数算力硬顶
		#[weight = 50_000]
		fn set_lc(origin, count: LC) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;
			match count {
				LC::BtcCount(x) => <LCbtc>::put(x),
				LC::EthCount(x) => <LCeth>::put(x),
				LC::UsdtCount(x) => <LCusdt>::put(x),
				LC::EosCount(x) => <LCeos>::put(x),
				LC::EcapCount(x) => <LCecap>::put(x),
				_ => return Err(Error::<T>::UnknownSymbol)?,

			}
			Self::deposit_event(RawEvent::SetLC);
			Ok(())
		}


		/// 设置某个币种的次数算力硬顶(不针对个人,针对币种)
		#[weight = 50_000]
		fn set_tlc(origin, count: TLC) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;
			match count {
				TLC::BtcCount(x) => <TLCbtc>::put(x),
				TLC::EthCount(x) => <TLCeth>::put(x),
				TLC::UsdtCount(x) => <TLCusdt>::put(x),
				TLC::EosCount(x) => <TLCeos>::put(x),
				TLC::EcapCount(x) => <TLCecap>::put(x),
				_ => return Err(Error::<T>::UnknownSymbol)?,

			}
			Self::deposit_event(RawEvent::SetTLC);
			Ok(())
		}


		/// 设置某个币种的金额算力硬顶(不针对个人,针对币种)
		#[weight = 50_000]
		fn set_tla(origin, amount: TLA) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;
			match amount {
				TLA::BtcAmount(x) => <TLAbtc>::put(x),
				TLA::UsdtAmount(x) => <TLAusdt>::put(x),
				TLA::EosAmount(x) => <TLAeos>::put(x),
				TLA::EthAmount(x) => <TLAeth>::put(x),
				TLA::EcapAmount(x) => <TLAecap>::put(x),
				_ => return Err(Error::<T>::UnknownSymbol)?,
			}
			Self::deposit_event(RawEvent::SetTLA);
			Ok(())

		}


		/// 设置币种的最大算力占比
		#[weight = 50_000]
		fn set_mr(origin, percent: MR) -> DispatchResult {
			T::MineSetOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;
			match percent {
				MR::Btc(x) => <MRbtc>::put(x),
				MR::Eth(x) => <MReth>::put(x),
				MR::Eos(x) => <MReos>::put(x),
				MR::Usdt(x) => <MRusdt>::put(x),
				MR::Ecap(x) => <MRecap>::put(x),
				_ => return Err(Error::<T>::UnknownSymbol)?,
			}
			Self::deposit_event(RawEvent::SetMR);
			Ok(())
		}


		/// 挖矿
		#[weight = 50_000]
        pub fn create_mine(
        origin,mine_tag: MineTag, tx: Vec<u8>, from_address: Vec<u8>,to_address:Vec<u8>,symbol:Vec<u8>,
        amount: Vec<u8>,protocol:Vec<u8>,decimal:u32,usdt_nums:u64,blockchain:Vec<u8>,memo:Vec<u8>)
        ->  DispatchResult {
        	/// 创建挖矿
			{debug::info!("开始挖矿!")}
        	let sender = ensure_signed(origin)?;

        	// 如果自己在举报队列 则不能挖矿
        	ensure!(!(T::ReportedTxs::is_reported(sender.clone())),  Error::<T>::BeingReported);

        	{debug::info!("通过签名！")}
        	let address = from_address;
        	ensure!(address.clone() != to_address.clone(),Error::<T>::TransferToYourself);
        	ensure!(usdt_nums <= u64::max_value(),Error::<T>::Overfolw);  // 这个不可能溢出的
        	ensure!(usdt_nums >= 5 * USDT_DECIMALS, Error::<T>::AmountTooLow);  // 前端需要乘于100
        	ensure!(!<TxVerifyMap>::contains_key(&(tx.clone(),mine_tag.clone())), Error::<T>::InUsingTxAndMinetype);
			{debug::info!("挖矿金额足够！")}

        	// 删除过期的交易tx（为了减轻存储负担）
			Self::remove_expire_record(sender.clone(), false);

        	ensure!(<AllMiners<T>>::contains_key(sender.clone()), Error::<T>::NotRegister);

        	ensure!(Self::check_amount(amount.clone()), Error::<T>::AmountError);

        	let mut is_from_exists = false;
        	let mut is_to_exists = false;
        	let mut is_active = false;

        	for i in <AddressOf<T>>::get(&sender).iter(){

        		// 寻找已经激活的地址
        		if i.1 == AddressStatus::active{
        				is_active = true;
        				if to_address == i.0 {
							is_to_exists = true;
							break;
						}
						if address == i.0 {
							is_from_exists = true;
							break;
						}
        			}

        	}

        	// 如果地址存在并且被激活
        	// 挖矿是客户端 绑定to地址； 挖矿是钱包，绑定from地址
        	if is_active && (is_from_exists || is_to_exists){
        		// 挖矿是客户端 绑定to地址； 挖矿是钱包，绑定from地址
        		match mine_tag {
        		MineTag::CLIENT => {
        			ensure!(is_to_exists, Error::<T>::NotYourTokenAddress);
        			},

        		MineTag::WALLET => {
        			ensure!(is_from_exists, Error::<T>::NotYourTokenAddress);
        		},

        		_ => {return Err(Error::<T>::UnknownMineType)?;},

        		}

        	}

        	else{
        		return Err(Error::<T>::InActiveAddress)?;
        	}

			// 不存在  说明还没有挖矿
			ensure!(!<OwnerMineRecord<T>>::contains_key(tx.clone(), mine_tag.clone()), Error::<T>::InUseTx);

			// 如果该币的全网挖矿算力大于一定的配额比例  则不再挖矿
			ensure!(!Self::is_token_power_more_than_portion(symbol.clone())?, Error::<T>::MoreThanProportion);

			// 如果该币种的个人挖矿算力大于个人挖矿硬顶 则不能再挖
			ensure!(!Self::is_person_power_to_max(sender.clone(), symbol.clone())?, Error::<T>::AmountOrCountToMax);

			// 如果该币种的币种挖矿算力达到硬顶 则不能再挖
			ensure!(!Self::is_token_power_to_max(symbol.clone())?, Error::<T>::AmountOrCountToMax);

			 // 保证不能超过队列最大长度
            ensure!(LenOfTxVerify::get() <= T::TxsMaxCount::get(), Error::<T>::OverMaximum);

            {debug::info!("初步验证通过， 可以进行挖矿！")}

        	let block_num = Self::now();

			let action = "transfer".as_bytes().to_vec();   // 固定 为 transfer

			let mine_tag_cp = mine_tag.clone();  // 数据处理而以

			let mine_tag_1 = match mine_tag.clone(){
				MineTag::WALLET => MineTag::CLIENT,
				MineTag::CLIENT => MineTag::WALLET,
			};

			let mut mine_count = 1u16;

			// 如果另外一种挖矿类型已经存在 那么是第二次挖矿
			if <OwnerMineRecord<T>>::contains_key(tx.clone(), mine_tag_1.clone()){
				mine_count += 1;
			}

			let mine_parm = MineParm{
						mine_tag,
						mine_count,
						action,  // action是字符串  这里先定义下来（substrate是Vec<u8>)
						tx: tx.clone(),
						address,
						to_address,
						symbol,
						amount,
						protocol,
						decimal,
						usdt_nums,
						blockchain,
						memo
				};
			debug::info!("进入挖矿方法");
			Self::deposit_event(RawEvent::StartMine);
			Self::mining(mine_parm,sender, mine_tag_cp)?;
			Ok(())
        }


		fn on_finalize(block_number: T::BlockNumber) {

            if (block_number % T::ArchiveDuration::get()).is_zero() {
                Self::archive(block_number);
            }
        }
    }
}


impl<T: Trait> Module<T> {

	fn check_amount(amount: Vec<u8>) -> bool{
		let mut all = true;
		let len = amount.len() as u32;
		if len > 36{
			return false;
		}
		for i in amount.iter(){
			let num = u128::from_le((*i).into());
			if num < 48u128 || num > 57u128{
				all = false;
				break
			}
		}
		all
	}


	fn mining(mut mine_parm:MineParm,sender: T::AccountId, mine_tag: MineTag)-> DispatchResult{
		ensure!(<AllMiners<T>>::contains_key(sender.clone()), Error::<T>::NotRegister);

		let (btc, eth, usdt, eos, ecap) = Self::symbols_to_vec();

		let symbol = match mine_parm.symbol.clone() {
			_ if btc.clone() == mine_parm.symbol.clone()  => BTC,
			_ if eth.clone() == mine_parm.symbol.clone() =>  ETH,
			_ if usdt.clone() == mine_parm.symbol.clone() => USDT,
			_ if eos.clone() == mine_parm.symbol.clone() => EOS,
			_ if ecap.clone() == mine_parm.symbol.clone() => ECAP,
			_ => return Err(Error::<T>::UnknownSymbol)?
		};

		// 获取日期
		let block_num = Self::now();

		// 归档周期不能是0
		let now_day = block_num.checked_div(&T::ArchiveDuration::get()).ok_or(Error::<T>::DivZero)?;

		let owned_mineindex = <OwnedMineIndex<T>>::get(&(sender.clone(),now_day));

		// 挖矿次数过多
		if owned_mineindex >= <MiningMaxNum>::get(){
			return Err(Error::<T>::MineCountTooMore)?;
		}

		let now_time = Self::time();   // 记录到秒

		// ***以下跟算力相关***

		// 计算算力
		let workforce = Self::calculate_workforce(
			sender.clone(), block_num, symbol.clone(), mine_parm.usdt_nums.clone(), mine_tag.clone())?;

		// 获取金额算力
		let mut amount_workforce = workforce.0;
		// 获取次数算力
		let mut count_workforce = workforce.1;

		// 获取昨天的总金额算力
		let prev_total_amount = workforce.2;
		// 获取昨天的总次数算力
		let prev_total_count = workforce.3;

		// 计算总算力占比（这个占比是乘于精度过的）
		//  结果： 真实算力占比*100亿
		let workforce_ratio = Self::calculate_workforce_ratio(
			amount_workforce.clone(), count_workforce.clone(), prev_total_amount.clone(), prev_total_count.clone())?;

		let decimal = match <BalanceOf<T> as TryFrom::<u64>>::try_from(100_0000_0000u64).ok(){
			Some(x) => x,
			// 不会返回错误  这里不作处理
			_ => return Err(Error::<T>::Overfolw)?
		};

		// 把算力占比变成balance类型
		let workforce_ratio_change_into_balance = match <BalanceOf<T> as TryFrom::<u64>>::try_from(workforce_ratio).ok(){
			Some(b) => b,
			None => return Err(Error::<T>::MineCountTooMore)?,
		};

		let today_reward = Self::per_day_mine_reward_token()?;

		// 计算这一次的总挖矿奖励
		let thistime_reward = today_reward * workforce_ratio_change_into_balance/decimal;

        // 计算每一个人的奖励
		let per_one_reward = Self::calculate_reward(sender.clone(), thistime_reward)?;

		// 奖励所有人
		Self::reward_all_people(sender.clone(), per_one_reward.0, per_one_reward.1, per_one_reward.2, per_one_reward.3)?;

		// 全网算力存储
		<PowerInfoStoreItem<T>>::add_power(
			workforce_ratio.clone(), 1u64,
			count_workforce.clone(), mine_parm.usdt_nums.clone(),
		amount_workforce.clone(), block_num.clone());

		// 全网token信息存储
		<TokenPowerInfoStoreItem<T>>::add_token_power(
			symbol.clone(), workforce_ratio, 1u64,
			count_workforce, mine_parm.usdt_nums.clone(),
			amount_workforce, block_num);

		// 矿工个人算力存储
		let curr_point = Self::miner_power_info_point().1;
		<MinerPowerInfoStoreItem<T>>::add_miner_power(
			&sender, curr_point.clone(), symbol.clone(),
			workforce_ratio, 1u64, count_workforce,
			mine_parm.usdt_nums.clone(), amount_workforce, block_num);

		// 处理并存储与本次有关的tx
		Self::add_tx(mine_parm.clone(), block_num.clone(), sender.clone());

		// 添加挖矿信息  交易验证模块用得到
		let person_mine_record = PersonMineRecord::new(
			&mine_parm, sender.clone(), now_time, block_num,
			per_one_reward.0, per_one_reward.1, per_one_reward.2,
			count_workforce, amount_workforce )?;

		<OwnerMineRecord<T>>::insert(&mine_parm.tx, mine_parm.mine_tag.clone(), person_mine_record);

		<OwnerWorkForceItem<T>>::add(amount_workforce.clone(), count_workforce.clone(), &sender,mine_parm.usdt_nums,now_day,block_num)?;
		// 将用户的挖矿记录+1
		let new_owned_mineindex = owned_mineindex.checked_add(1).ok_or(Error::<T>::Overfolw)?;
		<OwnedMineIndex<T>>::insert(&(sender.clone(),now_day), new_owned_mineindex);
		#[cfg(feature = "std")]{
			println!("-----------OwnedMineIndex:{:?}------------",new_owned_mineindex);
		}
		// tx 验证初始化
		<TxVerifyMap>::insert(&(mine_parm.tx,mine_tag),1000);
		LenOfTxVerify::mutate(|n|*n += 1);

		// 奖励统计
		<ThisArchiveDurationTotalReward<T>>::mutate(|a| *a += thistime_reward.clone());

		<HistoryTotalReward<T>>::mutate( |a| *a += thistime_reward.clone());

		// 把矿工添加到集合中
		<LastTimeMiners<T>>::mutate(|h| h.insert(sender.clone()));

		Self::deposit_event(RawEvent::Mined(sender, new_owned_mineindex));

		Ok(())
	}


    /// 获取存储矿工算力信息的指示
    fn miner_power_info_point() -> (u32, u32) {
        let prev_point = <MinerPowerInfoPrevPoint>::get();
        let curr_point = match prev_point {
            1 => 2,
            2 => 1,
            _ => 0,
        };
        (prev_point, curr_point)
    }


	/// 将当日挖矿信息进行归档，不可更改地存储在网络中。
	fn archive(block_number: T::BlockNumber) {

		// 添加历史挖矿奖励信息
		let days = <HistorySpecificReward<T>>::get().len() as u32;
		<HistorySpecificReward<T>>::mutate(|h| h.push((days, <ThisArchiveDurationTotalReward<T>>::get())));
		// 清掉本周期奖励统计
		<ThisArchiveDurationTotalReward<T>>::kill();

		// 对算力信息和Token算力信息进行归档
		<PowerInfoStoreItem<T>>::archive(block_number.clone()).unwrap();
		Self::deposit_event(RawEvent::PowerInfoArchived(block_number.clone()));

		// 初始化昨天的挖矿算力
		Self::init_yesterday_total_power(block_number);

		<TokenPowerInfoStoreItem<T>>::archive(block_number.clone()).unwrap();
		Self::deposit_event(RawEvent::TokenPowerInfoArchived(block_number.clone()));

		// 对矿工的挖矿信息进行归档
		let (prev_point, curr_point) = Self::miner_power_info_point();
		if curr_point == 0 {
			// 当日和昨日的矿工算力信息均不存在，无需归档
			<MinerPowerInfoPrevPoint>::put(1u32);
			return;
		}

		// 删除前一日的矿工算力数据，并将今日的算力作为前一日的矿工算力。
		<MinerPowerInfoStoreItem<T>>::archive(prev_point, block_number.clone());
		<MinerPowerInfoPrevPoint>::put(curr_point);
		Self::deposit_event(RawEvent::MinerPowerInfoArchived(block_number.clone()));

	}


	fn calculate_workforce(
		who: T::AccountId, block_number: T::BlockNumber, coin_name: &'static str, mut usdt_nums: u64, mine_tag: MineTag)
		-> result::Result<(u64, u64, u64, u64), DispatchError> {
		/// 计算次数或是金额算力  coin_amount指本次交易以USDT计价的金额

		let miner_id = &who;
		// 获取指定编号的矿工算力信息
        let (prev_point, curr_point) = Self::miner_power_info_point();

		let now_token_power_info = <TokenPowerInfoStoreItem<T>>::get_curr_token_power(block_number);

		// 获取昨天的总金额算力
		let prev_total_amount = match <LastTotolAmountPowerAndMinersCount>::get().0 {
					0u64 => T::ZeroDayAmount::get() * INIT_MINER_COUNT,
					n @ _ => n,
			};
		// 获取昨天的总次数算力
		let prev_total_count = match <LastTotolCountPowerAndMinersCount>::get().0{
					0u64 => T::ZeroDayCount::get() * INIT_MINER_COUNT,
					n @ _ => n,
			};

		// 获取矿工当日算力信息
		let token_power_of = <MinerPowerInfoStoreItem<T>>::get_miner_power_info(
			curr_point, miner_id, block_number.clone());

		// 该矿工今天金额算力
		let today_total_amount = token_power_of.amount_power;
		// 该矿工今天次数算力
		let today_total_count = token_power_of.count_power;

		let mut MLA = 0u64;

		match coin_name {

			BTC => {

				{debug::info!("比特币挖矿")}
				MLA = <MLAbtc>::get();
			},

			ETH => {

				{debug::info!("以太坊挖矿")}

				MLA = <MLAeth>::get();
			},

			USDT => {

				{debug::info!("USDT挖矿")}

				MLA = <MLAusdt>::get();
			},

			EOS => {

				{debug::info!("柚子挖矿")}
				MLA = <MLAeos>::get();

			},

			ECAP => {
				{debug::info!("ECAP挖矿")}

				MLA = <MLAecap>::get();
				// 算力是usdt的两倍
				usdt_nums *= 2;
			}


			_ =>  {
				 return Err(Error::<T>::UnknownSymbol)?;
			}

		}

		let amount_work_power = Self::final_work_power(who.clone(), MLA, mine_tag.clone(), prev_total_amount, usdt_nums, today_total_amount,true)?;
		let count_work_power = Self::final_work_power(who.clone(), 0u64, mine_tag.clone(), prev_total_count, 1u64, today_total_count, false)?;

		Ok((amount_work_power, count_work_power, prev_total_amount, prev_total_count))
	}


	/// 计算总算力占比(这里也附带了一个算力检查功能: 如果本次挖的算力过大, 是有可能不给挖矿的)
	fn calculate_workforce_ratio(
		amount_workforce: u64, count_workforce: u64, pre_amount_workfore: u64, pre_count_workforce: u64)
		-> result::Result<u64, DispatchError>{

		// 检查分母是否为0
		if pre_amount_workfore == 0u64 || pre_count_workforce == 0u64{
			return Err(Error::<T>::DivZero)?;
		}

		let a_sr = T::AR::get() ;  // 金额奖励占比
		let c_sr= Permill::from_percent(100).saturating_sub(a_sr);  // 次数奖励占比

		let decimal = 100_0000_0000u64;

		// 这个地方的计算是有可能溢出的
		let workforce_ratio =   (a_sr * amount_workforce.checked_mul(decimal).ok_or(Error::<T>::Overfolw)?  / pre_amount_workfore).checked_add(
		(c_sr * count_workforce.checked_mul(decimal).ok_or(Error::<T>::Overfolw)? / pre_count_workforce))
		.ok_or(Error::<T>::Overfolw)?;

		<Ratio>::put((a_sr * amount_workforce, pre_amount_workfore,  c_sr * count_workforce, pre_count_workforce));

		Ok(workforce_ratio)
	}


	/// 删除过期记录
	fn remove_expire_record(who: T::AccountId, is_remove_all: bool) {

		let block_num = Self::now(); // 获取区块的高度
		let now = block_num / T::ArchiveDuration::get();

		if <MinerDays<T>>::contains_key(&who) {
			let all_days = <MinerDays<T>>::get(&who);
			if !all_days.is_empty() {
				// 如果是删除全部（提供给外部模块， 这个模块不使用）
				if is_remove_all{
					for day in all_days.iter() {
						Self::remove_per_day_record(day.clone(), who.clone());
						}
				}
					// 正常删除
				else{
					for day in all_days.iter() {
						if now.clone() - day.clone() >= T::RemovePersonRecordDuration::get(){
						Self::remove_per_day_record(day.clone(), who.clone());
					}
					}
				}
			}
		}
	}


	/// 删除被选中的那天的记录
	fn remove_per_day_record(day: T::BlockNumber, who: T::AccountId) {

		let mut all_days = <MinerDays<T>>::get(&who);
		let all_tx = <MinerAllDaysTx<T>>::get(who.clone(), day.clone());
		//如果当天交易存在 那么就删除掉
		if !all_tx.is_empty() {
			for tx in all_tx.iter() {
				<OwnerMineRecord<T>>::remove_prefix(tx.clone());  // tx不能直接用remove方法来删除？？？？？？？？
			}
		}

		<MinerAllDaysTx<T>>::remove(who.clone(), day.clone());

		if let Some(pos) = all_days.iter().position(|a| a == &day) {
			all_days.swap_remove(pos);

			// 更新本人的未删除记录
			<MinerDays<T>>::insert(who.clone(), all_days.clone())
		}
	}


	/// 判断该token在全网算力是否超额 这里主要是金额算力  次数算力可以忽略不计
	fn is_token_power_more_than_portion(symbol: Vec<u8>) -> result::Result<bool, DispatchError>{// 参数要小写

		let mut is_too_large: bool = false;
		let mut max_portion: Permill = Permill::from_percent(0);
		let block_num = Self::now();

		// 用当前24小时内的信息（这里不是一个窗口函数，是会有一点问题的）
		let now_tokenpower_info =  <TokenPowerInfoStoreItem<T>>::get_curr_token_power(block_num.clone());

		// 获取昨天的总算力
		let power_info = <PowerInfoStoreItem<T>>::get_prev_power(block_num.clone());

		let mut all_token_power_total = power_info.total_power;

		// 总算力不一定是昨日的那个数据  因为昨日算力有可能是0
		let total =	<LastTotolAmountPowerAndMinersCount>::get().0.saturating_add(<LastTotolCountPowerAndMinersCount>::get().0);

		if power_info.total_power < total {
			all_token_power_total = total;
		}

		let (btc, eth, usdt, eos, ecap) = Self::symbols_to_vec();

		match symbol.clone() {
			_ if symbol.clone() == btc => {
				if now_tokenpower_info.btc_total_power  >= <MRbtc>::get() * all_token_power_total{
					is_too_large = true;
			}
			},

			_ if symbol.clone() == eth => {
				if now_tokenpower_info.eth_total_power  >= <MReth>::get() * all_token_power_total{
					is_too_large = true;
			}
			},

			_ if symbol.clone() == usdt => {
				if now_tokenpower_info.usdt_total_power  >= <MRusdt>::get() * all_token_power_total{
					is_too_large = true;
			}
			},

			_ if symbol.clone() == eos => {
				if now_tokenpower_info.eos_total_power  >= <MReos>::get() * all_token_power_total{
					is_too_large = true;
			}
			},

			_ if symbol.clone() == ecap => {
				if now_tokenpower_info.ecap_total_power  >= <MRecap>::get() * all_token_power_total{
					is_too_large = true;
			}
			},

			_ => return Err(Error::<T>::UnknownSymbol)?

		}

		Ok(is_too_large)

		}


	/// 把字符串的币种改成vec
	fn symbols_to_vec() -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>){
		//**********支持挖矿的币种********************
		let btc = BTC.as_bytes().to_vec();
		let eth = ETH.as_bytes().to_vec();
		let usdt = USDT.as_bytes().to_vec();
		let eos = EOS.as_bytes().to_vec();
		let ecap = ECAP.as_bytes().to_vec();
		//*************在这里添加**********************
		(btc, eth, usdt, eos, ecap)
	}


	/// 个人算力是否达到硬顶
	fn is_person_power_to_max(who: T::AccountId, symbol: Vec<u8>) -> result::Result<bool, DispatchError>{

		let block_number = Self::now();

		let (_, curr_point) = Self::miner_power_info_point();

		// 获取个人当天的累计算力
		let power_info = <MinerPowerInfoStoreItem<T>>::get_miner_power_info(
			curr_point, &who, block_number.clone());

		let (btc, eth, usdt, eos, ecap) = Self::symbols_to_vec();

		match symbol.clone() {
			_ if btc == symbol.clone()  => {
				ensure!(<LAbtc>::get() > power_info.amount_power && <LCbtc>::get() > power_info.count_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if eth == symbol.clone() =>  {
				ensure!(<LAeth>::get() > power_info.amount_power && <LCeth>::get() > power_info.count_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if usdt == symbol.clone() => {
				ensure!(<LAusdt>::get() > power_info.amount_power && <LCusdt>::get() > power_info.count_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if eos == symbol.clone() => {
				ensure!(<LAeos>::get() > power_info.amount_power && <LCeos>::get() > power_info.count_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if ecap == symbol.clone() => {
				ensure!(<LAecap>::get() > power_info.amount_power && <LCecap>::get() > power_info.count_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ => return Err(Error::<T>::UnknownSymbol)?
		}

		Ok(false)
	}


	/// 币种算力是否达到硬顶
	fn is_token_power_to_max(symbol: Vec<u8>) -> result::Result<bool, DispatchError>{

		let block_number = Self::now();

		let (btc, eth, usdt, eos, ecap) = Self::symbols_to_vec();

		let now_token_power_info = <TokenPowerInfoStoreItem<T>>::get_curr_token_power(block_number);

		 match symbol.clone() {
			_ if btc == symbol.clone()  => {
				ensure!(<TLCbtc>::get() > now_token_power_info.btc_count_power && <TLAbtc>::get() > now_token_power_info.btc_amount_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if eth == symbol.clone() =>  {
				ensure!(<TLCeth>::get() > now_token_power_info.eth_count_power && <TLAeth>::get() > now_token_power_info.eth_amount_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if usdt == symbol.clone() => {
				ensure!(<TLCusdt>::get() > now_token_power_info.usdt_count_power && <TLAusdt>::get() > now_token_power_info.usdt_amount_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if eos == symbol.clone() => {
				ensure!(<TLCeos>::get() > now_token_power_info.eos_count_power && <TLAeos>::get() > now_token_power_info.eos_amount_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ if ecap == symbol.clone() => {
				ensure!(<TLCecap>::get() > now_token_power_info.ecap_count_power && <TLAecap>::get() > now_token_power_info.ecap_amount_power,
				Error::<T>::AmountOrCountToMax);
			},
			_ => return Err(Error::<T>::UnknownSymbol)?
		}

		Ok(false)
	}


	/// 获取当前区块高度
	fn now() -> T::BlockNumber{
		<system::Module<T>>::block_number()
	}


	/// 计算今天的挖矿奖励
	fn per_day_mine_reward_token() -> result::Result<BalanceOf<T>, DispatchError>{
		if T::ArchiveDuration::get() == T::BlockNumber::from(0u32) {
			return Err(Error::<T>::DivZero)?;
		}
		let block_num = Self::now(); // 获取区块的高度

		let mut per_day_tokens = T::PerDayMinReward::get();

		// 国库可以使用的钱
		let useable_balance = Self::pot();

		// 如果国库剩余的钱小与最小要求的奖励金额  那么用国库剩余的来计算
		if per_day_tokens > useable_balance{
			// 取更小
			per_day_tokens = useable_balance.clone();
		}

		let e: u32 = ((100 as u64).checked_mul(<<T as system::Trait>::BlockNumber as TryInto<u64>>::try_into(block_num).ok().unwrap()).ok_or(Error::<T>::Overfolw)? /((36525*SubHalfDuration).checked_mul(<<T as system::Trait>::BlockNumber as TryInto<u64>>::try_into(T::ArchiveDuration::get()).ok().unwrap()).
		ok_or(Error::<T>::Overfolw)?)) as u32;

		// 128年之后的挖矿奖励基本为0 所以这时候可以使用最低奖励了
		if e > 32{
			T::Currency3::make_free_balance_be(&MODULE_ID.into_account(), useable_balance.saturating_sub(per_day_tokens.clone()) + T::Currency3::minimum_balance());
		}

		else{

			let num = 2_u32.pow(e);  // 意味着e最大值是32  运行32*4 = 128年
			per_day_tokens = <BalanceOf<T> as TryFrom::<Balance>>::try_from(FirstYearPerDayMineRewardToken).ok().unwrap()/<BalanceOf<T>>::from(num);

			// 如果奖励数过低  那么启用最低奖励
			if per_day_tokens < T::PerDayMinReward::get(){

				per_day_tokens = T::PerDayMinReward::get();
				T::Currency3::make_free_balance_be(&MODULE_ID.into_account(), useable_balance.saturating_sub(per_day_tokens) + T::Currency3::minimum_balance());
			}

		}

		<ThisDayReward<T>>::put(per_day_tokens.clone());

		Ok(per_day_tokens)
	}


	/// 计算膨胀算力
	fn inflate_power(who: T::AccountId, mine_power: u64) -> result::Result<u64, DispatchError>{  // todo 膨胀算力在计算算力之后  把膨胀算力加入到累计算力里面
		// 把这个usdt金额数值再放大到100倍  这样计算数值的时候才能最大限度的准确
		let mut grandpa = Permill::from_percent(0);
		let mut father = Permill::from_percent(0);
		if let Some(father_address) = <AllMiners<T>>::get(who.clone()).father_address{
			father = T::FatherInflationRatio::get();
		};
		if let Some(grandpa_address) = <AllMiners<T>>::get(who.clone()).grandpa_address{
			grandpa = T::SuperiorInflationRatio::get();
		};

		let inflate_power = (mine_power.checked_add(father * mine_power).ok_or(Error::<T>::Overfolw)?)
		.checked_add(grandpa * mine_power).ok_or(Error::<T>::Overfolw)?;
		Ok(inflate_power)
	}


	/// 计算本次挖矿每个人的奖励
	fn calculate_reward(who: T::AccountId, thistime_reward: BalanceOf<T>)
		-> result::Result<(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>), DispatchError>{

		// 分母不能是0
		let total_portion = <MinerSharePortion >::get() + <FatherSharePortion>::get() + <SuperSharePortion>::get();
		if  total_portion == 0u32 {
			return Err(Error::<T>::DivZero)?;
		}

		let mut founders_total_reward = <BalanceOf<T>>::from(0);

		if <Founders<T>>::get().len() != 0{
			// 创始团队成员拿20%
			 founders_total_reward = T::FoundationShareRatio::get() * thistime_reward.clone();

		}

		let mut miner_reward = thistime_reward.clone() - founders_total_reward;
		let miner_reward_cp = miner_reward.clone();

		// 奖励上级
		let mut fa_reward = <BalanceOf<T>>::from(0u32);

		if let Some(father_address) = <AllMiners<T>>::get(who.clone()).father_address{

			fa_reward = miner_reward_cp.checked_mul(&<BalanceOf<T>>::from(<FatherSharePortion>::get())).ok_or(Error::<T>::Overfolw)? /<BalanceOf<T>>::from(total_portion);
			miner_reward -= fa_reward.clone();
		};
		// 奖励上上级
		let mut gr_reward = <BalanceOf<T>>::from(0u32);

		if let Some(grandpa_address) = <AllMiners<T>>::get(who.clone()).grandpa_address{
			gr_reward = miner_reward_cp.checked_mul(&<BalanceOf<T>>::from(<SuperSharePortion>::get())).ok_or(Error::<T>::Overfolw)? /<BalanceOf<T>>::from(total_portion);
			miner_reward -= gr_reward.clone();
		};

		// 矿工奖励 上级奖励 上上级奖励 创始团队奖励
		let decimal = 10000_00000_00000u64;

		// todo 用于测试
		<ThisTimeReward<T>>::put(thistime_reward);

		Ok((miner_reward, fa_reward, gr_reward, founders_total_reward))
	}


	/// 计算算力的最后一步
	fn final_work_power(who: T::AccountId, MLA: u64, mine_tag: MineTag, mut pre_power: u64, mut nums: u64, today_total_power: u64, is_amount_power: bool) -> result::Result<u64, DispatchError>{
		// pre_power 前一天挖矿的该币种的总算力
		// num 未做膨胀处理的金额或是次数
		// is_amount_power 是否是金额算力
		// mine_tag 挖矿种类
		// MLA 硬顶金额

		// 前一天挖矿算力不能初始化成0
		if pre_power == 0u64 {
			return Err(Error::<T>::PrePowerErr)?;
		}

		let mut count = 0u64;

		// 把金额放大Multiple倍(算力大概是金额或是次数的Multiple倍)
		nums = nums.checked_mul(Multiple).ok_or(Error::<T>::Overfolw)?;

		// 计算膨胀算力
		nums = Self::inflate_power(who.clone(), nums)?;

		// 根据挖矿种类计算算力
		if mine_tag == MineTag::CLIENT{
			// 客户端挖矿
			nums = T::ClientRatio::get() * nums;
		}
		else{
			// 钱包挖矿
			nums = nums - T::ClientRatio::get() * nums ;
		}

		let mut final_power = 0u64;

		if is_amount_power{
			match <LastTotolAmountPowerAndMinersCount>::get().1 {
				0u64 => {count = INIT_MINER_COUNT;},  //
				n @ _ => {count = n;}
			}

			// 如果是金额算力 单次转账金额超过硬顶则用硬顶金额
			if nums > MLA{
				nums = MLA
			}
		}

		else{
			match <LastTotolCountPowerAndMinersCount>::get().1 {
				0u64 => {count = INIT_MINER_COUNT;},
				n @ _ => {count = n;}
			}
		}

		// 计算该币种前一天的挖矿平均值
		let av = pre_power.checked_div(count).ok_or(Error::<T>::Overfolw)?;

		// 计算本次算力是平均算力的多少倍
		let mut n = today_total_power.checked_add(nums).ok_or(Error::<T>::Overfolw)?.checked_div(av).ok_or(Error::<T>::Overfolw)?;
		// 向上取整
		if (today_total_power + nums) % av != 0{
			n += 1;
		}

		// todo 测试用?
		<FinalCalculateExceptTag>::mutate(|h| h.push((today_total_power, nums, av, n)));

		// 如果倍数大于10000 那么就不给了
		if n  > 10000u64{
			return Err(Error::<T>::PowerTooLarge)?;
		}

		// 如果不大于一倍 那么就用真实的膨胀算力
		if n <= 1{
			final_power = nums;
			return Ok(final_power);
		}

		// 指定exp的值
		let exp = T::DeclineExp::get() as u128;

		// 大于100倍 直接用100
		if n >= 100 {
			final_power = nums / n / 100u64;
			return Ok(final_power);
		}

		else{

			// 如果正常情况不能处理()
			if exp.checked_pow(n as u32).is_none(){

				let number = (exp as f64/ 10 as f64).powi(n as i32);

				// 如果大于100 则直接用100计算(这是合理的) 因为此时矿工的算力已经被钝化到1.01*av
				if number > 100f64{
					final_power = nums / n / 100u64;
					return Ok(final_power);
				}
				else{
					let mut e = number as u64 / 10u64;

					// 向上取整
					if number as u64 % 10u64 != 0u64{
						e += 1u64;
					}
					final_power = nums / n / e / 10u64;
					return Ok(final_power);
				}
			}
				// 如果能x^n处理
			else{

				let times = n as u128;
				if (nums as u128).checked_mul(10u128.checked_pow(times as u32).unwrap()).is_some(){
					// 不会发生错误
					final_power = ((nums as u128).checked_mul(10u128.checked_pow(times as u32).unwrap()).ok_or(Error::<T>::Overfolw)? / times / exp.checked_pow(times as u32).unwrap()) as u64;
					return Ok(final_power)

				}
				else{
					final_power = nums / n / 100u64;
					return Ok(final_power);
				}
			}
			}
	}

	/// 执行奖励操作
	fn reward_all_people(
		who: T::AccountId, miner_reward: BalanceOf<T>, fa_reward: BalanceOf<T>, gr_reward: BalanceOf<T>,
		founders_total_reward: BalanceOf<T>)
		->  DispatchResult{

		let time = Self::time();

		let fouders = <Founders<T>>::get();
		let member_count = fouders.len() as u32;
		if member_count != 0u32{
			let per_founder_reward = founders_total_reward.clone()/<BalanceOf<T>>::from(member_count);
			// 奖励每一个创始团队成员
			for i in fouders.iter(){
				// 用方法deposit_creating就可以  不要损害到矿工的利益
				T::ShouldAddOrigin::on_unbalanced(T::Currency3::deposit_creating(&i, per_founder_reward));
				<CommissionAmount<T>>::mutate(i.clone(), |h| {h.0 += per_founder_reward.clone(); h.1 = per_founder_reward.clone(); h.2 = time.clone();});
		}

		}

		// 奖励上级
		if let Some(father_address) = <AllMiners<T>>::get(who.clone()).father_address{
			T::ShouldAddOrigin::on_unbalanced(T::Currency3::deposit_creating(&father_address, fa_reward.clone()));
			<CommissionAmount<T>>::mutate(father_address.clone(), |h| {h.0 += fa_reward.clone(); h.1 = fa_reward.clone(); h.2 = time.clone();});
		};

		// 奖励上上级
		if let Some(grandpa_address) = <AllMiners<T>>::get(who.clone()).grandpa_address{
			T::ShouldAddOrigin::on_unbalanced(T::Currency3::deposit_creating(&grandpa_address, gr_reward.clone()));
			<CommissionAmount<T>>::mutate(grandpa_address.clone(), |h| {h.0 += gr_reward.clone(); h.1 = gr_reward.clone(); h.2 = time.clone();});
		};

		// 奖励矿工
		T::ShouldAddOrigin::on_unbalanced(T::Currency3::deposit_creating(&who, miner_reward.clone()));

		<MineReward<T>>::put((miner_reward, fa_reward, gr_reward, founders_total_reward));
		Ok(())
	}


	/// 计算此时国库的可用余额
	fn pot() -> BalanceOf<T> {
		// 这个方法用于时刻保护国库账号的存活
		T::Currency3::free_balance(&MODULE_ID.into_account())
			// Must never be less than 0 but better be safe.
			.saturating_sub(T::Currency3::minimum_balance())
	}


	/// 初始化昨天的挖矿算力（因为昨天的算力要作一些特殊情况的处理， 不是直接使用昨天的)
	fn init_yesterday_total_power(block_number: T::BlockNumber){
		// 统计今天挖矿人数
		let mut count =  <LastTimeMiners<T>>::get().len() as u64;

		// 如果有人挖矿
		if count != 0u64 {

			MinerCount::put(count);
			let power_info = <PowerInfoStoreItem<T>>::get_prev_power(block_number);
			// 获取今天的总金额算力
			let amount_power = power_info.amount_power;
			// 获取今天的总次数算力
			let count_power = power_info.count_power;

			// 统计金额算力

			// 计算昨日平均金额算力(如果低于最低要求就用最低要求的)
			let mut av_amount_power = amount_power/count;
			if av_amount_power < T::ZeroDayAmount::get(){
				av_amount_power = T::ZeroDayAmount::get();
			}

			// 计算昨日平均次数算力（如果低于最低要求就用最低要求的)
			let mut av_count_power = count_power/count;
			if av_count_power < T::ZeroDayCount::get(){
				av_count_power = T::ZeroDayCount::get();
			}

			// 如果人数低于最低要求 就用最低要求人数
			if count < INIT_MINER_COUNT{
				count = INIT_MINER_COUNT;
			}

			// 更新昨日算力
			<LastTotolAmountPowerAndMinersCount>::put((av_amount_power.saturating_mul(count), count));

			<LastTotolCountPowerAndMinersCount>::put((av_count_power.saturating_mul(count), count));
		}
		// 删除上个周期挖矿人员名单
		<LastTimeMiners<T>>::kill();
	}


	// 获取当下的时间戳
	fn time() -> T::Moment{
		<timestamp::Module<T>>::get()

	}


	/// 添加并存储相应的挖矿tx记录
	fn add_tx(mut mine_parm: MineParm, block_num: T::BlockNumber, sender: T::AccountId){
		// 如果交易已经进入队列，说明正在进行第二次挖矿，挖矿次数加1
		let tx = mine_parm.tx.clone();

		// 另外一种挖矿存在 说明不是第一次挖矿 不用添加记录
		let mine_tag = match mine_parm.mine_tag.clone(){
			MineTag::WALLET => MineTag::CLIENT,
			MineTag::CLIENT => MineTag::WALLET,
		};

		if <OwnerMineRecord<T>>::contains_key(tx.clone(), mine_tag){
		}
		// 如果是第一次添加该比交易 则去添加今天的日期进队列   如果已经存在不需要添加
		else{
			 // 获取区块的高度
			let now_day = block_num/T::ArchiveDuration::get();

			if <MinerAllDaysTx<T>>::contains_key(sender.clone(), now_day.clone()){
				let mut all_tx = <MinerAllDaysTx<T>>::get(sender.clone(), now_day.clone());
				all_tx.push(tx.clone());
				<MinerAllDaysTx<T>>::insert(sender.clone(), now_day.clone(), all_tx.clone());
			}
			else{
				<MinerAllDaysTx<T>>::insert(sender.clone(), now_day.clone(), vec![tx.clone()]);
			}

			// 获取本人的所有有记录的天数
			let all_days = <MinerDays<T>>::get(sender.clone());
			if all_days.is_empty(){
				let days = vec![now_day];
				<MinerDays<T>>::insert(sender.clone(), days);
			}
			else{
				if !all_days.contains(&now_day){
					let mut days = all_days.clone();
					days.push(now_day);
					<MinerDays<T>>::insert(sender.clone(), days);
				}
			}
		}
	}

	/// 初始化
	fn initialize_founders(members: &[T::AccountId]){

		<Founders<T>>::put(members);

	}

}





