use super::Runtime;
use frame_support::log::error;
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;
use crate::AccountId;

#[derive(Default)]
pub struct CustomExtension;

impl ChainExtension<Runtime> for CustomExtension {
	fn call<E>(
		&mut self,
		env: Environment<E, InitState>,
	) -> pallet_contracts::chain_extension::Result<RetVal>
	where
		E: Ext<T = Runtime>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let mut env = env.buf_in_buf_out();
		let func_id = env.func_id();

		match func_id {
			11 => {
				//  Upgrade contract using pallet-contract set_code() function

				let (contract, code_hash): (AccountId, <Runtime as frame_system::Config>::Hash) =
					env.read_as()?;
			
				let err_code = match crate::Contracts::set_code(
					RawOrigin::Root.into(),
					sp_runtime::MultiAddress::Id(contract),
					code_hash,
				)
				.is_ok()
				{
					true => 0,
					false => 1,
				};
			
				Ok(RetVal::Converging(err_code))
			},
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				Err(DispatchError::Other("Unimplemented func_id"))
			},
		}
	}

	fn enabled() -> bool {
		true
	}
}