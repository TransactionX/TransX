use sp_std::prelude::*;
use sp_std::convert::TryInto;
use frame_support::{debug, ensure, decl_module, decl_storage, decl_event, Parameter,
               StorageValue, StorageMap, StorageDoubleMap, Blake2_256};
use sp_runtime::traits::{ Hash, Member, AtLeast32Bit, Bounded, MaybeDisplay, CheckedAdd, MaybeSerializeDeserialize};
use frame_system::ensure_signed;
use pallet_timestamp;
use codec::{Encode, Decode};
use sp_std::{self, result};
use sp_std::fmt::Debug;

use crate::constants::{time::ArchiveDurationTime, symbol::{EOS, ETH, BTC, USDT, ECAP}};

/// `PowerInfo`存储全网的算力信息，每日都会归档一次，并新建一个供当日使用。
/// `ChainRunDays`表示区块链运行天数，可以根据`ChainRunDays`获取当前`PowerInfo`。
/// `ChainRunDays`通过let chain_run_days = block_number / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);计算而来。
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PowerInfo<BlockNumber> {
    pub total_power: u64,                       // 24小时总算力
    pub(crate) total_count: u64,                           // 24小时总交易次数
	pub count_power: u64,

    pub total_amount: u64,                          // 24小时总金额（以USDT计）
	pub amount_power: u64,

    block_number: BlockNumber,                  // 区块高度
}

pub struct PowerInfoStore<Storage, BlockNumber>(sp_std::marker::PhantomData<(Storage, BlockNumber)>);
impl<Storage, BlockNumber>PowerInfoStore<Storage, BlockNumber> where
    BlockNumber:Parameter + Member + MaybeDisplay + AtLeast32Bit + Default + Bounded + Copy + MaybeSerializeDeserialize
+ Debug + From<u32>,
    Storage: StorageMap<u32, PowerInfo<BlockNumber>, Query = Option<PowerInfo<BlockNumber>>>,
{
    fn new_power_info(block_number: BlockNumber) -> PowerInfo<BlockNumber> {
        PowerInfo {total_power: 0u64, total_count: 0u64, count_power: 0u64, total_amount: 0u64, amount_power: 0u64, block_number }
    }

    // 获取编号为number的PowerInfo，number=1表示存储第一天的算力信息，当获取不到时，
    // 则返回一个新建的PowerInfo。
    fn read(number: u32, block_number: BlockNumber) -> PowerInfo<BlockNumber> {
        Storage::get(&number).unwrap_or_else(|| Self::new_power_info(block_number))
    }

    fn write(number: u32, power_info: &PowerInfo<BlockNumber>) {
        Storage::insert(&number, power_info);
    }

    // 从本地存储中获取当前24小时内的算力信息
    pub(crate) fn get_curr_power(block_number: BlockNumber) -> PowerInfo<BlockNumber> {
        let chain_run_days = block_number.clone() / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();

        Self::read(number, block_number)
    }

    // 从本地存储中获取前一天的算力信息
    pub(crate) fn get_prev_power(block_number: BlockNumber) -> PowerInfo<BlockNumber> {
        let chain_run_days = block_number.clone() / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();

        Self::read(number-1, block_number)
    }

    // 增加算力
    pub(crate) fn add_power(add_power_value: u64, add_count: u64, add_count_power: u64,
                 add_amount: u64, add_amount_power: u64,  block_number: BlockNumber) -> result::Result<PowerInfo<BlockNumber>, &'static str> {
        let chain_run_days = block_number.clone() / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();
        let mut power_info = Self::read(number, block_number.clone());

        power_info.total_power += add_power_value;
        power_info.total_count += add_count;
		power_info.count_power += add_count_power;
        power_info.total_amount += add_amount;
		power_info.amount_power += add_amount_power;
        power_info.block_number = block_number;

        Self::write(number, &power_info);

        Ok(power_info)
    }

    // 对当日的算力进行归档，不可更改地存储在网络中。
    pub(crate) fn archive(block_number: BlockNumber) -> result::Result<PowerInfo<BlockNumber>, &'static str> {
        let chain_run_days = block_number / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();
        let mut archive_power_info = Self::read(number, block_number.clone());

        archive_power_info.block_number = block_number.clone();

        Self::write(number, &archive_power_info);

        let new_power_info = Self::new_power_info(block_number.clone());
        Self::write(number+1, &new_power_info);

        Ok(archive_power_info)
    }

}


