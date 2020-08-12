## settings/developer
配置json格式
```json
{


  "Moment": "u64",

    "TokenTotalPower": {
    "btc_amount_power": "u64",
    "btc_count_power": "u64",
    "eth_amount_power": "u64",
    "eth_count_power": "u64",
    "usdt_amount_power": "u64",
    "usdt_count_power": "u64",
    "eos_amount_power": "u64",
    "eos_count_power": "u64"
  },

  "ReportModuleAmount": {
    "_enum": {
     "ReportReserveAmount": "Balance",
    "ReportReward": "Balance",
    "PunishmentAmount": "Balance",
    "CouncilReward": "Balance",
    "CancelReportSlash": "Balance"
}

},

  "ReportModuleTime": {
    "_enum": {
    "ProposalExpireTime": "BlockNumber",
    "RewardDuration": "BlockNumber"

}

},

  "AssetTime": {
  "_enum": {
  "Days": "u32",
  "Minutes": "u32",
  "Hours": "u32"
  }
  },

  "AssetChangeableParams": {
  "_enum": {
  "MintPledge": "Balance",
  "BurnPledge": "Balance",
  "MintMinAmount": "Balance",
  "BurnMinAmount": "Balance",
  "MintExistsHowLong": "AssetTime",
  "MintPeriod": "AssetTime",
  "BurnExistsHowLong": "AssetTime",
  "MaxLenOfMint": "u32",
  "MaxLenOfBurn": "u32"
  }
  },
  "MR": {
    "_enum": {
      "Btc": "Permill",
      "Eth": "Permill",
      "Usdt": "Permill",
      "Eos": "Permill",
      "Ecap": "Permill"
      }

    },
  "TLC": {
      "_enum": {
          "BtcCount": "u64",
          "EthCount": "u64",
          "UsdtCount": "u64",
          "EosCount": "u64",
          "EcapCount": "u64"
      }
   },

   "LC": {
          "_enum": {
              "BtcCount": "u64",
              "EthCount": "u64",
              "UsdtCount": "u64",
              "EosCount": "u64",
              "EcapCount": "u64"
          }
       },

  "LA": {
    "_enum": {
     "BtcAmount": "u64",
     "EthAmount": "u64",
     "UsdtAmount": "u64",
     "EosAmount": "u64",
     "EcapAmount": "u64"
    }
   },

  "TLA": {
    "_enum": {
      "BtcAmount": "u64",
     "EthAmount": "u64",
     "UsdtAmount": "u64",
     "EosAmount": "u64",
     "EcapAmount": "u64"
}

},

  "MLA": {
    "_enum": {
      "BtcAmount": "u64",
     "EthAmount": "u64",
     "UsdtAmount": "u64",
     "EosAmount": "u64",
     "EcapAmount": "u64"
}

},



  "MinerInfo": {
    "hardware_id": "Vec<u8>",
    "father_address": "Option<AccountId>",
    "grandpa_address": "Option<AccountId>",
    "register_time": "Moment",
    "machine_state": "Vec<u8>",
    "machine_owner": "AccountId"
  },

  "BurnInfo": {
    "start_block": "BlockNumber",
    "burn_man": "AccountId",
    "asset_id": "AssetId",
    "amount": "Balance",
    "foundation_tag_man": "Option<AccountId>"
  },

  "MintVote": {
    "start_block": "BlockNumber",
    "pass_block": "Option<BlockNumber>",
    "mint_block": "Option<BlockNumber>",
    "mint_man": "AccountId",
    "asset_id": "AssetId",
    "amount": "Balance",
    "approve_list": "Vec<AccountId>",
    "reject_list": "Vec<AccountId>",
    "technical_reject": "Option<AccountId>"
  },

  "AssetVote": {
    "_enum": [
      "Approve",
      "Reject"
    ]
  },

  "conviction": {
    "_enum": [
      "None",
      "Locked1x",
      "Locked2x",
      "Locked3x",
      "Locked4x",
      "Locked5x",
      "Locked6x"
    ]
  },

  "MineIndex": "u64",

  "BlockNumberOf": "u32",

  "MineTag": {
    "_enum": [
      "CLIENT",
      "WALLET"
    ]
  },

  "AddressStatus": {
    "_enum": [
      "active",
      "inActive"
    ]
  },

  "MinerStatus": {
    "_enum": [
      "Success",
      "Invalid",
      "Slashed"
    ]
  },

    "Count": "u64",

  "OwnerMineRecordItem": {
    "mine_tag": "MineTag",
    "mine_count": "u16",
    "timestamp": "Moment",
    "blocknum": "BlockNumber",
    "miner_address": "AccountId",
    "from_address": "Vec<u8>",
    "to_address": "Vec<u8>",
    "symbol": "Vec<u8>",
    "blockchain": "Vec<u8>",
    "tx": "Vec<u8>",
    "usdt_amount": "u64",
    "sym_amount": "Vec<u8>",
    "pcount_workforce": "u64",
    "decimal":"u32",
    "pamount_workforce": "u64",
    "reward": "Balance",
    "grandpa_reward": "Balance",
    "father_reward": "Balance"
  },

  "USD": "u64",

  "OwnerMineWorkForce": {
    "mine_cnt": "u64",
    "usdt_nums": "u64",
    "amount_work_force": "u64",
    "count_work_force": "u64",
    "settle_blocknumber": "u32"
  },

  "PriceInfo": {
    "dollars": "u64",
    "account": "AccountId",
    "url": "Vec<u8>"
  },

  "PriceFailed": {
    "account": "AccountId",
    "sym": "Vec<u8>",
    "errinfo": "Vec<u8>"
  },

  "PriceFailedOf": "PriceFailed",

  "VoteInfo": {
    "start_vote_block": "BlockNumber",
    "symbol": "Vec<u8>",
    "tx": "Vec<u8>",
    "reporter": "AccountId",
    "report_reason": "Vec<u8>",
    "illegal_man": "AccountId",
    "transaction_amount": "Vec<u8>",
    "usdt_amount": "Balance",
    "decimals": "u32",
    "approve_mans": "Vec<AccountId>",
    "reject_mans": "Vec<AccountId>"
  },

  "PowerInfoItem": {
    "total_power": "u64",
    "total_count": "u64",
    "count_power": "u64",
    "total_amount": "u64",
    "amount_power": "u64",
    "block_number": "BlockNumber"
  },

  "TokenPowerInfoItem": {
    "btc_total_power": "u64",
    "btc_total_count": "u64",
    "btc_count_power": "u64",
    "btc_total_amount": "u64",
    "btc_amount_power": "u64",
    "eth_total_power": "u64",
    "eth_total_count": "u64",
    "eth_count_power": "u64",
    "eth_total_amount": "u64",
    "eth_amount_power": "u64",
    "eos_total_power": "u64",
    "eos_total_count": "u64",
    "eos_count_power": "u64",
    "eos_total_amount": "u64",
    "eos_amount_power": "u64",
    "usdt_total_power": "u64",
    "usdt_total_count": "u64",
    "usdt_count_power": "u64",
    "usdt_total_amount": "u64",
    "usdt_amount_power": "u64",
    
    "ecap_total_power": "u64",
    "ecap_total_count": "u64",
    "ecap_count_power": "u64",
    "ecap_total_amount": "u64",
    "ecap_amount_power": "u64",
    "block_number": "u64"
  },

  "MinerPowerInfoItem": {
    "miner_id": "AccountId",
    "total_power": "u64",
    "total_count": "u64",
    "count_power": "u64",
    "total_amount": "u64",
    "amount_power": "u64",
    "btc_power": "u64",
    "btc_count": "u64",
    "btc_count_power": "u64",
    "btc_amount": "u64",
    "btc_amount_power": "u64",
    "eth_power": "u64",
    "eth_count": "u64",
    "eth_count_power": "u64",
    "eth_amount": "u64",
    "eth_amount_power": "u64",
    "eos_power": "u64",
    "eos_count": "u64",
    "eos_count_power": "u64",
    "eos_amount": "u64",
    "eos_amount_power": "u64",
    "usdt_power": "u64",
    "usdt_count": "u64",
    "usdt_count_power": "u64",
    "usdt_amount": "u64",
    "usdt_amount_power": "u64",
    
    "ecap_power": "u64",
    "ecap_count": "u64",
    "ecap_count_power": "u64",
    "ecap_amount": "u64",
    "ecap_amount_power": "u64",
    "block_number": "BlockNumber"
  },

  "VoteRewardPeriodEnum": {
  "_enum": {
    "Days": "u32",
    "Minutes": "u32",
    "Hours": "u32"
  }
  },


  "PermilllChangeIntoU64": "u64",

  "FetchFailedOf": {
    "timestamp": "Moment",
     "tx": "Vec<u8>",
    "err": "Vec<u8>"
  }
}
```
