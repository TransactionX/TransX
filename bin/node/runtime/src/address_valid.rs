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
                    StorageDoubleMap,ensure,weights::Weight};
use frame_system::{self as system,RawOrigin,Origin, ensure_signed,ensure_none, offchain};
use hex;

use pallet_timestamp as timestamp;
use pallet_authority_discovery as authority_discovery;

use sp_runtime::{DispatchResult,DispatchError};
use sp_io::{self, misc::print_utf8 as print_bytes};
use codec::{ Encode,Decode };
use num_traits::float::FloatCore;
use frame_system::offchain::{
    SendTransactionTypes,
    SubmitTransaction,
};
use sp_runtime::{
    AnySignature,MultiSignature,MultiSigner,
    offchain::http, transaction_validity::{
        TransactionValidity, TransactionLongevity, ValidTransaction, InvalidTransaction,TransactionSource,TransactionPriority},
    traits::{CheckedSub,CheckedAdd,Printable,Member,Zero,IdentifyAccount},
    RuntimeAppPublic};
use app_crypto::{sr25519};
//use crate::price_fetch::crypto::AuthorityId;

use crate::register::{self,TokenStatus,TokenInfo,AddressOf,AddressUsedForMiner,ChangeAddressCount,
                      PerMinerUsingAddress, TokenStatusLen, Trait as RegisterTrait};
use crate::offchain_common::*;

/// Our local KeyType.
///
/// For security reasons the offchain worker doesn't have direct access to the keys
/// but only to app-specific subkeys, which are defined and grouped by their `KeyTypeId`.


type Signature = AnySignature;
pub mod address_crypto {
    use super::{TX_KEY_TYPE as KEY_TYPE,AccountIdPublicConver,Signature};
    pub mod app_sr25519 {
        use super::{KEY_TYPE,AccountIdPublicConver};
        use sp_runtime::{MultiSignature,MultiSigner};
        use sp_runtime::traits::{IdentifyAccount};  // AccountIdConversion,
        use sp_core::{crypto::AccountId32 as AccountId};
        use sp_runtime::app_crypto::{app_crypto,sr25519};
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

    app_crypto::with_pair! {
		/// An bridge-eos keypair using sr25519 as its crypto.
		pub type AuthorityPair = app_sr25519::Pair;
	}

    pub type AuthoritySignature = app_sr25519::Signature;

    pub type AuthorityId = app_sr25519::Public;
}


// 请求的查询接口
pub const ADDRESS_FETCHED_CRYPTS: [(&[u8], &[u8]); 1] = [
    (b"localhost",b"http://localhost:8421/v1/account/verify"),
];

enum VerifyStatus {
    Continue,  //  不做任何处理
    Failed,    // 注册失败
    Pass,      // 注册成功
}

/// The module's configuration trait.
pub trait Trait: BaseLocalAuthorityTrait + SendTransactionTypes<Call<Self>> + RegisterTrait{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

//    type SubmitSignedTransaction: offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
//    type SubmitUnsignedTransaction: offchain::SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;

    /// The local AuthorityId
//    type AuthorityId: RuntimeAppPublic + Clone + Parameter+ Into<sr25519::Public> + From<sr25519::Public>+ AccountIdPublicConver<AccountId=Self::AccountId>;

    type Duration: Get<Self::BlockNumber>;  // 对记录的清除周期

