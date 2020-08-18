/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

// We have to import a few things
use sp_std::{prelude::*,convert::TryInto};
use sp_core::{crypto::AccountId32 as AccountId};
use sp_core::{crypto::KeyTypeId,offchain::Timestamp};

use frame_support::{print,Parameter,decl_module, decl_storage, decl_event, dispatch, debug, traits::Get,IterableStorageMap,
                    StorageDoubleMap, IterableStorageDoubleMap, ensure,weights::Weight};
use frame_system::{self as system,RawOrigin,Origin, ensure_signed,ensure_none, offchain,
                   offchain::{SubmitTransaction,SendTransactionTypes}};
//use simple_json::{ self, json::JsonValue };

use hex;
use pallet_timestamp as timestamp;
use pallet_authority_discovery as authority_discovery;

use sp_runtime::{DispatchResult,DispatchError};
use sp_io::{self};
use codec::{ Encode,Decode };
use num_traits::float::FloatCore;
use sp_runtime::{
    AnySignature,MultiSignature,MultiSigner,
    offchain::http, transaction_validity::{
        TransactionValidity, TransactionLongevity, ValidTransaction, InvalidTransaction,TransactionSource},
    traits::{CheckedSub,CheckedAdd,Printable,Member,Zero,IdentifyAccount},
    RuntimeAppPublic};
use app_crypto::{sr25519};
//use crate::price_fetch::crypto::AuthorityId;

use crate::mine::{self,TxVerifyMap,LenOfTxVerify,OwnerMineRecord,OwnerMineRecordItem, Trait as MineTrait};
use crate::mine_linked::MineTag;
use crate::report::{self,Trait as ReportTrait};
use crate::offchain_common::*;


type BlockNumberOf<T> = <T as system::Trait>::BlockNumber;  // u32
type StdResult<T> = core::result::Result<T, &'static str>;
// 为了兼容返回为空的情况
pub type StrDispatchResult = core::result::Result<(), &'static str>;

/// Our local KeyType.
///
/// For security reasons the offchain worker doesn't have direct access to the keys
/// but only to app-specific subkeys, which are defined and grouped by their `KeyTypeId`.
pub const TX_KEY_TYPE: KeyTypeId = KeyTypeId(*b"oftx");
pub const VERIFY_MSG: &[u8] = b"verify_msg";        // 验证的失败消息

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode,Clone)]
pub struct FetchFailed<Moment>{
    // 失败的请求
    timestamp:Moment,
    tx:Vec<u8>,
    err:Vec<u8>
}


type FetchFailedOf<T> = FetchFailed<<T as timestamp::Trait>::Moment>;


type Signature = AnySignature;
pub mod tx_crypto {
    use super::{TX_KEY_TYPE as KEY_TYPE,AccountIdPublicConver,Signature};
    pub mod app_sr25519 {
        use super::{KEY_TYPE,AccountIdPublicConver};
        //        use app_crypto::{app_crypto, sr25519};
//        use node_primitives::{AccountId};
        use sp_runtime::{MultiSignature,MultiSigner};
        use sp_runtime::traits::{IdentifyAccount};  // AccountIdConversion,
        use sp_core::{crypto::AccountId32 as AccountId};
        use sp_runtime::app_crypto::{app_crypto, sr25519};
        app_crypto!(sr25519, KEY_TYPE);

        impl From<Signature> for super::Signature {
            fn from(a: Signature) -> Self {
                sr25519::Signature::from(a).into()
            }
        }

        impl AccountIdPublicConver for Public{
            type AccountId = AccountId;
            fn into_account32(self) -> AccountId{
                let s: sr25519::Public = self.into();
                MultiSigner::from(s).into_account()
            }
        }
    }

    pub type AuthorityId = app_sr25519::Public;
    #[cfg(feature = "std")]
    pub type AuthorityPair = app_sr25519::Pair;
}


// 请求的查询接口
pub const TX_FETCHED_CRYPTS: [(&[u8], &[u8]); 1] = [
    (b"localhost",b"http://localhost:8421/v1/tx/verify"),
];

enum ReportStatus {
    Continue,  //  不做任何处理OwnerMineRecordItem
    Report,    // 举报
    Pass,      // 通过
}