/// `TokenPowerInfo`记录每日的每个Token的算力信息，和`PowerInfo`一样，通过`ChainRunDays`来获取。
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TokenPowerInfo<BlockNumber> {

    pub btc_total_power: u64,         // 24小时BTC累计算力
    pub(crate) btc_total_count: u64,             // 24小时BTC累计交易次数
	pub btc_count_power: u64,
    pub(crate) btc_total_amount: u64,            // 24小时BTC累计交易金额，以USDT计算
	pub btc_amount_power: u64,

    pub eth_total_power: u64,         // 24小时ETH累计算力
    pub eth_total_count: u64,             // 24小时ETH累计交易次数
	pub eth_count_power: u64,
    pub eth_total_amount: u64,            // 24小时ETH累计交易金额，以USDT计算
	pub eth_amount_power: u64,

    pub eos_total_power: u64,         // 24小时EOS累计算力
    pub eos_total_count: u64,             // 24小时EOS累计交易次数
	pub eos_count_power: u64,
    pub eos_total_amount: u64,            // 24小时EOS累计交易金额，以USDT计算
	pub eos_amount_power: u64,

    pub usdt_total_power: u64,         // 24小时USDT累计算力
    pub usdt_total_count: u64,             // 24小时USDT累计交易次数
	pub usdt_count_power: u64,
    pub usdt_total_amount: u64,            // 24小时USDT累计交易金额，以USDT计算
	pub usdt_amount_power: u64,

	pub ecap_total_power: u64,         // 24小时USDT累计算力
    pub ecap_total_count: u64,             // 24小时USDT累计交易次数
	pub ecap_count_power: u64,
    pub ecap_total_amount: u64,            // 24小时USDT累计交易金额，以USDT计算
	pub ecap_amount_power: u64,


    block_number: BlockNumber,        // 区块高度
}

