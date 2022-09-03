#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod test_demo2 {

    /// This struct contains the smart contract storage.
    /// The storage will always be retained, even when `set_code_hash` is called.
    #[ink(storage)]
    pub struct TestDemo2 {
        value: u32,
    }

    impl TestDemo2 {
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
            self.value + 100
        }

        #[ink(message, selector = 0x1d9a30cd)]
        pub fn mul_by_10(&self) -> u32 {
            self.value * 10
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
        #[ink(message, selector = 0x1831688a)]
        pub fn upgrade_contract(&mut self, code_hash: [u8; 32]) {
            ink_env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!(
                    "Failed to `set_code_hash` to {:?} due to {:?}",
                    code_hash, err
                )
            });
            ink_env::debug_println!("Switched code hash to {:?}.", code_hash);
        }
    }
}