/// The module's configuration trait.
pub trait Trait: TxValidLocalAuthorityTrait + SendTransactionTypes<Call<Self>> + MineTrait + ReportTrait{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
//    type Call: From<Call<Self>>;

//    type SubmitSignedTransaction: offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
//    type SubmitUnsignedTransaction: offchain::SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;

    /// The local AuthorityId
//    type AuthorityId: RuntimeAppPublic + Clone + Parameter+ Into<sr25519::Public> + From<sr25519::Public>+ AccountIdPublicConver<AccountId=Self::AccountId>; // From<Self::AccountId> + Into<Self::AccountId>

    type Duration: Get<Self::BlockNumber>;  // 对记录的清除周期

}

decl_event!(
  pub enum Event<T> where
    Moment = <T as timestamp::Trait>::Moment,
    AccountId = <T as system::Trait>::AccountId,
    {
    FetchedSuc(AccountId,Moment, Vec<u8>, u64), // 当前tx 查询状态
  }
);

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as txValid {
        //记录查询结果,key: T::BlockNumber(1小时的周期数)+T::AccountId, val:(成功次数,2000x 状态码次数,5000x状态码次数).不会删除
       FetchRecord get(fn fetch_record): double_map hasher(blake2_128_concat) T::BlockNumber,hasher(blake2_128_concat) T::AccountId => (u32,u32,u32);

       // 记录失败的,定期全部清除. Vec<FetchFailedOf<T>> 最多保持50个的长度.原本是 linked_map
       pub TxFetchFailed get(fn fetch_failed): map hasher(blake2_128_concat) T::AccountId => Vec<FetchFailedOf<T>>;
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;

    // Clean the state on initialization of the block
    fn on_initialize(block: T::BlockNumber) -> Weight{
        if (block % T::Duration::get()).is_zero() {
            // 删除所有的失败记录
        for key_value in <TxFetchFailed<T>>::iter().into_iter(){ // sym,vec<>, linked_map的作用
            let (key,val) = key_value;
             <TxFetchFailed<T>>::remove(&key);
            }
        }
        0
    }

    #[weight = 0]
    pub fn record_tx(
      origin,
      _block_num:T::BlockNumber,
      _key: <T as BaseLocalAuthorityTrait>::AuthorityId,
      account_id: T::AccountId,
      mine_tag: MineTag,
      tx:Vec<u8>,
      status: u64,
      _signature: <<T as BaseLocalAuthorityTrait>::AuthorityId as RuntimeAppPublic>::Signature
    ) ->DispatchResult {
      ensure_none(origin)?;

      let now = <timestamp::Module<T>>::get();
      let block_num = <system::Module<T>>::block_number();
      let duration = block_num / T::Duration::get();
      debug::info!("-------record_tx--------");
      ensure!(<TxVerifyMap>::contains_key((tx.clone(),mine_tag.clone())), "不需要再操作了,tx 已经从TxVerifyMap队列移除");
//      let tx_vec = tx.to_vec();
//
//        1000: 表示初始值
//        1001: 表示验证，但是请求失败了
//        120x: 终止，pass
//        1x1x: 终止，举报,有1个节点验证失败
//        1009: 终止，网络全部失败   todo: 怎么处理？
//        1109: 终止，pass
      if status == 0{  // 20000
        // 成功
       <FetchRecord<T>>::mutate(
        duration,account_id.clone(),
        |val|{
            val.0 = val.0.checked_add(1).unwrap();
//            val
        });
       <TxVerifyMap>::mutate(&(tx.clone(),mine_tag.clone()),|num|*num = num.checked_add(100).unwrap());//  通过次数加1
      }else if status == 255{ // query  ./token-query 没有收到返回的消息
         <FetchRecord<T>>::mutate(
            duration,account_id.clone(),
            |val|{
                val.2 = val.2.checked_add(1).unwrap();
//                val
            });
            <TxVerifyMap>::mutate(&(tx.clone(),mine_tag.clone()),|num|*num = num.checked_add(1).unwrap());// 仅仅对总次数加1
      }else{
        <FetchRecord<T>>::mutate( // 可能是200x 400x
            duration,account_id.clone(),
            |val|{
                val.1 = val.1.checked_add(1).unwrap();
//                val
            });
            <TxVerifyMap>::mutate(&(tx.clone(),mine_tag.clone()),|num|*num = num.checked_add(10).unwrap()); // 失败次数加1,总次数加1
      }
      if let ReportStatus::Report = Self::tx_verify_map_handle(&tx,mine_tag.clone())?{
        // 举报
        debug::warn!("调用举报举报");
        let origin = T::Origin::from(RawOrigin::Signed(account_id));
        <report::Module<T>>::report(origin,tx,mine_tag.clone(),"".as_bytes().to_vec());  //

        // Signed tx
//         let local_accts = T::SubmitSignedTransaction::find_all_local_keys();
//         let (local_acct, local_key) = &local_accts[0];
//         debug::info!("acct: {:?}", local_acct);
//         let call = Call::record_fail_fetch(block_num,account.clone(),tx_vec[0].clone(), e.as_bytes().to_vec());
//         let call = report::Call::<T>::report(sym);
//        <T::SubmitSignedTransaction as SubmitSignedTransaction<T, <T as Trait>::Call>>::SignAndSubmit::sign_and_submit(call, local_key.clone());
      }

      debug::info!("----上链成功: record_tx-----: {:?}", duration);
      Ok(())
    }

    #[weight = 0]
    fn record_fail_fetch(
        _origin,
        _block: T::BlockNumber,
        _key: <T as BaseLocalAuthorityTrait>::AuthorityId,
        account: T::AccountId,
        mine_tag: MineTag,
        tx: Vec<u8>,
        err: Vec<u8>,
        _signature: <<T as BaseLocalAuthorityTrait>::AuthorityId as RuntimeAppPublic>::Signature
        )->DispatchResult{
            // 记录获取fetch失败的信息
            ensure_none(_origin)?;
            debug::info!("--------record_fail_fetch--------");
            ensure!(<TxVerifyMap>::contains_key((tx.clone(),mine_tag.clone())), "不需要再操作了,tx 已经从TxVerifyMap队列移除");
            let now = <timestamp::Module<T>>::get();
            let failed_struct = FetchFailedOf::<T> {
                    timestamp: now,
                    tx: tx.clone(),
                    err: err
                };

            <TxVerifyMap>::mutate(&(tx.clone(),mine_tag.clone()),|num|*num = num.checked_add(1).unwrap());// 仅仅对总次数加1
            let status:u64 = <TxVerifyMap>::get((tx.clone(),mine_tag.clone()));
            debug::info!("链上 修改后的tx 状态码={:?}",status);
            if let ReportStatus::Report = Self::tx_verify_map_handle(&tx,mine_tag.clone())?{
                // 举报
                 debug::warn!("调用举报举报");
                let origin = T::Origin::from(RawOrigin::Signed(account.clone()));
                <report::Module<T>>::report(origin,tx,mine_tag.clone(),"".as_bytes().to_vec());  //
            }
            <TxFetchFailed<T>>::mutate(account, |fetch_failed| {
            if fetch_failed.len()>50{  // 最多保留50个的长度
                fetch_failed.pop();
            }
            fetch_failed.push(failed_struct)
            });
            debug::info!("------fetch失败记录上链成功:record_fail_fetch------");
            Ok(())
    }


    fn offchain_worker(block: T::BlockNumber) {
        if sp_io::offchain::is_validator() { // 是否是验证人的模式启动
//             let Some(key),Some(account) = Self::authority_id()
             if let (Some(authority_id),Some(account)) = T::authority_id() {
             debug::info!("-----------tx_valid offchain work------------");
                Self::offchain(block,authority_id,&account);
            }
        }
    } // end of `fn offchain_worker()`
  }
}


