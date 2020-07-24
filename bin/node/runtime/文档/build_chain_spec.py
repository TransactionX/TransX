import os
file = r"./substrate build-spec --chain=staging > localspec.json"
raw = r"./substrate build-spec --chain localspec.json --raw > customspec.json"
all =[file, raw]
for i in all:
	os.popen(i).readlines()
