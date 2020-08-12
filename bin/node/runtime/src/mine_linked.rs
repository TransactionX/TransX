use frame_support::{decl_storage, decl_module,decl_event, Parameter,StorageValue, StorageMap,
               ensure,dispatch::Vec};
use frame_system::{ensure_signed};
use sp_runtime::{ DispatchResult, traits::{ Hash,Member, AtLeast32Bit,Bounded,MaybeDisplay,CheckedAdd}};
use codec::{Encode, Decode};
use sp_std::{self, result};

use crate::constants::time::ArchiveDurationTime;

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode, Clone)]
pub struct MineParm {
	pub mine_tag: MineTag,
	pub mine_count: u16,
    pub action:Vec<u8>,
    pub tx:Vec<u8>,
    pub address:Vec<u8>,
    pub to_address:Vec<u8>,
    pub symbol:Vec<u8>,
    pub amount:Vec<u8>,  // eth 等需要是整数   amount/10.pow(decimal)
    pub protocol:Vec<u8>,
    pub decimal:u32,  // 精度
    pub usdt_nums: u64,
    pub blockchain:Vec<u8>,
    pub memo:Vec<u8>
}

// 个人算力 汇总表
#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct PersonMineWorkForce<BlockNumber>{
    mine_cnt: u64, // 当天的挖矿次数
    usdt_nums: u64,  // 完成的金额
    amount_work_force: u64,  // 当天金额算力
	count_work_force: u64,  // 当天次数算力
    settle_blocknumber:BlockNumber, // 上一次结算时的区块高度,用于区分是否是第二天了
}

// 为了来存储 PersonMineWorkForce
// 仅仅是为了让编译器通过, Storage:PersonMineWorkForce Key:T::AccountId
// 这里可以传递任意多个泛型,只要后面被使用就行
pub struct PersonMine<Storage, Key,BlockNumber>(sp_std::marker::PhantomData<(Storage, Key,BlockNumber)>);

impl<Storage, Key,BlockNumber> PersonMine<Storage, Key,BlockNumber> where
    Key: Parameter, // Key  T::AccountId
    BlockNumber:Parameter + Member + MaybeDisplay + AtLeast32Bit + Default + Bounded + Copy,
    Storage: StorageMap<(Key,BlockNumber),PersonMineWorkForce<BlockNumber>, Query = Option<PersonMineWorkForce<BlockNumber>>>,
{
    fn write(key: &Key,day:BlockNumber, personmine_work_force: PersonMineWorkForce<BlockNumber>) {
        Storage::insert(&(key.clone(),day),personmine_work_force);
    }

    fn read(key: &Key,day_num:BlockNumber) ->PersonMineWorkForce<BlockNumber>{
        let zero_block = BlockNumber::from(0 as u32);
        Storage::get(&(key.clone(),day_num)).unwrap_or_else(|| PersonMineWorkForce {
            mine_cnt: 0,
            usdt_nums: 0,
            amount_work_force:0,
			count_work_force: 0,
            settle_blocknumber: zero_block
        })
    }


    pub fn add(amount_work_force: u64, count_work_force: u64, key: &Key,usdt_nums:u64,now_day:BlockNumber,block_num:BlockNumber)-> DispatchResult{
        // 获取上次的算力
        let mut personmine_work_force = Self::read(key,now_day);
        let block_nums = BlockNumber::from(ArchiveDurationTime);
        let last_day = personmine_work_force.settle_blocknumber.checked_div(&block_nums)
                        .ok_or("add function: div causes error of last_day")?;
        let now_day = block_num.checked_div(&block_nums)
                        .ok_or("user add function: div causes error of now_day")?;

        personmine_work_force.settle_blocknumber = block_num;
        if last_day==now_day{
            // 相当于是同一天
            personmine_work_force.mine_cnt =  personmine_work_force.mine_cnt.checked_add(1)
                                .ok_or("add function: add causes overflow of mine_cnt")?;
            personmine_work_force.usdt_nums =  personmine_work_force.usdt_nums.checked_add(usdt_nums)
                                .ok_or("add function: add causes overflow of usdt_nums")?;
            personmine_work_force.amount_work_force = personmine_work_force.amount_work_force.checked_add(amount_work_force)
                                .ok_or("add function: add causes overflow of work_force")?;
			personmine_work_force.count_work_force = personmine_work_force.count_work_force.checked_add(count_work_force)
                                .ok_or("add function: add causes overflow of work_force")?;

        }else{
            //第二天
            personmine_work_force.mine_cnt =  1;
            personmine_work_force.usdt_nums =  usdt_nums;
			personmine_work_force.amount_work_force = amount_work_force;
			personmine_work_force.count_work_force = count_work_force
        }

        Self::write(key,now_day,personmine_work_force);
        Ok(())
    }
}
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum MineTag {  // TODO HAHAHA
	CLIENT,  // 收款客户端
	WALLET,  // 钱包
}
// 挖矿的标记参数  用于确定是来自收款客户端还是 钱包