//fn vecchars_to_vecbytes <I: IntoIterator<Item = char> + Clone>(it: &I) -> Vec<u8> {
//    it.clone().into_iter().map(|c| c as u8).collect::<_>()
//}


impl<T: Trait> Module<T> {
    fn offchain(block_num:T::BlockNumber,key: <T as BaseLocalAuthorityTrait>::AuthorityId, account: &T::AccountId) -> DispatchResult {
        for (remote_src, remote_url) in TX_FETCHED_CRYPTS.iter() {
            // 最多推进去10个, 取最后一个
            let mut tx_vec = vec![];
//            let seed_num = seed % 10;  // 产生 10以内的数字
//            debug::info!("产生随机数:seed={},seed_num={}",seed,seed_num);
            for key_value in <TxVerifyMap>::iter().into_iter() {
                let ((tx,mine_tag), status) = key_value;
//                if <TxVerifyMap>::get(&tx) < 1000 {
//                    debug::error!("=====挖矿验证失败:当前的{:?},状态为 {:?}=====", hex::encode(&tx),<TxVerifyMap>::get(&tx));
//                    Self::call_record_fail_fetch(block_num, account.clone(), tx.clone(), b"".to_vec())?;
//                    return Err(DispatchError::Other("status less than 1000"));
//                }
                tx_vec.push((tx, mine_tag));
            }
            if tx_vec.is_empty() {
                return Ok(());
            }
//            let tx_str = core::str::from_utf8(&tx_vec[0]).map_err(|e|{
//                debug::info!("解析错误:{:?}",e);
//                "from_utf8 error" })?;

            let (tx,mine_tag) = &tx_vec[0];
            debug::info!("*****tx={:?}******",hex::encode(&tx));   // core::str::from_utf8(remote_src).unwrap()
            let body = Self::from_mine_item(&tx,mine_tag.clone()).ok_or("from_mine_item error"); // 获取请求的post body
            let body = match body {
                Ok(body) => body,
                Err(e) => {
                    debug::error!("---------from_mine_item error {:?}---------",e);
                    Self::call_record_fail_fetch(block_num,key.clone(), account.clone(),  mine_tag.clone(),tx.clone(),e.as_bytes().to_vec())?;
                    return Err(DispatchError::Other("from_mine_item error"));
                }
            };

            match T::fetch_status(*remote_src, *remote_url, body) {
                Ok(status) => {
                    debug::info!("*** fetch ***: {:?}:{:?},{:?}",
                            core::str::from_utf8(remote_src).unwrap(),
                            core::str::from_utf8(remote_url).unwrap(),
                            hex::encode(&tx),
                        );
                    Self::call_record_tx(block_num, key.clone(), account, mine_tag.clone(),&tx, status)?;
                },
                Err(e) => {
                    debug::error!("~~~~~~Error tx fetching~~~~~~~~:  {:?}: {:?},{:?}",
                    core::str::from_utf8(remote_src).unwrap(),
                    e,
                    hex::encode(&tx),
                    );
                    // 实现错误信息上报记录
                    Self::call_record_fail_fetch(block_num, key.clone(), account.clone(), mine_tag.clone(),tx.clone(), e.as_bytes().to_vec())?;
                }
            }
            break;
        }
        Ok(())
    }

//    fn fetch_tx<'a>(
//        remote_src: &'a [u8],
//        remote_url: &'a [u8],
//        body:Vec<u8>
//    ) -> StdResult<u64> {
//        let json = T::fetch_json(remote_url, body)?; // http请求
//        let status = match remote_src {
//            src if src == b"localhost" => T::fetch_parse(json)  // 解析
//                .map_err(|_| "fetch_price_from_localhost error"),
//            _ => Err("Unknown remote source"),
//        }?;
//        Ok(status)
//    }

//        T::SubmitUnsignedTransaction::submit_unsigned(call.clone())
//            .map_err(|e| {
//                debug::info!("{:?}",e);
//                "============fetch_price: submit_signed(call) error=================="})?;

//        // Signed tx
//         let local_accts = T::SubmitSignedTransaction::find_all_local_keys();
//         let (local_acct, local_key) = &local_accts[0];
//         debug::info!("acct: {:?}", local_acct);
//        <T::SubmitSignedTransaction as SubmitSignedTransaction<T, <T as Trait>::Call>>::SignAndSubmit::sign_and_submit(call, local_key.clone());