pub struct TokenPowerInfoStore<Storage, BlockNumber>(sp_std::marker::PhantomData<(Storage, BlockNumber)>);
impl<Storage, BlockNumber>TokenPowerInfoStore<Storage, BlockNumber> where
    BlockNumber:Parameter + Member + MaybeDisplay + AtLeast32Bit + Default + Bounded + Copy + From<u32>,
    Storage: StorageMap<u32, TokenPowerInfo<BlockNumber>, Query = Option<TokenPowerInfo<BlockNumber>>>,
{
    fn new_token_power_info(block_number: BlockNumber) -> TokenPowerInfo<BlockNumber> {
        TokenPowerInfo {
            btc_total_power: 0u64, btc_total_count: 0u64, btc_count_power: 0u64, btc_total_amount: 0u64, btc_amount_power: 0u64,
            eth_total_power: 0u64, eth_total_count: 0u64, eth_count_power: 0u64, eth_total_amount: 0u64, eth_amount_power: 0u64,
            eos_total_power: 0u64, eos_total_count: 0u64, eos_count_power: 0u64, eos_total_amount: 0u64,  eos_amount_power: 0u64,
            usdt_total_power: 0u64, usdt_total_count: 0u64, usdt_count_power: 0u64, usdt_total_amount: 0u64, usdt_amount_power:0u64,
			ecap_total_power: 0u64, ecap_total_count: 0u64, ecap_count_power: 0u64, ecap_total_amount: 0u64, ecap_amount_power:0u64,
            block_number
        }
    }

    // 获取编号为number的TokenPowerInfo，number=1表示存储第一天的算力信息，当获取不到时，
    // 则返回一个新建的TokenPowerInfo。
    fn read(number: u32, block_number: BlockNumber) -> TokenPowerInfo<BlockNumber> {
        Storage::get(&number).unwrap_or_else(|| Self::new_token_power_info(block_number))
    }

    fn write(number: u32, token_power_info: &TokenPowerInfo<BlockNumber>) {
        Storage::insert(&number, token_power_info);
    }

    // 从本地存储中获取当前24小时内的TokenPowerInfo
    pub(crate) fn get_curr_token_power(block_number: BlockNumber) -> TokenPowerInfo<BlockNumber> {
        let chain_run_days = block_number / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();

        Self::read(number, block_number.clone())
    }

    // 从本地存储中获取前一天的TokenPowerInfo
    pub(crate) fn get_prev_token_power(block_number: BlockNumber) -> TokenPowerInfo<BlockNumber> {
        let chain_run_days = block_number / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();

        Self::read(number-1, block_number.clone())
    }

    // 增加Token算力
    pub(crate) fn add_token_power(token_name: &'static str, add_power: u64, add_count: u64, add_count_power: u64,
                       add_amount: u64, add_amount_power:u64, block_number: BlockNumber)
        -> result::Result<TokenPowerInfo<BlockNumber>, &'static str> {
        let chain_run_days = block_number / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();
        let mut token_power_info = Self::read(number, block_number.clone());

		match token_name{
			BTC => {
				token_power_info.btc_total_power += add_power;
				token_power_info.btc_total_count += add_count;
				token_power_info.btc_count_power += add_count_power;
				token_power_info.btc_total_amount += add_amount;
				token_power_info.btc_amount_power += add_amount_power;
				},
			ETH => {
				token_power_info.eth_total_power += add_power;
				token_power_info.eth_total_count += add_count;
				token_power_info.eth_count_power += add_count_power;
				token_power_info.eth_total_amount += add_amount;
				token_power_info.eth_amount_power += add_amount_power;
			},
			EOS => {
				token_power_info.eos_total_power += add_power;
				token_power_info.eos_total_count += add_count;
				token_power_info.eos_count_power += add_count_power;
				token_power_info.eos_total_amount += add_amount;
				token_power_info.eos_amount_power += add_amount_power;
			},
			USDT => {
				token_power_info.usdt_total_power += add_power;
				token_power_info.usdt_total_count += add_count;
				token_power_info.usdt_count_power += add_count_power;
				token_power_info.usdt_total_amount += add_amount;
				token_power_info.usdt_amount_power += add_amount_power;
			},

			ECAP=> {
				token_power_info.ecap_total_power += add_power;
				token_power_info.ecap_total_count += add_count;
				token_power_info.ecap_count_power += add_count_power;
				token_power_info.ecap_total_amount += add_amount;
				token_power_info.ecap_amount_power += add_amount_power;

			}

			_ => {
			return Err("Unsupported token")
			}

		}

        token_power_info.block_number = block_number.clone();

        Self::write(number, &token_power_info);

        Ok(token_power_info)
    }

    // 对当日的Token算力进行归档，不可更改地存储在网络中。
    pub(crate) fn archive(block_number: BlockNumber) -> result::Result<TokenPowerInfo<BlockNumber>, &'static str> {
        let chain_run_days = block_number / BlockNumber::from(ArchiveDurationTime) + BlockNumber::from(1u32);
        let number: u32 = chain_run_days.try_into().ok().unwrap();
        let mut archive_token_power_info = Self::read(number, block_number.clone());

        archive_token_power_info.block_number = block_number.clone();

        Self::write(number, &archive_token_power_info);

        let new_token_power_info = Self::new_token_power_info(block_number.clone());
        Self::write(number+1, &new_token_power_info);

        Ok(archive_token_power_info)
    }
}


