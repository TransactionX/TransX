use codec::{ Encode,Decode };
use sp_std::{prelude::*,convert::TryInto};
use hex;
use pallet_timestamp as timestamp;
use sp_runtime::RuntimeAppPublic;
use frame_support::{Parameter,debug};
use app_crypto::{sr25519};
use frame_support::traits::{FindAllAuthor};
use frame_system::{self as system};
use sp_core::{crypto::KeyTypeId,offchain::Timestamp};
use pallet_authority_discovery as authority_discovery;
use sp_runtime::{offchain::http};
use alt_serde::{Deserialize, Deserializer};
use crate::register::{self,IsValidtorOcw, Trait as RegisterTrait};
use frame_support::{StorageMap}; // 含有get
pub const TX_KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofty");
pub const VERIFY_STATUS: &[u8] = b"verify_status";  // 验证的返回状态

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct ResponseStatus {
    verify_status: u64,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(de)?;
    Ok(s.as_bytes().to_vec())
}

// post json中常用的关键字符
pub(crate) const POST_KEYWORD:[&[u8]; 5] = [
    b"{",     // {
    b"\"",   // "
    b"\":",  // ":
    b",",    // ,
    b"}"      // }
];


#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode,Clone)]
pub struct FetchFailed<Moment>{
    // 失败的请求
    pub timestamp:Moment,
    pub tx:Vec<u8>,
    pub err:Vec<u8>
}


pub type FetchFailedOf<T> = FetchFailed<<T as timestamp::Trait>::Moment>;

pub type BlockNumberOf<T> = <T as system::Trait>::BlockNumber;  // u32
pub type StdResult<T> = core::result::Result<T, &'static str>;
// 为了兼容返回为空的情况
pub type StrDispatchResult = core::result::Result<(), &'static str>;

pub fn vecchars_to_vecbytes <I: IntoIterator<Item = char> + Clone>(it: &I) -> Vec<u8> {
    it.clone().into_iter().map(|c| c as u8).collect::<_>()
}

pub fn int_covert_str(inner: u64) ->Vec<u8>{
    let mut x:u32 = 0;    //位数
    let mut s :Vec<&str> = vec![]; //保存字符串
    loop {
        let r = inner / ((10 as u64).pow(x));
        if r == 0 {
            s.reverse();
            return  s.join("").as_bytes().to_vec();
        }
        let r = r % 10;
        s.push(num_to_char(r));
        x += 1;
    }
}

pub fn num_to_char<'a>(n:u64)->&'a str{
    if n > 10{return ""}
    match n{
        0=>"0",
        1=>"1",
        2=>"2",
        3=>"3",
        4=>"4",
        5=>"5",
        6=>"6",
        7=>"7",
        8=>"8",
        9=>"9",
        _ => {""},
    }
}

pub fn hex_to_u8<'a>(param: &'a [u8]) -> Vec<u8>{
    // 将 param  首先转化为 16进制字符串,然后加上0x  . 将tx等16进制保持字符串传递
    // 例如: param的十六进制形式为0x1122,变为"0x"+"1122"的字符串,然后编码为&[u8]
    let hex_0x = "0x".as_bytes();
    let tx_hex =  hex::encode(param);   // tx_hex 是 16进制的字符串
    let tx_vec = &[hex_0x,tx_hex.as_bytes()].concat();

    return tx_vec.to_vec();;
}


pub trait AccountIdPublicConver{
    type AccountId;
    fn into_account32(self)->Self::AccountId; // 转化为accountId
}