        // T::SubmitSignedTransaction::submit_signed(call);

    fn call_record_tx<'a>(
        block_num:T::BlockNumber,
        key:<T as BaseLocalAuthorityTrait>::AuthorityId,
        account_id:&T::AccountId,
        mine_tag: MineTag,
        tx:&'a [u8],  //tx
        status: u64
    )-> DispatchResult{
        let signature = key.sign(&(block_num,account_id,tx.to_vec(),status).encode()).ok_or("Offchain error: signing failed!")?;
        debug::info!("完成签名,block_num = {:?}",block_num);
        let call = Call::record_tx(
            block_num,
            key,
            account_id.clone(),
            mine_tag.clone(),
            tx.to_vec(),
            status,
            signature
        );

        // Unsigned tx
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|e| {
                debug::error!("{:?}",e);
                "============fetch_price: submit_signed(call) error=================="}
            )?;
        debug::info!("***fetch price over ^_^***");
        Ok(())
    }

    fn call_record_fail_fetch(
        block_num: T::BlockNumber,
        key: <T as BaseLocalAuthorityTrait>::AuthorityId,
        account: T::AccountId,
        mine_tag: MineTag,
        tx: Vec<u8>,
        err: Vec<u8>
    )->DispatchResult{
        let signature = key.sign(&(block_num,account.clone(),tx.to_vec()).encode()).ok_or("Offchain error: signing failed!")?;
        let call = Call::record_fail_fetch(block_num, key, account, mine_tag.clone(), tx, err, signature);
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|_| {
                debug::info!("===record_fail_fetch: submit_unsigned_call error===");
                "===record_fail_fetch: submit_unsigned_call error==="
            })?;
        debug::info!("+++++++record_fail_fetch suc++++++++++++++");
        Ok(())
    }




    fn from_mine_item(tx: &[u8],mine_tag: MineTag) -> Option<Vec<u8>> {
        //*
//         let mine_struct = <OwnerMineRecord<T>>::get(tx)?;//PersonMineRecord todo: 如果没有获取到
//		let info = <OwnerMineRecord<T>>::iter_prefix(tx).collect::<Vec<_>>();
//        let mine_struct = info.swap_remove(0).1;
        let mine_struct = <OwnerMineRecord<T>>::get(tx,mine_tag.clone())?;
        let tx = mine_struct.tx;

        let timestamp = mine_struct.timestamp;
        let timestamp:u64 =  timestamp.try_into().ok()?.try_into().ok()?;

        let symbol = mine_struct.symbol;
        let from = mine_struct.from_address;
        let to = mine_struct.to_address;

        let quantity = mine_struct.sym_amount;   // 字符串
        let decimal = mine_struct.decimal;
        let amount_usd = mine_struct.usdt_amount; // 还差 个数
       // */
        /*
        let tx = b"0x485615bff2000aa18399a0c8314239a395facf7412ee64cb57a75065f6480c84";
        let symbol = b"eth";
        let timestamp:u64 = 1578628445;
        let from = b"0x137ad9c4777e1d36e4b605e745e8f37b2b62e9c5";
        let to = b"0x0c8df6dfb99522d70d3247c4f56358ff23c0d810";
        let quantity = 58187;
        let decimal = 4; // 上面的 decimal 需要转换为 u64
        */

        let keys:[&[u8];8] = [b"tx",b"symbol",b"from",b"to",b"quantity",b"amount_usd",b"timestamp",b"decimal"];
        let timestamp = int_covert_str(timestamp);
//        let quantity = int_covert_str(quantity as u64);
        let decimal = int_covert_str(decimal.into());
        let amount_usd = int_covert_str(amount_usd);

        let tx_vec = hex_to_u8(&tx);  // 先编码为对应的hex字符串
        let from_vec = hex_to_u8(&from);
        let to_vec = hex_to_u8(&to);
        let vals:[&[u8];8] = [&tx_vec,&symbol,&from_vec,&to_vec,&quantity,&amount_usd,&timestamp,&decimal];

        let mut json_val = vec![POST_KEYWORD[0]];
        for (i,key) in keys.iter().enumerate(){
            json_val.push(POST_KEYWORD[1]); //json_val.push("\"");
            json_val.push(key);
            json_val.push(POST_KEYWORD[2]);  //json_val.push("\":");
            if i <= 5 {
                json_val.push(POST_KEYWORD[1]);  // json_val.push("\"");
                json_val.push(vals[i]);
                json_val.push(POST_KEYWORD[1]);   //json_val.push("\"");
            }else{
//                let val = int_covert_str(vals_int[i]);  放在外层,生命周期问题
//                let s = sp_std::str::from_utf8(&vals_u8[i]).ok()?;
                json_val.push(vals[i]);
            }
            json_val.push(POST_KEYWORD[3]);    //json_val.push(",");
        }

        json_val.pop();
        json_val.push(POST_KEYWORD[4]);    //json_val.push("}");
        let json_vec = json_val.concat().to_vec();
        debug::info!("mine验证,请求的json:{:?}",core::str::from_utf8(&json_vec).ok()?);
        Some(json_val.concat().to_vec())
    }