    type UnsignedPriority: Get<TransactionPriority>;

}

decl_event!(
  pub enum Event<T> where
    Moment = <T as timestamp::Trait>::Moment,
    AccountId = <T as system::Trait>::AccountId,
    {
        FetchedSuc(AccountId,Moment, Vec<u8>, u64), // 当前address 状态记录事件
  }
);

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as addressValid {
        //记录查询结果,key: T::BlockNumber(1小时的周期数)+T::AccountId, val:(成功次数,2000x 状态码次数,5000x状态码次数).不会删除
       FetchRecord get(fn fetch_record): double_map hasher(blake2_128_concat) T::BlockNumber,hasher(blake2_128_concat) T::AccountId => (u32,u32,u32);

       // 记录失败的,定期全部清除. Vec<FetchFailedOf<T>> 最多保持50个的长度.原本是 linked_map
       pub AddressFetchFailed get(fn fetch_failed): map hasher(blake2_128_concat) T::AccountId => Vec<FetchFailedOf<T>>;
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
        for key_value in <AddressFetchFailed<T>>::iter().into_iter(){ // sym,vec<>, linked_map的作用
            let (key,val) = key_value;
             <AddressFetchFailed<T>>::remove(&key);
            }
        }
        0
    }

    #[weight = 0]
    pub fn record_address(
      origin,
      _block_num:T::BlockNumber,
      account_id: T::AccountId,
      key: <T as BaseLocalAuthorityTrait>::AuthorityId,
      tx:Vec<u8>,
      status: u64,
      _signature: <<T as BaseLocalAuthorityTrait>::AuthorityId as RuntimeAppPublic>::Signature
    ) ->DispatchResult {
      ensure_none(origin)?;
      let now = <timestamp::Module<T>>::get();
      let block_num = <system::Module<T>>::block_number();
      let duration = block_num / T::Duration::get();
//        1000: 表示初始值
//        1001: 表示验证1次，但是请求失败了
//        130x: 终止，3个验证通过 pass
//        1x7x: 终止，7个节点验证不通过,就failed
//        1009: 终止，网络全部失败   todo: 怎么处理？ 目前按照 failed处理
//        1109: pass 处理

       debug::info!("response status={:?}",status);
       let (token_status,register_account,symbol) = <TokenStatus<T>>::get(tx.clone());
       debug::info!("token_status={:?},register_account={:?},symbol={:?}",token_status,register_account,symbol);
       ensure!(<TokenStatus<T>>::contains_key(tx.clone()), "不需要再操作,tx已经从TokenStatus移除");
       debug::info!("获取到了本地服务的返回信息,对状态位操作");
      if status== 0{   // 20000
        // 成功
       <FetchRecord<T>>::mutate(
        duration,account_id.clone(),
        |val|{
            val.0 = val.0.checked_add(1).unwrap();
        });
       <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(100).unwrap());//  通过次数加1,总次数加1
      }else if status == 255{ // query  ./token-query 没有收到返回的消息
         <FetchRecord<T>>::mutate(
            duration,account_id.clone(),
            |val|{
                val.2 = val.2.checked_add(1).unwrap();
//                val
            });
             <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(1).unwrap());// 仅仅对总次数加1
      }else{
        <FetchRecord<T>>::mutate( // 可能是200x 400x
            duration,account_id.clone(),
            |val|{
                val.1 = val.1.checked_add(1).unwrap();
//                val
            });
             <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(10).unwrap()); // 失败次数加1,总次数加1
      }
      Self::address_verify_handle(&tx);
      debug::info!("----上链成功: record_address:{:?}-----", duration);
      Ok(())
    }

    #[weight = 0]
    fn record_fail_verify(
        _origin,
        _block: T::BlockNumber,
        account: T::AccountId,
        key: <T as BaseLocalAuthorityTrait>::AuthorityId,
        tx: Vec<u8>,
        err: Vec<u8>,
        _signature: <<T as BaseLocalAuthorityTrait>::AuthorityId as RuntimeAppPublic>::Signature
        )->DispatchResult{
            ensure_none(_origin)?;
            let now = <timestamp::Module<T>>::get();
            <TokenStatus<T>>::try_mutate(&tx,|val|{
                if val.0 == 0 {
                    debug::error!("当前 TokenStatus 状态为 0.不需要再操作,tx已经从TokenStatus移除");
                    return Err("");
                }
                debug::info!("record_fail_verify 对状态位加 1");
                val.0 = val.0.checked_add(1).unwrap();  // 无应答情况
                return Ok(&tx)}
            )?;
              // 记录获取fetch失败的信息
            let failed_struct = FetchFailedOf::<T> {
                    timestamp: now,
                    tx: tx.clone(),
                    err: err
            };
            let status:u64 = <TokenStatus<T>>::get(tx.clone()).0;
            debug::info!("------验证失败:status={:?},tx={:?}-------",status,hex::encode(&tx));
            Self::address_verify_handle(&tx);
            <AddressFetchFailed<T>>::mutate(account, |fetch_failed| {
            if fetch_failed.len()>50{  // 最多保留50个的长度
                fetch_failed.pop();
            }
            fetch_failed.push(failed_struct)
            });
            debug::info!("------fetch失败记录上链成功:record_fail_verify--------");
            Ok(())
    }


    fn offchain_worker(block: T::BlockNumber) {
        if sp_io::offchain::is_validator() { // 是否是验证人的模式启动
             if let (Some(authority_id),Some(account)) = T::authority_id() {
                debug::info!("-----------adress_valid offchain work------------");
                Self::offchain(block,authority_id,&account);
            }
        }
    } // end of `fn offchain_worker()`
  }
}