/// `MinerPowerInfo`保存矿工的算力信息。由于每个矿工都要保存一个这样的结构，并且计算矿工的挖矿算力需要
/// 使用前一天的算力，因此使用MinerPowerInfoDict get(miner_power_info): map(T::AccountId, MinerPowerInfoPrevPoint) => Option<MinerPowerInfo<T>>;
/// 来存储。其中MinerPowerInfoPrevPoint用来区分存储前一天矿工算力信息。
///  = 0，表示第一天挖矿，矿工还不存在前一日算力信息。
///  = 1，表示前一天挖矿信息保存在`MinerPowerInfoDict(AccountId, 1)`中，而当日的算力信息保存在`MinerPowerInfoDict(AccountId, 2)`中。
///  = 2，表示前一天挖矿信息保存在`MinerPowerInfoDict(AccountId, 2)`中，而当日的算力信息保存在`MinerPowerInfoDict(AccountId, 1)`中。
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MinerPowerInfo<AccountId, BlockNumber> {
    miner_id: AccountId,                        // 矿工ID
    total_power: u64,                           // 24小时累计算力
    total_count: u64,                           // 24小时累计交易次数
	pub(crate) count_power: u64,
    pub(crate) total_amount: u64,                          // 24小时累计交易金额，以USDT计算
	pub(crate) amount_power: u64,

    btc_power: u64,                             // 24小时BTC累计算力
    pub(crate) btc_count: u64,                             // 24小时BTC累计次数
	pub(crate) btc_count_power: u64,
    btc_amount: u64,                            // 24小时BTC累计金额，以USDT计算
	pub(crate) btc_amount_power: u64,


    eth_power: u64,                             // 24小时ETH累计算力
    pub(crate) eth_count: u64,                             // 24小时ETH累计次数
	pub(crate) eth_count_power: u64,
    eth_amount: u64,                            // 24小时ETH累计金额，以USDT计算
	pub(crate) eth_amount_power: u64,

    eos_power: u64,                             // 24小时EOS累计算力
    pub(crate) eos_count: u64,                             // 24小时EOS累计次数
	pub(crate) eos_count_power: u64,
    eos_amount: u64,                            // 24小时EOS累计金额，以USDT计算
	pub(crate) eos_amount_power: u64,

    usdt_power: u64,                            // 24小时USDT累计算力
    pub(crate) usdt_count: u64,                            // 24小时USDT累计次数
	pub(crate) usdt_count_power: u64,
    usdt_amount: u64,                           // 24小时USDT累计金额，以USDT计算
	pub(crate) usdt_amount_power: u64,

	ecap_power: u64,                            // 24小时USDT累计算力
    pub(crate) ecap_count: u64,                            // 24小时USDT累计次数
	pub(crate) ecap_count_power: u64,
    ecap_amount: u64,                           // 24小时USDT累计金额，以USDT计算
	pub(crate) ecap_amount_power: u64,

    block_number: BlockNumber,                  // 区块高度
}

