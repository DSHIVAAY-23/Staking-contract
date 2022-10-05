#![cfg_attr(not(feature = "std"), no_std)]
#![allow(arithmetic_overflow)]

use ink_lang as ink;

#[ink::contract]
mod staking {
    use ink_storage::{
        Mapping,
        traits::{
            SpreadAllocate,
            PackedLayout,
            SpreadLayout,
        },
    };

    type Time = u64;

    #[derive(PackedLayout, SpreadLayout)]
    #[derive(Debug, PartialEq, Eq)]
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Lock {
        locked_amt: Balance,
        locked_on: Time,
        last_claimed: Option<Time>,
    }

    impl Lock {

        // Calculates the unlocked amount at a given time
        pub fn claimable_value(&self, time: Option<Time>) -> Balance {
            if time.is_none() {
                return 0;
            }
            let time = time.unwrap();
            let mut period = self.locked_on;
            let mut tokens = self.locked_amt;
            let mut claim = tokens/2;   // 50% unlocks on Day 1
            let daily_unlock = tokens/10; // 10% unlocks daily from Day 2
            const DAY: Time = 1000*60*60*24;

            tokens -= claim;
            period += DAY;
            while time >= period && tokens > 0 {
                tokens -= daily_unlock;
                claim += daily_unlock;
                if tokens < daily_unlock {
                    // Considers the case when some tokens are left out because of division
                    claim += tokens;
                    tokens = 0;
                }
                period += DAY;
            }
            return claim;
        }

        // Returns the total amount locked by the given user
        pub fn locked_amt(&self) -> Balance {
            self.locked_amt
        }
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Staking {
        stakes: Mapping<AccountId, Lock>,
    }

    impl Staking {
        
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|_| {})
        }

        /// Reads lock state of the invoker
        #[ink(message)]
        pub fn get_lock_details(&self) -> Option<Lock> {
            let caller = self.env().caller();
            self.stakes.get(&caller)
        }

        /// Returns the amount of tokens still locked
        #[ink(message)]
        pub fn get_locked_amt(&self) -> Balance {
            let lock = self.get_lock_details();
            let now = self.env().block_timestamp();
            match lock {
                None => 0,
                Some(lock) => lock.locked_amt() - lock.claimable_value(Some(now))
            }
        }

        /// Returns the amount already claimed by the user so far
        #[ink(message)]
        pub fn get_claimed_amt(&self) -> Balance {
            let lock = self.get_lock_details();
            match lock {
                None => 0,
                Some(lock) => lock.claimable_value(lock.last_claimed)
            }
        }

        /// Returns the amount of token that the invoker can claim at time t
        #[ink(message)]
        pub fn get_pending_amt(&self) -> Balance {
            let lock = self.get_lock_details();
            let now = self.env().block_timestamp();
            match lock {
                None => 0,
                Some(lock) => lock.claimable_value(Some(now)) - lock.claimable_value(lock.last_claimed)
            }
        }

        // @dev: Only there for simple testing
        fn insert(&mut self, locked_amt: Balance, locked_on: Time) {
            let caller = self.env().caller();
            let lock = Lock{locked_amt, locked_on, last_claimed:None};
            self.stakes.insert(&caller,&lock);
        }

        /// User can send tokens for staking
        /// User can stake only when no previous stake order is pending
        #[ink(message,payable)]
        pub fn lock_tokens(&mut self) {
            let locked_amt = self.env().transferred_value();

            assert!(locked_amt > 0, "Zero tokens sent");
            assert!(self.get_locked_amt() == 0, "Your previous lock period has not ended");
            assert!(self.get_pending_amt() == 0, "Claim your tokens from previous lock first");

            let caller = self.env().caller();
            let locked_on = self.env().block_timestamp();
            let last_claimed: Option<Time> = None;
            let lock = Lock{locked_amt,locked_on,last_claimed};
            self.stakes.insert(&caller,&lock);
        }

        /// User can claim their pending tokens which have been unlocked
        #[ink(message)]
        pub fn claim_tokens(&mut self) {
            let value = self.get_pending_amt();
            if value == 0 {
                return;
            }
            let mut lock = self.get_lock_details().unwrap();
            lock.last_claimed = Some(self.env().block_timestamp());
            let caller = self.env().caller();
            self.env().transfer(caller,value).unwrap();
            self.stakes.insert(&caller,&lock);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let staking = Staking::new();
            assert_eq!(staking.get_lock_details(), None);
        }

        #[ink::test]
        fn insert_works() {
            let mut staking = Staking::new();
            assert_eq!(staking.get_lock_details(), None);
            staking.insert(5,10);
            let output = Lock{locked_amt: 5,locked_on: 10, last_claimed: None};
            assert_eq!(staking.get_lock_details(), Some(output));
        }
    }
}