impl<T: Trait> Module<T> {
    fn offchain(block_num:T::BlockNumber,key: <T as BaseLocalAuthorityTrait>::AuthorityId, account: &T::AccountId) -> DispatchResult{

        for (remote_src, remote_url) in ADDRESS_FETCHED_CRYPTS.iter() {
            // let (mut symbol, mut token_address,mut tx) = (vec![], vec![], vec![]);
            for (iter_key,value) in <TokenStatus<T>>::iter().into_iter(){
                let (status,verify_account,symbol) = value;   // 注册的账号名, eth
                let tx= iter_key;  // 转账tx
                let tx_hex = hex::encode(&tx);
                let symbol_str = core::str::from_utf8(&symbol).map_err(|e|
                    {debug::info!("symbol解析错误:{:?}",e);
                    "from_utf8 error" })?;
                debug::info!("迭代器获取 symbol = {:?}",&symbol_str);
                debug::info!("迭代器获取 tx = {:?}",tx_hex);

                // post json 构造
                let body = Self::from_register_item(&tx,&symbol,&verify_account).ok_or("from_register_item error");
                let body = match body{
                    Ok(body) => body,
                    Err(e) => {
                        debug::error!("---------{:?}---------",e);
                        Self::call_record_fail_verify(block_num,key.clone(),account,&tx,e)?;
                        return Err(DispatchError::Other("from_register_item error"));
                    }
                };
                // post请求,并结果上链   Self::fetch_address(block_num,key.clone(),account,*remote_src, *remote_url,&tx,body)
                match T::fetch_status(*remote_src,*remote_url,body){
                    Ok(status) => {
                        let tx_hex = hex::encode(&tx);
                        debug::info!("*** fetch ***: {:?}:{:?},{:?}",
                            core::str::from_utf8(remote_src).unwrap(),
                            core::str::from_utf8(remote_url).unwrap(),
                            tx_hex,
                        );
                        Self::call_record_address(block_num, key.clone(), account, &tx, status)?;
                    },
                    Err(e) => {
                        debug::info!("~~~~~~ Error address fetching~~~~~~~~:  {:?}: {:?}",tx_hex,e);
                        Self::call_record_fail_verify(block_num,key.clone(),account,&tx,e)?;
                        // 实现错误信息上链
                    }
                }
                break;
            }
        }
        Ok(())
    }



//    fn fetch_address<'a>(remote_src:&'a [u8], remote_url:&'a [u8], body:Vec<u8>) -> StdResult<u64> {
//        let json = T::fetch_json(remote_url,body)?; // http请求
//        let status = match remote_src {
//            src if src == b"localhost" => Self::fetch_address_from_localhost(json)  // 解析,只需要解析除状态码就行了
//                .map_err(|_| "fetch_address_from_localhost error"),
//            _ => Err("Unknown remote source"),
//        }?;
//
//        Ok(status)
//    }


    fn call_record_address<'a>(
        block_num: T::BlockNumber,
        key: <T as BaseLocalAuthorityTrait>::AuthorityId,
        account_id: &T::AccountId,
        tx: &'a [u8],  //tx
        status: u64
    )-> StrDispatchResult{
        let signature = key.sign(&(block_num,account_id,tx.to_vec(),status).encode()).ok_or("Offchain error: signing failed!")?;
        debug::info!("record_address调用前签名,block_num = {:?},tx={:?}",block_num, hex::encode(tx));
        let call = Call::record_address(
            block_num,
            account_id.clone(),
            key,
            tx.to_vec(),
            status,
            signature
        );

        // Unsigned tx
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|e| {
                debug::info!("{:?}",e);
                "============fetch_price: submit_signed(call) error=================="})?;

        debug::info!("***fetch price over ^_^***");
        Ok(())
    }

    fn call_record_fail_verify<'a>(
        block_num:T::BlockNumber,
        key: <T as BaseLocalAuthorityTrait>::AuthorityId,
        account: &T::AccountId,
        tx: &'a [u8],
        e: &'a str,
    ) -> StrDispatchResult{
        // 实现错误信息上链
        let signature = key.sign(&(block_num,account.clone(),tx.to_vec()).encode()).ok_or("signing failed!")?;
        debug::info!("record_fail_verify调用前签名,block_num = {:?},tx={:?}",block_num, hex::encode(&tx));

        let call = Call::record_fail_verify(block_num,account.clone(),key.clone(),tx.to_vec(), e.as_bytes().to_vec(),signature);
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|_| {
                debug::error!("===record_fail_verify: submit_unsigned_call error===");
                "===record_fail_verify: submit_unsigned_call error==="
            })?;
        debug::info!("+++++++record_fail_verify suc++++++++++++++");
        Ok(())
    }


    // 组成 post json
    fn from_register_item<'a>(tx: &'a [u8], symbol: &'a [u8], verify_account: &T::AccountId) -> Option<Vec<u8>> {
        let token_address = <TokenInfo<T>>::get(verify_account.clone(), symbol.clone()).0;
        if token_address.len() ==0 {
            debug::error!("tokenInfo 获取信息失败,tx = {:?}",hex::encode(tx));
            return None;
        }

        let keys:[&[u8];3] = [b"tx",b"symbol",b"account"];