//    fn fetch_price_from_localhost(json_val: JsonValue) -> StdResult<u64> {
//        let data =  json_val.get_object().map_err(|e|"get_object failed:SimpleError")?;
//        let (k, v) = data.iter()
//            .filter(|(k, _)| VERIFY_STATUS.to_vec() == vecchars_to_vecbytes(k))
//            .nth(0)
//            .ok_or("fetch_price_from_coincap: JSON does not conform to expectation")?;
//
//        let status_utf8 = vecchars_to_vecbytes(k); //已经够了
//        // todo:  status保留测试用
//        let status = core::str::from_utf8(&status_utf8).map_err(|e|"status from utf8 to str failed")?;
//        let val:u64 = v.get_number_f64().map_err(|e|"get_number_f64 failed")? as u64;
//        Ok(val)
//    }

    fn tx_verify_map_handle(tx: &[u8],mine_tag: MineTag) -> StdResult<ReportStatus>{
        // 是否举报, true 表示举报
//        判断
//        1.十位的数字 >=1 report ,举报
//        2.个位数字 >=8, report
//        3.百位数字 >=2,pass

        let mut report_status:ReportStatus = ReportStatus::Continue;//初始化一个值
        let status:u64 = <TxVerifyMap>::get((tx.clone(),mine_tag.clone()));
        let num = LenOfTxVerify::get();
        if status < 1000{
            debug::error!("=====挖矿验证失败:当前的{:?},状态为 {:?}=====", hex::encode(&tx),status);
            <TxVerifyMap>::remove((tx.clone(),mine_tag.clone()));
            if num > 0{
                LenOfTxVerify::mutate(|n|*n -= 1);
            }
            return Err("status 小于 1000");
        }


        if status/10%10 >= 2 || status%10 >= 8{  // 第十位数
            report_status = ReportStatus::Report;
        }else if status/100%10 >= 2{  // 验证数量超过了10个,或者 验证通过超过2个
            report_status =  ReportStatus::Pass;
        }else{
            return Ok(ReportStatus::Continue);
        }

        debug::warn!("移除 tx={:?},当前队列剩余 {:?} 个",hex::encode(tx),num);
        <TxVerifyMap>::remove((tx.clone(),mine_tag.clone())); // 移除掉
        if num >0{
            LenOfTxVerify::mutate(|n|*n -= 1);
        }
        return Ok(report_status);
    }

}