pub struct MinerPowerInfoStore<Storage, AccountId, BlockNumber>(sp_std::marker::PhantomData<(Storage, AccountId, BlockNumber)>);
impl<Storage, AccountId, BlockNumber>MinerPowerInfoStore<Storage, AccountId, BlockNumber> where
    AccountId: Parameter,
    BlockNumber:Parameter + Member + MaybeDisplay + AtLeast32Bit + Default + Bounded + Copy + From<u32>,
    Storage: StorageDoubleMap<u32, AccountId, MinerPowerInfo<AccountId, BlockNumber>, Query = Option<MinerPowerInfo<AccountId, BlockNumber>>>,
{
    fn new_miner_power_info(miner_id: &AccountId, block_number: BlockNumber) -> MinerPowerInfo<AccountId, BlockNumber> {
        MinerPowerInfo {
            miner_id: miner_id.clone(),
            total_power: 0u64, total_count: 0u64, total_amount: 0u64,count_power: 0u64, amount_power: 0u64,
            btc_power: 0u64, btc_count: 0u64, btc_amount: 0u64, btc_count_power: 0u64, btc_amount_power: 0u64,
            eth_power: 0u64, eth_count: 0u64, eth_amount: 0u64, eth_count_power: 0u64, eth_amount_power: 0u64,
            eos_power: 0u64, eos_count: 0u64, eos_amount: 0u64, eos_count_power: 0u64, eos_amount_power: 0u64,
            usdt_power: 0u64, usdt_count: 0u64, usdt_amount: 0u64, usdt_count_power: 0u64, usdt_amount_power: 0u64,
			ecap_power: 0u64, ecap_count: 0u64, ecap_amount: 0u64, ecap_count_power: 0u64, ecap_amount_power: 0u64,
            block_number,
        }
    }

    fn read(number: u32, miner_id: &AccountId, block_number: BlockNumber) ->MinerPowerInfo<AccountId, BlockNumber>{
        Storage::get(&number, &(miner_id.clone())).unwrap_or_else(|| Self::new_miner_power_info(miner_id, block_number))
    }

    fn write(number: u32, miner_id: &AccountId, miner_power_info: &MinerPowerInfo<AccountId, BlockNumber>) {
        Storage::insert(&number, &(miner_id.clone()),miner_power_info);
    }

    // 获取指定编号的矿工算力信息
    pub(crate) fn get_miner_power_info(number: u32, miner_id: &AccountId, block_number: BlockNumber) -> MinerPowerInfo<AccountId, BlockNumber> {
        Self::read(number, miner_id, block_number)
    }

    // 增加矿工当日算力，curr_point指MinerPowerInfoDict中指向当日算力的key，为MinerPowerInfoPrevPoint的对立值。
    pub(crate) fn add_miner_power(miner_id: &AccountId, curr_point: u32, token_name: &'static str, add_power: u64,
                       add_count: u64, add_count_power: u64, add_amount: u64, add_amount_power: u64, block_number: BlockNumber) -> result::Result<MinerPowerInfo<AccountId, BlockNumber>, &'static str> {
        let mut miner_power_info = Self::read(curr_point, miner_id, block_number);
		match token_name{
			BTC => {
				miner_power_info.btc_power += add_power;
				miner_power_info.btc_count += add_count;
				miner_power_info.btc_amount += add_amount;
				miner_power_info.btc_count_power += add_count_power;
				miner_power_info.btc_amount_power += add_amount_power;
			},

			ETH => {
				miner_power_info.eth_power += add_power;
				miner_power_info.eth_count += add_count;
				miner_power_info.eth_amount += add_amount;
				miner_power_info.eth_count_power += add_count_power;
				miner_power_info.eth_amount_power += add_amount_power;
			},

			EOS => {
				miner_power_info.eos_power += add_power;
				miner_power_info.eos_count += add_count;
				miner_power_info.eos_amount += add_amount;
				miner_power_info.eos_count_power += add_count_power;
				miner_power_info.eos_amount_power += add_amount_power;
			},

			USDT => {
				miner_power_info.usdt_power += add_power;
				miner_power_info.usdt_count += add_count;
				miner_power_info.usdt_amount += add_amount;
				miner_power_info.usdt_count_power += add_count_power;
				miner_power_info.usdt_amount_power += add_amount_power;
			},

			ECAP => {
				miner_power_info.ecap_power += add_power;
				miner_power_info.ecap_count += add_count;
				miner_power_info.ecap_amount += add_amount;
				miner_power_info.ecap_count_power += add_count_power;
				miner_power_info.ecap_amount_power += add_amount_power;
			}
			_ => {
			return Err("Unsupported token")
			}

		}


		miner_power_info.total_power += add_power;
		miner_power_info.total_count += add_count;
		miner_power_info.total_amount += add_amount;
		miner_power_info.count_power += add_count_power;
		miner_power_info.amount_power += add_amount_power;

        miner_power_info.block_number = block_number.clone();

        Self::write(curr_point, miner_id, &miner_power_info);

        Ok(miner_power_info)
    }

    // 对当日的矿工算力进行归档：将前一日的算力值清空，然后将当日的算力设置为前一个日的算力。
    pub(crate) fn archive(prev_point: u32, block_number: BlockNumber)
        -> result::Result<(), &'static str> {
        let curr_point = match prev_point {
            1 => 2,
            2 => 1,
            _ => 0,
        };

        if curr_point == 0 {
            return Err("Today's miner power info does not exist");
        }

        Storage::remove_prefix(&prev_point);

        Ok(())
    }
}