//        let tx_hex:&str = tx.to_hex();
//        let tx_hex =  hex::encode(tx);   // tx_hex 是 16进制的字符串
//        let tx_vec = &[hex_0x,tx_hex.as_bytes()].concat();

//        let token_address_hex: &str = token_address.to_hex();
//        let token_address_hex = hex::encode(token_address);   // tx_hex 是 16进制的字符串
//        let token_address_vec = &[hex_0x,token_address_hex.as_bytes()].concat(); // token_address_hex.as_bytes();

        let tx_vec= hex_to_u8(tx);
        let token_address_vec = hex_to_u8(&token_address);
        let vals:[&[u8];3] = [&tx_vec,symbol,&token_address_vec];

        let mut json_val = vec![POST_KEYWORD[0]];
        for (i,key) in keys.iter().enumerate(){
            // 形如 "tx": "xxxx",
            json_val.push(POST_KEYWORD[1]); //json_val.push("\"");
            json_val.push(key);
            json_val.push(POST_KEYWORD[2]);  //json_val.push("\":");
            json_val.push(POST_KEYWORD[1]);  // json_val.push("\"");
            json_val.push(vals[i]);
            json_val.push(POST_KEYWORD[1]);  //json_val.push("\"");
            json_val.push(POST_KEYWORD[3]);  //json_val.push(",");
        }
        json_val.pop();    // 移除最后一个 ","
        json_val.push(POST_KEYWORD[4]);    //json_val.push("}");

        let json_vec = json_val.concat().to_vec();
        debug::info!("请求的json:{:?}",core::str::from_utf8(&json_vec).ok()?);

        Some(json_vec)
    }

    // 返回结果 解析
