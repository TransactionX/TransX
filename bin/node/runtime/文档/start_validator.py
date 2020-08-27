import os

cmd = r"./substrate --chain customspec.json --name WJA_ALIYUN --validator --ws-external --rpc-external  --rpc-cors=all --execution=NativeElseWasm --base-path db1 --node-key-file key --rpc-methods=Unsafe --rpc-port 9933"
result = os.popen(cmd).readlines()
for i in result:
	print(i)
--node-key-file