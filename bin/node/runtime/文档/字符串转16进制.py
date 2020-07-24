import binascii
while True:
	str1 = input("请输入字符串：")
	b_str = bytes(str1, encoding="utf-8")
	result = binascii.b2a_hex(b_str)
	print("16进制：", result)
	print("*"*100)