//    fn fetch_address_from_localhost(json_val: JsonValue) -> StdResult<u64> {
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

    fn address_verify_handle(tx: &[u8]) -> StdResult<VerifyStatus>{
        // 是否举报, true 表示举报
//        判断
//        1.十位的数字 >=7 fail ,   记录失败个数
//        2.个位数字 >=8, failed
//        3.百位数字 >=3,pass
        let mut verify_status:VerifyStatus = VerifyStatus::Continue;//初始化一个值
        let (status,register_account,symbol) = <TokenStatus<T>>::get(tx);
        let num = TokenStatusLen::get();
        if status  < 1000{
            debug::error!("=====绑定验证失败:当前的 tx={:?},状态为 {:?},=====",hex::encode(tx),status);
            <TokenStatus<T>>::remove(tx);
            if num > 0{
                TokenStatusLen::mutate(|n|*n -= 1);
            }
            return Err("status 小于 1000");
        }

        debug::info!("当前状态 status={:?}",status);
        let units_digit = status%10;     // 个位数
        let tens_digit = status/10%10;   // 十位数
        let hundreds_digit  = status/100%10;  // 百位数

        if tens_digit >= 6 {    // 十位数
            verify_status = VerifyStatus::Failed;
        }else if hundreds_digit >=3 {   // 百位数大于3
            verify_status = VerifyStatus::Pass;
        } else if units_digit >= 8 {   // 个位数大于8
            // 且十位数
            if hundreds_digit >=2 {
                verify_status = VerifyStatus::Pass;
            }else{
                verify_status = VerifyStatus::Failed;
            }
        } else {
             verify_status = VerifyStatus::Continue;  // 默认值
        }

        let (token_address, address_status, _,_) = <TokenInfo<T>>::get(register_account.clone(), symbol.clone());
        match verify_status{
            VerifyStatus::Failed => {  // 失败
                debug::info!("--注册失败--");
                <TokenStatus<T>>::remove(tx); // 移除掉
                debug::info!("移除 tx={:?},队列剩余:{:?} 个",hex::encode(tx.clone()),num);
                if num > 0{
                    TokenStatusLen::mutate(|n|*n -= 1);
                }
                Self::insert_active_status(register_account.clone(), symbol.clone(),tx,token_address.clone(),register::AddressStatus::inActive);
//                <AddressOf<T>>::mutate(register_account, |v|{
//                    v.push((token_address,register::AddressStatus::inActive,tx.to_vec(),symbol.clone()));
//                });
            }
            VerifyStatus::Pass => {  // 成功
                debug::info!("--注册成功--");
                debug::info!("移除 tx={:?},队列剩余:{:?} 个",hex::encode(tx.clone()),num);
                <TokenStatus<T>>::remove(tx); // 移除掉
                if num >0{
                    TokenStatusLen::mutate(|n|*n -= 1);
                }
                <TokenInfo<T>>::mutate(register_account.clone(), symbol.clone(),
                                       |val|{
                                           val.0 = token_address.clone();
                                           val.1 = register::AddressStatus::active;   // 修改为激活
                                           val.2 = tx.to_vec();
                                       });

                let count =  <ChangeAddressCount<T>>::get((register_account.clone(),symbol.clone()));
                debug::info!("当前已经修改过了 {:?} 次",count);
                Self::insert_active_status(register_account.clone(), symbol.clone(),tx,token_address.clone(),register::AddressStatus::active);
//                if count != 0{
//                    debug::info!("当前已经修改过了 {:?} 次",count);
//                  // 非首次添加
//                    Self::insert_active_status(register_account.clone(), symbol.clone(),tx,token_address.clone(),register::AddressStatus::active);
//                }else{
//                    // 只需要添加
//                    debug::info!("首次添加 {:?} 次",count);
//                    <AddressOf<T>>::mutate(register_account.clone(), |v|{
//                        v.push((token_address.clone(),register::AddressStatus::active,tx.to_vec(),symbol.clone()));
//                    });
//                }
                <AddressUsedForMiner<T>>::insert((symbol.clone(),token_address.clone()),register_account.clone());// (symbol, address) => account_id
                <ChangeAddressCount<T>>::mutate((register_account.clone(),symbol.clone()),|n|*n += 1);// (account_id, symbol) => count
                <PerMinerUsingAddress<T>>::mutate(register_account.clone(),|v|v.push((symbol.clone(),token_address)));//account_id => Vec<(symbol, address)>
            }
            _ => {}
        }
        return Ok(verify_status);
    }

    fn insert_active_status(register_account: T::AccountId, symbol: Vec<u8>,tx:&[u8],token_address: Vec<u8>, active_status: register::AddressStatus){
        let mut register_list = <AddressOf<T>>::get(&register_account);
        let position = register_list.iter().position(|p| p.3 == symbol.clone());  // 注册的列表
        match position{
            Some(x) => {
                debug::info!("---------AddressOf 已经存在了 {:?}---------",hex::encode(tx));
                register_list[x] = (token_address,active_status,tx.to_vec(),symbol);
                <AddressOf<T>>::insert(register_account,register_list);
            },
            None => {
                debug::info!("------- AddressOf 不存在 {:?}-----------",hex::encode(tx));
                <AddressOf<T>>::mutate(register_account, |v|{
                    v.push((token_address,active_status,tx.to_vec(),symbol));
                });
            }
        }

//        if let Some(x) = position{
//            register_list[x] = (token_address,active_status,tx.to_vec(),symbol);
//        }

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
        debug::info!("--------------validate_unsigned time:{:?}--------------------",now);
        match call {
            Call::record_address(block_num,account_id,key,tx,status, signature) => {
                debug::info!("############## record_address : now = {:?},block_num = {:?}##############",now,block_num);

                // check signature (this is expensive so we do it last).
                let signature_valid = &(block_num,account_id,tx,status).using_encoded(|encoded_sign| {
                    key.verify(&encoded_sign, &signature)
                });

                if !signature_valid {
                    debug::error!("................ record_address 签名验证失败 .....................");
                    return InvalidTransaction::BadProof.into();
                }

                Ok(ValidTransaction {
                    priority: T::UnsignedPriority::get(),
                    requires: vec![],
                    provides: vec![(block_num,account_id,tx,status).encode()],
                    longevity: TransactionLongevity::max_value(),
                    propagate: true,
                })
            },

            Call::record_fail_verify(block,account,key,tx,err,signature) => {
                debug::info!("############# record_fail_verify :block={:?},time={:?}##############",block,now);
                // check signature (this is expensive so we do it last).
                let signature_valid = &(block,account,tx).using_encoded(|encoded_sign| {
                    key.verify(&encoded_sign, &signature)
                });
                if !signature_valid {
                    debug::error!("................ record_fail_verify 签名验证失败 .....................");
                    return InvalidTransaction::BadProof.into();
                }
                Ok(ValidTransaction {
                    priority: T::UnsignedPriority::get(),
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