// 个人算力 单次挖矿表, 不做存储
#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode,Clone)]
pub struct PersonMineRecord<Moment,BlockNumber,Balance,AccountId>{
	pub mine_tag: MineTag, // 本次交易的挖矿标记
	pub mine_count: u16, // 这一比交易的挖矿次数
    pub timestamp:Moment,         // 挖矿时间
    pub blocknum:BlockNumber,
    pub miner_address:AccountId,   //矿工地址
    pub from_address:Vec<u8>,    // 不为空，钱包发起支付挖矿地址
    pub to_address:Vec<u8>,      // 不为空，接收客户端挖矿地址
    pub symbol:Vec<u8>,          // 币种
    pub blockchain:Vec<u8>,       // 哪条链
    pub tx:Vec<u8>,              // 交易的hash
    pub usdt_amount:u64,         // usdt 总价格
    // 币种个数的计算方式: sym_amount/10.pow(decimal)
    pub sym_amount:Vec<u8>,        //
    pub decimal:u32,           // 币种精度
    pub pcount_workforce:u64,     // 这次交易频次算力
    pub pamount_workforce:u64,     //这次交易金额算力
    pub reward:Balance,                 // 奖励的token
    pub grandpa_reward:Balance,        // 上级奖励的token
    pub father_reward:Balance           // 上上级奖励的token
}

pub fn bytes_into_int(bytes: Vec<u8>) -> u128{
	let mut len = bytes.len() as u32;
	let mut num = 0u128;
	for i in bytes.iter(){
		let num = num + (u128::from_le((*i).into()) - 48u128) * (10 as u128).pow(len as u32);
		len -= 1u32;
	}
	num

}

impl <Moment,BlockNumber,Balance,AccountId>PersonMineRecord<Moment,BlockNumber,Balance,AccountId>
    where Balance:Copy,   // 只需要有copy属性
{
    pub fn new(mine_parm:&MineParm,sender:AccountId,moment:Moment,block_number:BlockNumber,
			   reward: Balance, on_reward: Balance, superior_reward: Balance, pcount_workforce: u64,
				pamount_workforce: u64)
        ->  result::Result<PersonMineRecord<Moment,BlockNumber,Balance,AccountId>, &'static str> {

        if bytes_into_int(mine_parm.amount.clone()) > u128::max_value(){
            // panic!("overflow f64");
            return Err("overflow f64");
        }

        let res =  PersonMineRecord{
			mine_tag: mine_parm.mine_tag.clone(),
			mine_count: mine_parm.mine_count.clone(),
            timestamp:moment,
            blocknum: block_number,
            miner_address: sender,  // transx用户地址?
            from_address: mine_parm.address.clone(),
            to_address: mine_parm.to_address.clone(),
            symbol: mine_parm.symbol.clone(),

			sym_amount: mine_parm.amount.clone(),

            blockchain: mine_parm.blockchain.clone(),
            tx: mine_parm.tx.clone(),
            usdt_amount: mine_parm.usdt_nums,
			decimal: mine_parm.decimal,
            pcount_workforce: pcount_workforce,
            pamount_workforce: pamount_workforce,

            reward: reward,  // 矿工的奖励
            grandpa_reward: superior_reward,  //上上级的奖励
            father_reward: on_reward  // 上级奖励
        };
        Ok(res)
    }

}
