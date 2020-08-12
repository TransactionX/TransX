#democracy模块文档
## 说明
1. propose提交的议案在前端的"民主权利"
2. referendum: 全民公投
3. Conviction: 信念
4. 这个模块提议的只有每个模块之中需要root权限的方法
5. 投票的信息只有： 赞成或是反对+信念
6. 公投的权限是最大的
7. proposal_hash只是起到标识符的作用，没有进行直接的decode等等
8. 信念应该就是等同于你在该投票之中的话语权， 公投一旦进入队列 即锁仓
9. 一个信念的时间长短是固定的
## 主要方法
1. 提交议案 fn propose(origin,
			proposal_hash: T::Hash,
			#[compact] value: BalanceOf<T>
		)  
思路：
    * 这个与collective模块不同， 不是直接操作其他模块中的方法
    * 任何人都可以发起议案
    * 发起议案需要抵押一定金额， 可以抵押任意金额，但是有最小额要求
    * 公投数是要统计的
    * 公投有索引（问题：重复了怎么办呢？？？？）
    * DepositOf可能是用来存储抵押的这些人的， 为了方便计算这个议案的总金额
    
    
 2. 其他人支持提交过的议案 相当于附议 n second(origin, #[compact] proposal: PropIndex)  
 思路：
    *  根据索引找到议案，所以索引一定要在
    * 这个议案的发起人抵押多少金额，就需要自己抵押多少金额
    * 把自己加入到 DepositOf中
    
 3. 投票 fn vote(origin,
			ref_index: ReferendumIndex,
			vote: Vote
		)  
    *  目前前端显示这个Vote没有Conv参数
    * 根据公投索引去投票 而不是议案索引
    * VoteOf保证了这个人投唯一票 如果已经存在 说明它已经投票（可以查看这个人在这个公投里的投票信息Vote， Vote包含赞成或是反对，还有信念
    * VoteFor是查看该公投有多少个人投了票
    * 
 4. 代理投票
    * 找到我代理的那个人，如果没有找到，就不执行
    * 我必须作为过代理方，而且代理权限正在激活状态
    * 相当于他自己去投票，跟我i无关
    
 5. 紧急取消公投 fn emergency_cancel(origin, ref_index: ReferendumIndex)  
    *  技术委员会2/3投票通过
    * 议案必须存在
    * 议案没有在取消队列
    
    
    ***
 6. fn external_propose(origin, proposal_hash: T::Hash)  
    * 可能是绿色通道  为了快速提议案
    * 议会成员1/2通过
    * NextExternal 里面没有议案
    * 该提案如果存在黑名单 必须是过期后才能再次提
 7. fn external_propose_majority  
    * 比上面方法的权限要大 
    * 需要议会成员3/4通过
    * 一旦通过直接覆盖
 8. fn external_propose_default(origin, proposal_hash: T::Hash)  
    * 需要议会成员百分之百通过
 >>> 以上三个方法实现的功能基本相同 但是第二个与第3个怎么区分权限大小呢 同时可以覆盖 NextExternal 
 
 ***
 
 9. 快速通道 fn fast_track(origin,
			proposal_hash: T::Hash,
			voting_period: T::BlockNumber,
			delay: T::BlockNumber
		)  
    *  NextExternal 必须存在（为什么一定要存在呢？？？？）
    * 技术委员会2/3通过
    * 议案权限不等与SuperMajorityApprove
    * 输入的议案hash必须与NextExternal 中的相同
    * 把这个议案注入到公投队列ReferendumInfoOf（存在这里面就说明当前公投处于激活状态）
    * 议案有最短时间要求 自己输入的时间太短 就用默认的
    * 
 10. fn veto_external(origin, proposal_hash: T::Hash) 
    * 技术委员会成员才能执行
    * 这个议案必须在NextExternal里面
    * 如果自己该议案的黑名单中 不能再操作
    * 把该议案放入到黑名单中（有什么意义？？？？？）
    
 11. 取消公投  fn cancel_referendum(origin, #[compact] ref_index: ReferendumIndex) {
			ensure_root(origin)?;
			Self::clear_referendum(ref_index);
		}  
    * 只有root权限才能执行
    
12. 从dispatch队列中删除 fn cancel_queued(origin, which: ReferendumIndex)  
    * 需要root权限
    * 
13. 激活代理 fn activate_proxy(origin, proxy: T::AccountId)
    * 已经激活的不再激活
    * 必须是存在open里面
    * open里面的账号必须是自己
 14. 关掉代理 fn close_proxy(origin)  
     * 应该是关掉自己的代理人
 15. 自己代理的改成未激活状态 fn deactivate_proxy(origin, proxy: T::AccountId)  
     * 自己是代理人
     * 该代理目前状态是激活
     
 16. 委托 pub fn delegate(origin, to: T::AccountId, conviction: Conviction)  
     * 加入委托队列
     * 自己extend_lock
     * 把自己从Locks中删除
     
 17. 取消委托
     * 自己存在于委托队列
     * 从委托队列中删除
     * 按照信念多少来重新锁仓
     * 
     
 18. 清除掉公投 fn clear_public_proposals(origin)  
	
     * root权限
    
 19. 不再锁仓 fn unlock(origin, target: T::AccountId)
 
 
 20. 设置代理 fn open_proxy(origin, target: T::AccountId) 
 
 21. fn note_preimage(origin, encoded_proposal: Vec<u8>)
     * 向Preimages里添加信息
     * 如果已经存在不需要添加
     * 根据encoded_proposal长度来决定抵押金额
22. fn note_imminent_preimage(origin, encoded_proposal: Vec<u8>)  
     * 把处于DispatchQueue中的hash进行注册之类的
     
23. fn reap_preimage(origin, proposal_hash: T::Hash)  
    * 删除掉处于Preimages中的信息
    * 不能过早删除
    * 存在紧急队列的不能删除
    * 需要抵押转账等操作
    
 24. 发起又一次公投 fn launch_next(now: T::BlockNumber)
 
 25. 执行提议 fn enact_proposal(proposal_hash: T::Hash, index: ReferendumIndex)  
     * 议案必须存在于Preimages中
     * 成功操作归还抵押
     * decode失败则惩罚抵押
26. 处理到了时间的公投 fn bake_referendum
    * 不管通不通过， 均要从队列中清除掉
    * 通过的提议赞成的人需要按照信念长短来锁仓
    * delay是0直接丢到执行方法里
    * delay不为0的要添加到dispatch队列中
27. fn begin_block(now: T::BlockNumber)
    * 检查发起公投的时间，如果到，则发起公投（将提议添加到队列）
    * 检查公投队列中是否有已经过了时间的提议，如果有，则处理
    * 执行DispatchQueue队列中的提议
    
28. 直接添加提议到公投队列 pub fn internal_start_referendum

29. 直接从公投队列中清除提议 pub fn internal_cancel_referendum(ref_index: ReferendumIndex)  

30. 发起公共提案（将议案升级为公共提案） fn launch_public(now: T::BlockNumber)  
    *  归还附议的人的抵押
    * 将提议提升为公共提案

    
## 问题
 1. 公投结束怎么去奖励呢
 2. 进入公投行列的提议怎么奖励
 3. 这些等待进入公投的提议没有优先权限吗
 
## 操作指南
 1. 目前的方法均是使用proposal_hash,这是很坑爹的，意味着要用这个模块还需要做一些别的工作
    
     
     
     
   
     
     
    
    
    
    
    
    
    
		
		
		
