#  一、mine.rs
   1. 在pub fn create_mine(挖矿)方法中添加一个代表挖矿标记的参数mine_tag,这个参数的类型是一个枚举，代表矿机挖矿还是钱包挖矿

   2. 把tx存在不能挖矿改成：tx挖矿不能超过2次，并且不是同样挖矿类型，才能挖矿

   3. 该币挖矿算力在全网中超额，则不再挖矿

   4. 每笔交易tx的第一次挖矿都要把代表当天日期的blocknum记录在案（MinerDAYS ），如果已经存在则不用添加，这个用来记录用户哪天有挖矿，以便找到过期的交易tx。

   5. 每笔交易的第一次挖矿都要把这个二级map信息加入：
      * key: AccountId 、BlockNumber
      * value: tx
          >>> 这个信息用于记录具体天数的所有交易的tx

   6. 每次挖矿自动触发去查找并删除已经过期的记录（这样可以避免操作大量数据而导致出块错误等问题）

   7. 挖矿的记录信息里添加两个字段： 挖矿标记mine_tag、 挖矿次数mine_count

   8. 地址必须处于激活状态才能挖矿

   9. 自己有被举报的tx, 则暂时不能挖矿

   >>> 以上均是在挖矿方法pub fn create_mine中完成
   ***
   8. fn remove_expire_record(who: T::AccountId, is_remove_all: bool)
   这个方法用于删除过期记录（目前打算让外部模块使用）

   9. fn remove_per_day_record(day: T::BlockNumber, who: T::AccountId)
   这个方法用于删除被选中的当天的记录（协助fn remove_expire_record方法以便更灵活实现外部调用）

   10. fn is_token_power_more_than_portion(symbol: Vec<u8>) -> bool
   这个方法用于判断该币是是否挖矿已经超额

   11. fn per_day_mine_reward_token() -> T::Balance
   计算该日期的挖矿奖励
   逻辑: 如果当天挖矿的奖励金额小于最小奖励金额 则用最小奖励金额; 如果最小奖励金额大于国库剩余金额,则用国库剩余奖励金额

   12. fn inflate_power(who: T::AccountId, mine_power: u64) -> u64
   计算膨胀算力

   13. 删除掉原来的fn mining_maximum()-> u64， Mining_Maximum在lib文件中获取

   14. fn reward_all_people(who: T::AccountId, thistime_reward: BalanceOf<T>) -> Result
   挖矿奖励所有人
      * 创始团队20%
      * 矿工:上级:上上级 = 2：1：0.5
   15. fn set_founders(fouders: Vec<T::AccountId>)
   设置创始人团队(从chain-spec中引入)
      * 目前设置成alice和bob
   16. fn Initialize_mutable_parameter()
   初始化可治理的参数 比如钝化系数
   17. caculate_workforce_ratio(amount_workforce: u64, count_workforce: u64, mut pre_amount_workfore: u64, mut pre_count_workforce: u64) -> u64
   计算本次总算力在全网的占比 这个值放大100亿倍 获得更大精度
   18. fn calculate_count_or_amount_workforce(miner_id: &T::AccountId, block_number: T::BlockNumber, coin_name: Vec<u8>, usdt_nums: u64, is_amount_power: bool)
		-> result::Result<u64, &'static str>
	计算本次挖矿的金额算力或是次数算力（不是占比）


# 二、mine_linked.rs

   1. pub struct MineParm中添加两个参数：mine_tag、 mine_count

   2. 添加pub enum Mine_Tag(挖矿类型)

# 说明
1. 挖矿是客户端 绑定to地址； 挖矿是钱包，绑定from地址
地址必须存在于矿工的tokeninfo之中 否则不允许挖矿








