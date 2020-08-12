#  nicks模块操作文档

## 模块主要说明
1. 名字字段长短有制约
2. 这里的名字与前端显示的诸如“Alice、Bob"等是不同的。这里是全网名字，Alice这些是本地名称
3. 根据nicks名称来转账的，在balance与generic-asset模块里，方法是xxx_by_name.
>>> 要根据nicks名来操作， 必须是先存在nicks名(先绑定)

## 模块主要方法
1. 命名 fn set_name(origin, name: Vec<u8>)   
思路:   
  * 任何人都可以命名
  * 名称全网唯一
  * 一个人只能命名一次
  * 命名需要抵押一定金额

2. 强制kill掉别人的名称  kill_name(origin, target: <T::Lookup as StaticLookup>::Source)  
思路:
  * root才有权限
  * 名称存在则删除
  * 惩罚掉目标人命名时候的抵押金额

3. 强制给某人命名  fn force_name(origin, target: <T::Lookup as StaticLookup>::Source, name: Vec<u8>)  
思路:
  * root权限才能够执行
  * 如果目标人已经有名字，那么保留当前的抵押不变; 如果没有，不需要抵押(抵押金额是0)
  * 如果name已经被非目标人占用， 那么归还占用的人的抵押

