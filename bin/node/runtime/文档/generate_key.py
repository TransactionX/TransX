import os
import getopt
import sys


# 根据这个文件生成的密钥对是固定的  wjy可以换成自己的password（这个时候密钥对不一样)
# 生成controller

# 输入密码 这使你的密钥对跟别人的不一样
password = "116000"

# 私钥存储的文件名
file = "wjy.txt"


controller = "./subkey inspect //%s//controller" % password + " >> %s" % file
stash = "./subkey inspect //%s//stash" % password + " >> %s" % file
grand = "./subkey --ed25519 inspect //%s//grand" % password + " >> %s" % file
babe = "./subkey inspect //%s//babe" % password + " >> %s" % file
im_online = "./subkey inspect //%s//im_online" % password + " >> %s" % file
authority = "./subkey inspect //%s//authority" % password + " >> %s" % file
all = [controller, stash, grand, babe, im_online, authority]
for cmd in all:
	result = os.system(cmd)
	with open(file, "a") as f:
		f.write("-------------------------------------------------------------- \n")

with open(file, "a") as f:
	f.write("********************************************************************************************* \n")
