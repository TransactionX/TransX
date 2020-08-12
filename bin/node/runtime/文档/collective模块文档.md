#  collective模块文档
## 主要方法
1. 设置成员 fn set_members(origin, new_members: Vec<T::AccountId>, prime: Option<T::AccountId>)  
思路：
    * 只有root权限才能够操作
    * 
2. 执行 fn execute(origin, proposal: Box<<T as Trait<I>>::Proposal>)   
思路：
    * 操作其他模块里的方法（需要议会权限的方法）
    * 不需要投票 只对议会origin有要求
    * 只有议会成员有执行权限
    
3. 发起议案 fn propose(origin, #[compact] threshold: MemberCount, proposal: Box<<T as Trait<I>>::Proposal>  
    *  发起一个操作其他模块方法的议案
    *  只有议会成员可以发起议案
    *  每一个方法的议案在全网同一时间至多只能一个
    * 对投票有要求的 就用propsal方法 
    *  threshold参数小与2，说明不需要投票，直接执行（这里不算公投）；但是它的origin是投票的形式的
    * 
 4. 关闭议案（这是系统判定没有通过的议案才会去执行） fn close(origin, proposal: T::Hash, #[compact] index: ProposalIndex)   
    *   索引不对不给执行
    *   还没到结束时间不给执行
    *  没有投的票如果自己自己在赞成票里并且在prime里，默认是赞成票，其他情况算是反对
    * 
    ***
    >>>  执行其他模块的方法需要那个方法有议会origin权限
    
    
 
   
    
