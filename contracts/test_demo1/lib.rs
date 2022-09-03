#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod test_demo1 {

    use set_code::SetCodeRef;

    /// This struct contains the smart contract storage.
    /// The storage will always be retained, even when `set_code_hash` is called.
    #[ink(storage)]
    pub struct TestDemo1 {
        value: u32,
    }

    impl TestDemo1 {
        /// Creates a new counter smart contract initialized to `0`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                value: 0
            }
        }

        /// Increments the counter value which is stored in the contract's storage.
        #[ink(message, selector = 0xb0f2b72a)]
        pub fn set_value(&mut self, new: u32) {
            self.value = new;
        }

        #[ink(message, selector = 0x125d8485)]
        pub fn add_a_number(&self) -> u32 {
            self.value + 5
        }

        /// Returns the counter value which is stored in this contract's storage.
        #[ink(message, selector = 0x3fa4f245)]
        pub fn get(&self) -> u32 {
            self.value
        }

        /// Modifies the code which is used to execute calls to this contract address (`AccountId`).
        ///
        /// We use this to upgrade the contract logic. We don't do any authorization here, any caller
        /// can execute this method. In a production contract you would do some authorization here.
        // #[ink(message, selector = 0x1831688a)]
        // pub fn upgrade_contract(&mut self, code_hash: [u8; 32]) {
        //     ink_env::set_code_hash(&code_hash).unwrap_or_else(|err| {
        //         panic!(
        //             "Failed to `set_code_hash` to {:?} due to {:?}",
        //             code_hash, err
        //         )
        //     });
        //     ink_env::debug_println!("Switched code hash to {:?}.", code_hash);
        // }

        #[ink(message, selector = 0x1831688a)]
        pub fn special_upgrade(&mut self, code_hash: Hash, lib_contr_addr: AccountId) -> bool{
            let mut lib_contr_addr: SetCodeRef = ink_env::call::FromAccountId::from_account_id(lib_contr_addr);
            lib_contr_addr.replace_code(code_hash)
        }
    }
}