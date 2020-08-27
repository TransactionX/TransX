import os
# 删除正在运行的进程
info = os.popen("ps -ef | grep transx").readlines()
if info:
	for i in info:
		j = i.split()[1].strip()
		os.system("kill -9 " + j)
