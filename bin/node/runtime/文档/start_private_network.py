import os
from threading import Thread

# 向外透露的端口 alice 9944
def node1():
	os.system(
		r"./substrate --base-path ./db/node1 --chain customspec.json  --port 30334 --ws-port 9944 --rpc-port " +
		r"30339   --alice" + r" --ws-external --rpc-external  --rpc-cors=all --execution=NativeElseWasm -lruntime=debug  > substrate_9944.log 2>&1 &")

# 向外透露接口
def node2():
	os.system(
		r"./substrate --base-path ./db/node4 --chain customspec.json  --port 9951 --ws-port 9945 --rpc-port " +
		r"9952  --bob" + r" --ws-external --rpc-external  --rpc-cors=all --execution=NativeElseWasm -lruntime=debug  > substrate_9945.log 2>&1 &")

# 不向外透露的端口 9911
def node3():
	os.system(
		r"./substrate --base-path ./db/node2 --chain customspec.json  --port 30335 --ws-port 9911 --rpc-port 30338 "
		r" --validator  --eve --execution-offchain-worker=Wasm -lruntime=debug  > substrate_9911.log 2>&1 &")

# 不向外透露的端口
def node4():
	os.system(
		r"./substrate --base-path ./db/node3 --chain customspec.json  --port 30336 --ws-port 9922 --rpc-port 30337 "
		r" --validator  --dave --execution-offchain-worker=Wasm -lruntime=debug > substrate_9922.log 2>&1 &")

# 不向外透露的端口
def node5():
	os.system(
		r"./substrate --base-path ./db/node5 --chain customspec.json  --port 9953 --ws-port 9946 --rpc-port 9954 "
		r" --validator  --ferdie --execution-offchain-worker=Wasm -lruntime=debug   > substrate_9946.log 2>&1 &")

# 不向外透露的端口
def node6():
	os.system(
		r"./substrate --base-path ./db/node6 --chain customspec.json --port 9955 --ws-port 9957 --rpc-port 9956 "
		r" --validator  --charlie --execution-offchain-worker=Wasm   > substrate_9955.log 2>&1 &")
if __name__ == "__main__":

	queue = []
	queue.append(Thread(target=node1, args=()))
	queue.append(Thread(target=node2, args=()))
	queue.append(Thread(target=node3, args=()))
	queue.append(Thread(target=node4, args=()))
	queue.append(Thread(target=node5, args=()))
	queue.append(Thread(target=node6, args=()))

	os.system("rm localspec.json")
	os.system("rm customspec.json")

	# 删除正在运行的进程
	info = os.popen("ps -ef | grep substrate").readlines()
	if info:
		for i in info:
			j = i.split()[1].strip()
			os.system("kill -9 " + j)

	os.system(r"./substrate build-spec --chain=sword > localspec.json ")
	# import re
	#
	# a = None
	# with open(r"localspec.json", "r") as f:
	# 	a = f.read()
	# 	a = re.sub('\"properties\": null,',
	# 			   '\"properties\": {\"tokenDecimals\": 14,\"tokenSymbol\": \"DCAP\"},', a)
	# 	print(a)
	#
	# with open(r"localspec.json", "w") as f:
	# 	f.write(a)

	os.system(r"./substrate build-spec --chain localspec.json --raw > customspec.json")
	import sys
	import getopt
	opts, args = getopt.getopt(sys.argv[1:], '-r', ["remove"])  # 参数要写齐
	for i, j in opts:
		if i == "--remove":
			os.system(r"rm -rf ./db/*")

	for i in queue:
		i.start()
	for i in queue:
		i.join()

