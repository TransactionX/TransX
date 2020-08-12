
from substrateinterface import SubstrateInterface, Keypair, SubstrateRequestException
from substrateinterface.utils.ss58 import ss58_encode
import threading

# 5CdDzb25wGQ41XnjsX3LHmTAWtJhxGyKVM9LsFZMgdGQzfrY
# 发送方
def run():

	try:
		substrate = SubstrateInterface(
			url="wss://hk.listen.io/",
			address_type=42,
			type_registry_preset='default'
		)

		import time
		time.sleep(1)
		keypair = Keypair.create_from_mnemonic("adult hunt thank force make satisfy saddle pumpkin reject very avoid goat")

		# print("Created address: {}".format(keypair.ss58_address))

		mnemonic = Keypair.generate_mnemonic()

		# 接收方随机生成
		keypair1 = Keypair.create_from_mnemonic(mnemonic, 2)

		# 可以直接调用自己定义的模块  不需要特殊处理
		call = substrate.compose_call(
			call_module='Listen',
			call_function='air_drop',
			call_params={
				# 接收方
				# 'dest': '5GnGKSCitk1QPpMNugtTGX9t6TqzDGvL5BqKzLfHNsLSrwqN',
				'des': keypair1.ss58_address,
				# 'value': 10 * 10**14

			}
		)

		extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)

		try:
			result = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
			print("Extrinsic '{}' sent and included in block '{}'".format(result['extrinsic_hash'], result['block_hash']))
			# substrate.s

		except SubstrateRequestException as e:
			print("Failed to send: {}".format(e))
	except Exception as e:
		print(e)
if __name__ == "__main__":

	# run(substrate)
	for i in range(100):
		run()

	# 多线程去跑没有任何意义 因为有nonce值的存在(nonce防止双花)
	# queue = []
	# for i in range(10):
	# 	queue.append(threading.Thread(target=run, args=()))
	# for i in queue:
	# 	i.start()
	# for i in queue:
	# 	i.join()



