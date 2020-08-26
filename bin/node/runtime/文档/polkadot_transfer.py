from substrateinterface import SubstrateInterface, Keypair, SubstrateRequestException
from substrateinterface.utils.ss58 import ss58_encode
import threading,os,time,binascii,csv,json
from decimal import *
# from TransX_Interface.Base.logs_print import Logger

class TransX_CreateAccount():

	def __init__(self, seed_secret, url):
		self.seed_secret = seed_secret
		self.url = url

	def create_MnemonicKey(self):
		substrate = SubstrateInterface(
			url= self.url,
			address_type=42,
			type_registry_preset='polkadot'
		)

		# keypair=Keypair.create_from_seed("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a") #alice账号

		file_name = open('account_info.csv', encoding='utf-8')

		csv_lines = csv.reader(file_name)
		data = []
		for row in csv_lines:
			if csv_lines.line_num == 1:
				continue
			else:
				data.append(row)

		print ("data", data)

		for info in data:
			print("info", info)
			account = info[0]
			keypair = Keypair.create_from_mnemonic(self.seed_secret)
			# keypair = Keypair.create_from_mnemonic(info[1])
			amount = info[1]

			print("type", type(amount))

			call = substrate.compose_call(
				call_module='Balances',
				call_function='transfer',
				call_params={
					# 接收方
					'dest': account,
					'value': Decimal(amount)*10**10
				}
			)

			extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)
			try:
				result = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
				print(result)
				print ("Extrinsic '{}' sent and included in block '{}'".format(result['extrinsic_hash'], result['block_hash']))
			except SubstrateRequestException as e:
				print(e)
				with open("faild_info.txt", "a") as f:

					f.write(account + "\n")

			time.sleep(5)



if __name__ == "__main__":

	# 您的私钥
	seed_secret = "scrub voice eyebrow gate onion engage visit easily wheat rookie soon renew"

	# 节点地址
	url = "ws://47.108.199.133:9944"
	# url = "wss://rpc.polkadot.io"

	instance = TransX_CreateAccount(seed_secret, url)
	instance.create_MnemonicKey()



