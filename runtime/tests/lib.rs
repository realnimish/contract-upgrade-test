#[cfg(test)]
mod tests {
	use codec::{Decode, Encode};
	use contracts_node_runtime::{
		AccountId, Balance, Block, BlockNumber, Contracts, Event, Hash, Origin, Runtime, System,
		Weight,
	};
	use frame_support::assert_ok;
	use pallet_contracts_primitives::ExecReturnValue;
	use pallet_contracts_rpc_runtime_api::runtime_decl_for_ContractsApi::ContractsApi;
	use sp_core::{hexdisplay::AsBytesRef, Bytes};
	use std::str::FromStr;
	const MAX_GAS: Weight = 200_000_000_000;

	pub const ALICE: AccountId = AccountId::new([1u8; 32]);

	fn deploy_contract(blob: Vec<u8>, sel_constructor: Vec<u8>) -> AccountId {
		assert_ok!(Contracts::instantiate_with_code(
			Origin::signed(ALICE),
			0,
			MAX_GAS,
			None,
			blob,
			sel_constructor,
			vec![]
		));

		let evts = System::events();

		let deployed_address = evts
			.iter()
			.rev()
			.find_map(|r| {
				if let Event::Contracts(pallet_contracts::Event::Instantiated {
					deployer: _,
					contract,
				}) = &r.event
				{
					Some(contract)
				} else {
					None
				}
			})
			.expect("unable to find contract");

		deployed_address.clone()
	}

	fn query_contract(
		caller: AccountId,
		addr: &AccountId,
		input: &Vec<u8>,
	) -> Result<ExecReturnValue, sp_runtime::DispatchError> {
		<Runtime as ContractsApi<Block, AccountId, Balance, BlockNumber, Hash>>::call(
			caller,
			addr.clone(),
			0,
			MAX_GAS,
			None,
			input.clone(),
		)
		.result
	}

	// Build genesis storage according to the mock runtime.
	pub fn new_test_ext() -> frame_support::sp_io::TestExternalities {
		let mut storage =
			frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, 100_000_000_000_000)] }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext = frame_support::sp_io::TestExternalities::new(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn test_set_code_hash() {
		new_test_ext()
			.execute_with(|| {

				// 1A. Deploy the library contract (set_code)
				let blob = std::fs::read("../contracts/set_code/dist/set_code.wasm")
					.expect("Could not find wasm blob");

                let sel_constructor = Bytes::from_str("0x9bae9d5e")
                    .map(|v| v.to_vec())
                    .expect("unable to parse selector");

                let upgrade_contract = deploy_contract(blob, sel_constructor);

                // 1B. Deploy demo_contract ver0
                let blob_demo0 = std::fs::read("../contracts/test_demo1/dist/test_demo1.wasm")
					.expect("Could not find wasm blob");

                let sel_constructor_demo0 = Bytes::from_str("0xed4b9d1b")
                    .map(|v| v.to_vec())
                    .expect("unable to parse selector");

                let demo0 = deploy_contract(blob_demo0, sel_constructor_demo0);

				// 2. Set storage value to 50
				let mut sel_set_value = Bytes::from_str("0xb0f2b72a")
				.map(|v| v.to_vec())
				.expect("unable to parse selector");

				sel_set_value.append(&mut 50u32.encode());

				assert_ok!(Contracts::call(
					Origin::signed(ALICE),
					demo0.clone().into(),
					0,
					MAX_GAS,
					None,
					sel_set_value.clone(),
				));

				// 3A. Test add_a_number (returns sum of value & 5)
				let sel_add_number = Bytes::from_str("0x125d8485")
				.map(|v| v.to_vec())
				.expect("unable to parse selector");

				let ExecReturnValue{flags, data} = query_contract(ALICE, &demo0, &sel_add_number)
					.expect("Execution without result");

				assert!(flags.is_empty());
				let value = u32::decode(&mut data.as_bytes_ref()).expect("failed to decode result");
				assert_eq!(value, 55u32);

				// 3B. Test mul_a_number doesn't exist
				let sel_mul_number = Bytes::from_str("0x1d9a30cd")
					.map(|v| v.to_vec())
					.expect("unable to parse selector");

				frame_support::assert_err!(
					query_contract(ALICE, &demo0, &sel_mul_number),
					pallet_contracts::Error::<Runtime>::ContractTrapped,
				);

				// --> Upgrade demo contract from ver1 to ver2

				// 4A. Check current code_version
				let mut sel_code_version = Bytes::from_str("0xe82d14a6")
					.map(|v| v.to_vec())
					.expect("unable to parse selector");

				sel_code_version.append(&mut demo0.encode());

				let ExecReturnValue{flags, data} = query_contract(ALICE, &upgrade_contract, &sel_code_version)
					.expect("Execution without result");

				assert!(flags.is_empty());
				let version_before_update = u32::decode(&mut data.as_bytes_ref()).expect("failed to decode result");
				assert_eq!(version_before_update, 0);

                // 4B. Upload demo_contract ver1 code
				let blob_demo1 = std::fs::read("../contracts/test_demo2/dist/test_demo2.wasm")
					.expect("Could not find wasm blob");

				let ch =
                    <Runtime as ContractsApi<Block, AccountId, Balance, BlockNumber, Hash>>::upload_code(
                        ALICE,
                        blob_demo1,
                        None,
                    )
                    .expect("Failed to upload code")
                    .code_hash;

				// 4C. Upgrade demo_v1 to demo_v2
				let mut sel_upgrade_contract = Bytes::from_str("0x1831688a")
				.map(|v| v.to_vec())
				.expect("unable to parse selector");

				println!("PASSED CONTR({:?} & CH({:?})", demo0, ch);

				sel_upgrade_contract.append(&mut ch.encode());
                sel_upgrade_contract.append(&mut upgrade_contract.encode());

				assert_ok!(Contracts::call(
					Origin::signed(ALICE),
					demo0.clone().into(),
					0,
					MAX_GAS,
					None,
					sel_upgrade_contract.clone(),
				));

				let evts = System::events();

				evts
				.iter()
				.find(|r| if let Event::Contracts(pallet_contracts::Event::ContractCodeUpdated{
					contract: _,
					new_code_hash,
					old_code_hash: _,
				}) = &r.event {
					new_code_hash == &ch
				} else {false})
				.expect("ContractCodeUpdated event not found");

				// 5A. Check new code_version
				let mut sel_code_version = Bytes::from_str("0xe82d14a6")
					.map(|v| v.to_vec())
					.expect("unable to parse selector");

				sel_code_version.append(&mut demo0.encode());

				let ExecReturnValue{flags, data} = query_contract(ALICE, &upgrade_contract, &sel_code_version)
					.expect("Execution without result");

				assert!(flags.is_empty());
				let version_before_update = u32::decode(&mut data.as_bytes_ref()).expect("failed to decode result");
				assert_eq!(version_before_update, 1);

				// 5B. Test updated add_a_number function
				System::set_block_number(2);
				let ExecReturnValue{flags, data} = query_contract(ALICE, &demo0, &sel_add_number)
					.expect("Execution without result");

				assert!(flags.is_empty());
				let value = u32::decode(&mut data.as_bytes_ref()).expect("failed to decode result");
				assert_eq!(value, 150u32);

				// 5C. Test mul_a_number exists and it works
				let ExecReturnValue{flags, data} = query_contract(ALICE, &demo0, &sel_mul_number)
					.expect("Execution without result");

				assert!(flags.is_empty());
				let value = u32::decode(&mut data.as_bytes_ref()).expect("failed to decode result");
				assert_eq!(value, 500u32);
			})
	}
}