//fn public_to_accountid()->AccountId{
//
//}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(
        _source: TransactionSource,
        call: &Self::Call,
    ) -> TransactionValidity {
        let now = <timestamp::Module<T>>::get();
        match call {
            Call::record_tx(block_num,key,account_id,..,tx,status, signature) => {
                debug::info!("############## record_tx :now = {:?} block_num = {:?}##############",now,block_num);

                // check signature (this is expensive so we do it last).
                let signature_valid = &(block_num,account_id,tx,status).using_encoded(|encoded_sign| {
                    key.verify(&encoded_sign, &signature)
                });

                if !signature_valid {
                    debug::info!("................ record_tx 签名验证失败 .....................");
                    return InvalidTransaction::BadProof.into();
                }

                Ok(ValidTransaction {
                    priority: 0,
                    requires: vec![],
                    provides: vec![(block_num,account_id,tx,status).encode()],
                    longevity: TransactionLongevity::max_value(),
                    propagate: true,
                })
            },

            Call::record_fail_fetch(block, key, account,.., tx, err, signature) => {
                debug::info!("############## record_fail_fetch :{:?}##############",now);

                // check signature (this is expensive so we do it last).
                let signature_valid = &(block,account,tx).using_encoded(|encoded_sign| {
                    key.verify(&encoded_sign, &signature)
                });

                if !signature_valid {
                    debug::info!("................ record_fail_fetch 签名验证失败 .....................");
                    return InvalidTransaction::BadProof.into();
                }
                Ok(ValidTransaction {
                    priority: 1,
                    requires: vec![],
                    provides: vec![(block,tx,err,account).encode()], // vec![(now).encode()],
                    longevity: TransactionLongevity::max_value(),
                    propagate: true,
                })},
            _ => InvalidTransaction::Call.into()
        }
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use sp_core::H256;
    use frame_support::{impl_outer_origin, assert_ok, parameter_types};
    use sr_primitives::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, weights::Weight, Perbill,
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
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
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
    type TemplateModule = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

    #[test]
    fn it_works_for_default_value() {
        new_test_ext().execute_with(|| {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(TemplateModule::something(), Some(42));
        });
    }
}