pub trait BaseLocalAuthorityTrait: timestamp::Trait + system::Trait + RegisterTrait{
    type AuthorityId: RuntimeAppPublic + Clone + Parameter+ Into<sr25519::Public> + From<sr25519::Public>+ AccountIdPublicConver<AccountId=Self::AccountId>;
    type FindAllAuthor: FindAllAuthor<Self::AccountId>;
    fn authority_id() -> (Option<Self::AuthorityId>,Option<Self::AccountId>){
        //通过本地化的密钥类型查找此应用程序可访问的所有本地密钥。
        // 然后遍历当前存储在chain上的所有ValidatorList，并根据本地键列表检查它们，直到找到一个匹配，否则返回None。
        let validators = Self::FindAllAuthor::find_all_author();  // AccountId

        // let authorities = <authority_discovery::Module<Self>>::authorities().iter().map(
        //     |i| { // (*i).clone().into()
        //         (*i).clone().into()
        //     }
        // ).collect::<Vec<sr25519::Public>>();
        let key_id = core::str::from_utf8(&Self::AuthorityId::ID.0).unwrap();
        debug::info!("当前的所有验证节点,validators keys: {:?}",validators);
        debug::info!("当前的节点 keytypeId: {:?}",key_id);

        for i in Self::AuthorityId::all().iter(){   // 本地的账号
            let authority: Self::AuthorityId = (*i).clone();
            let  authority_sr25519: sr25519::Public = authority.clone().into();
            let s: Self::AccountId= authority.clone().into_account32();
            debug::info!("本地账号信息:{:?}",s);
            if validators.contains(&s) && <IsValidtorOcw<Self>>::get(&s) == true{
                debug::info!("找到了本地账号: {:?}",s);
                return (Some(authority),Some(s));
            }
        }
        return (None,None);
    }

    fn fetch_json<'a>(remote_url: &'a [u8], body:Vec<u8>) -> StdResult<Vec<u8>>{  // http post
        let remote_url_str = core::str::from_utf8(remote_url)
            .map_err(|_| "Error in converting remote_url to string")?;

        let now = <timestamp::Module<Self>>::get();
        let deadline:u64 = now.try_into().
            map_err(|_|"An error occurred when moment was converted to usize")?  // usize类型
            .try_into().map_err(|_|"An error occurred when usize was converted to u64")?;
        let deadline = Timestamp::from_unix_millis(deadline+20000); // 等待最多10s
        let body = sp_std::str::from_utf8(&body).map_err(|e|"symbol from utf8 to str failed")?;
        let mut new_reuest = http::Request::post(remote_url_str,vec![body]);
        new_reuest.deadline = Some(deadline);
        let pending = new_reuest.send()
            .map_err(|_| "Error in sending http POST request")?;

        let http_result = pending.try_wait(deadline)
            .map_err(|_| "Error in waiting http response back")?;
        let response = http_result.map_err(|_| "Error in waiting http_result convert response" )?;

        if response.code != 200 {
            debug::warn!("Unexpected status code: {}", response.code);
            let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();
            debug::info!("error body:{:?}", core::str::from_utf8(&json_result).unwrap());
            return Err("Non-200 status code returned from http request");
        }

        let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

        // Print out the whole JSON blob
        debug::info!("---response---{:?}",&core::str::from_utf8(&json_result).unwrap());
//        print_bytes(&json_result);

//        let json_val: JsonValue = simple_json::parse_json(
//            &core::str::from_utf8(&json_result).unwrap())
//            .map_err(|_| "JSON parsing error")?;

        Ok(json_result)
    }

    fn fetch_parse(resp_bytes: Vec<u8>) -> StdResult<u64> {
        let resp_str = core::str::from_utf8(&resp_bytes).map_err(|_| "Error in fetch_parse")?;
        // Print out our fetched JSON string
        debug::info!("{}", resp_str);

        // Deserializing JSON to struct, thanks to `serde` and `serde_derive`
        let status: ResponseStatus =
            serde_json::from_str(&resp_str).map_err(|_| "convert to ResponseStatus failed")?;

        debug::info!("获取到的状态是:{:?}", status.verify_status);
        Ok(status.verify_status)
    }

    fn fetch_status<'a>(
        remote_src: &'a [u8],
        remote_url: &'a [u8],
        body:Vec<u8>
    ) -> StdResult<u64> {
        let json = Self::fetch_json(remote_url, body)?; // http请求
        let status = match remote_src {
            src if src == b"localhost" => Self::fetch_parse(json)  // 解析
                .map_err(|_| "fetch_price_from_localhost error"),
            _ => Err("Unknown remote source"),
        }?;
        Ok(status)
    }
}








