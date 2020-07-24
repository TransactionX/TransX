#  im-online模块文档
## 说明：
* 该模块用于验证验证人是否还在线（是否有心跳）
* 该模块有两种签名方式可以引用， 在其他模块中自定义签名方法可以借鉴这个模块
* 一个session这里是200个出块
* 
## 数据
ReceivedHeartbeats： 已经心跳检测的验证节集合
## 重要方法
* fn heartbeat(
			origin,
			heartbeat: Heartbeat<T::BlockNumber>,
			_signature: <T::AuthorityId as RuntimeAppPublic>::Signature
		)  
    * 方法中不需要进行进一步签名 因为参数输入已经做了要求
    * 查看是否已经存在已经检测队列中， 存在则不要要下一步
    * 查看这个验证索引是否在验证节点的集合中， 存在才能继续
    * 不能重复检测
    * 不能传入不是验证节点的key
    >>> 这个方法不管网络状态如何 均会存入队列
    * 网络状态的结构体为：
        *  '''pub struct OpaqueNetworkState {
	/// PeerId of the local node.
	pub peer_id: OpaquePeerId,
	/// List of addresses the node knows it can be reached as.
	pub external_addresses: Vec<OpaqueMultiaddr>,
}'''
***
* fn offchain_worker(now: T::BlockNumber)  
    * 在块之后运行
    * 自己是验证节点才会去发送心跳信息
## 辅助方法
* pub fn is_online(authority_index: AuthIndex)  
    * 注意 authority_index是u32类型
    * 如果输入的索引值大于当前session模块的验证者的数目 那么返回false
    * 根据数组索引拿到对应的验证id
    * 执行is_online_aux方法
    
* fn is_online_aux(authority_index: AuthIndex, authority: &T::ValidatorId) -> bool  
    * 获取当前的session模块的session索引
    * 根据ReceivedHeartbeats与AuthoredBlocks来判断
    >>> AuthoredBlocks是个啥东西
*  判断一个队列是否在ReceivedHeartbeats中 pub fn received_heartbeat_in_current_session(authority_index: AuthIndex)
     * 获取当前session模块的索引
     * 判断
     >>> 说明 ： 单纯依靠ReceivedHeartbeats来判断是否已经在线是不靠谱的
* fn note_authorship(author: T::ValidatorId)  
     * 改变AuthoredBlocks之中的地址
>>> 可能是在authorship模块中使用

* 发送心跳 pub(crate) fn send_heartbeats(block_number: T::BlockNumber)  
    * 不得小与一定的区块时间
    * 找到本机验证人队列 然后一个一个执行心跳检测
* 找出存在本地的Key队列中的验证索引与id fn local_authority_keys()  
    * let mut local_keys = T::AuthorityId::all(); 这个表达可能是从本地哪里找出验证节点

* 发送心跳最终执行的方法 fn send_single_heartbeat   
     * 获取网络状态 let network_state = sp_io::offchain::network_state()
    * 签名心跳信息 let signature = key.sign(&heartbeat_data.encode()).ok_or(OffchainErr::FailedSigning)?; 
    * 将签名与心跳数据传给ui方法heartbeat
    >>> 貌似签名只能通过这一种方式 在前端执行不了
    ***
    >>> 这个模块的最大特点是offchain-worker去执行上报心跳的方法
  