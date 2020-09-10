
use sc_chain_spec::ChainSpecExtension;
use serde_json::map::Map;
use std::convert::TryInto;
use sp_core::{Pair, Public, crypto::UncheckedInto, sr25519};
use serde::{Serialize, Deserialize};
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig, CouncilConfig,
	DemocracyConfig,GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus,
	StakingConfig, ElectionsConfig, IndicesConfig, SocietyConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, wasm_binary_unwrap, MineConfig, TransxCommiteeConfig, GenericAssetConfig,
};
use node_runtime::Block;
use node_runtime::constants::currency::*;
use sc_service::ChainType;
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Transx core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<
	GenesisConfig,
	Extensions,
>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	//  初始化验证人是否给两个就行
	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![

		( //5H13XJ1vYup8MJj2KpiEUjrdvUT1X9Pkp2oLUwopK7Djttt3
		hex!["36df07c6972a58f8b13837019a119b74373e17ae30f654c288446564cc625055"].into(),
		// 5FNQSa4K7k5XvD768QQjwykbjqF3eK5PfyPKAjNBzQ6hJqHq
		hex!["2a69db9b03837c9af7ed9911192ac0086c54c686b6c0f715ae7551875b97a80b"].into(),
		// 5GiC6YDxXEggvdcS8cMcNgwRkHBtUmjkAnR5dwWLGvTkrEgC
		hex!["04d2f61f6b1d345ec20da1d621eec61608c4dcc9ae781a61242bfaa35fdb2e0c"].unchecked_into(),
		// 5ExtcN2V7W4BSZBVZtc893tFuahKRxoNRauZytnjEc35mF6a
		hex!["0c186230ee3f811c4dee4ffd726493ada0bcc711b8e33de3eb113399fad5b479"].unchecked_into(),
		// 5ExtcN2V7W4BSZBVZtc893tFuahKRxoNRauZytnjEc35mF6a
		hex!["6af7ad3384a8f993380c598fed1454cb48e407351600a8eb4c3c3422c76a4375"].unchecked_into(),
		// 5ExtcN2V7W4BSZBVZtc893tFuahKRxoNRauZytnjEc35mF6a
		hex!["267469f51b398a619b8451a789564d65a3256955d72242f8ae8354a989e65e11"].unchecked_into(),
	),(
		// 5HgR7eXnMLjJUmD4udaZPebnaS23xtkBcMDjrfxxKjCnnNAM
		hex!["44f68ed065555550c5f6e46a8883d12b479ab9a969143b9833e4e4d8aae21d3e"].into(),
		// 5E2fPwLypdLEfZfGsGiLnMBXwMJwGrDjUG5PGdV25uricyph
		hex!["4cb14cb42837abb6fbe170b4bb7ba312a999df20ed622d796c761fa9cc97e218"].into(),
		// 5DCQWGgi1u4Prhu6uQr2CiQTvEoo2eayenk7oJ2Xmmci4vPZ
		hex!["c4a861a234f8339835245c15f28ea9c20b372bb9a816c4672d04bcbd73e939cc"].unchecked_into(),
		// 5H4NcpqS7LAAfKt8D77WgNL3ddUTFxT1axbpmnXD5JEDcfdH
		hex!["f82b4d6b7b35fed3832f1ca7c962914d8951e5fb4ffffdd43d51e81c16dff06c"].unchecked_into(),
		// 5H4NcpqS7LAAfKt8D77WgNL3ddUTFxT1axbpmnXD5JEDcfdH
		hex!["2884429144a7db339a6663c3ea3ab96b4f36debbe1d0f722fc10764b4cbae564"].unchecked_into(),
		// 5H4NcpqS7LAAfKt8D77WgNL3ddUTFxT1axbpmnXD5JEDcfdH
		hex!["4a027258265d673e28b080980ad5793832b4d87a7b2f346ba04debd33dab9d6d"].unchecked_into(),
	)];

	// 用hex就可以解决(这个要用自定义的)
	let root_key: AccountId = hex!["8e87d1c0b7588c8038d83317ef95c2be5449f500af057a174f14b43010a61e69"].into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	let technical_accounts: Vec<AccountId> = vec![
		hex!["5853712bf239d59accc1e9ce56f257736dde78192bb7ae1a616482445ea39e44"].into(),
		hex!["9efd40ccf6d27dc17a68e13f89414fd6ce443b9d4df37db68c7cf5c561b74f55"].into(),
		hex!["a0921eeb61d94111a26536f0218a53055e5c19d970e4f9cf51167437adf86860"].into(),
		hex!["50902010325da517dde82f459f85080923a486ef911b74028889ca5465b1c25f"].into(),
		hex!["a62e824a6d125a0d2c6657f48844a70911c7d5a24ea4e2db5a5582f06a67cb15"].into(),
	];

	testnet_genesis(
		initial_authorities,
		root_key,
		Some(endowed_accounts),
		false,
		technical_accounts,
	)
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {

	let boot_nodes = vec![
		// 自己的阿里云服务器
		String::from("/ip4/47.106.196.14/tcp/30333/p2p/12D3KooWMutoAuM5TrpcsiVUSfnCEcKsbAPZr9PmZSqwiFsmD6MX").try_into().unwrap(),

	];

	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(),"DCAP".into());
	properties.insert("tokenDecimals".into(),14.into());
	ChainSpec::from_genesis(
		"Transx Testnet",
		"Transx_testnet",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Staging telemetry url is valid; qed")),
		Some("transx"),
		Some(properties),
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	enable_println: bool,
	technical_accounts: Vec<AccountId>,
) -> GenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap();

	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 10_0000 * DOLLARS;
	const STASH: Balance = 100 * DOLLARS;
	const GenericAssetBalance: Balance = 10000 * DOLLARS;

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {

		balances: endowed_accounts.iter().cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH))).chain(
			technical_accounts.iter().cloned().map(|z| (z.clone(), STASH))

		)
				.collect(),
		}),

		pallet_indices: Some(IndicesConfig {
			indices: vec![],
		}),

		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
				))
			}).collect::<Vec<_>>(),
		}),

		pallet_staking: Some(StakingConfig {
			validator_count: 32u32, // 这里是验证节点个数的上限
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			invulnerables: vec![],
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),

		pallet_democracy: Some(DemocracyConfig::default()),

		// 在这里初始化议会成员
		pallet_elections_phragmen: Some(ElectionsConfig {
			members: technical_accounts.iter().cloned().map(|z| (z.clone(), STASH)).collect(),
		}),

		pallet_collective_Instance1: Some(CouncilConfig::default()),

		pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
			members: technical_accounts.clone(),

			phantom: Default::default(),
		}),

		// 金会(这里要用自定义的)
		pallet_collective_Instance3: Some(TransxCommiteeConfig {
			members: technical_accounts.clone(),
			phantom: Default::default(),
		}),

		generic_asset: Some(GenericAssetConfig{
			next_asset_id: 3u32,  // 下一次创建的资产的id
			staking_asset_id:0u32,  //
			spending_asset_id:1u32, //
			assets:vec![0,1,2],  // 初始化资产
			initial_balance: GenericAssetBalance,  // 每种资产每个人初始化金额

			endowed_accounts: technical_accounts,  // 每种资产里面初始配置人员

		}
		),

		pallet_contracts: Some(ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				enable_println, // this should only be enabled on development chains
				..Default::default()
			},
		}),
		pallet_sudo: Some(SudoConfig {
			key: root_key,
		}),

		pallet_babe: Some(BabeConfig {
			authorities: vec![],
		}),
		pallet_im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_treasury: Some(Default::default()),
		pallet_society: Some(SocietyConfig {
			members: endowed_accounts.iter()
						// .take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			pot: 0,
			max_members: 999,
		}),


		mine: Some(MineConfig{
			founders: vec![hex!["8e87d1c0b7588c8038d83317ef95c2be5449f500af057a174f14b43010a61e69"].into(),]
		}),

		pallet_vesting: Some(Default::default()),
	}
}
