/*
ABOUT THIS CONTRACT...
This contract allows users to post public broadcast messages to the Geode Blockchain. 
In this contract, to endorse a message is to upvote it (a kind of like button that might 
boost it's visibility in the front end). This contract also allows users to:
- follow and unfollow specific accounts,
- reply to regular message posts (NOT paid message posts), 
- declare their interests, 
- see paid messages that fit their interests, and 
- be paid in GEODE to endorse or upvote a paid message.
*/ 

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod geode_social {

    use ink::prelude::vec::Vec;
    use ink::prelude::vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::env::hash::{Sha2x256, HashOutput};

    // PRELIMINARY STORAGE STRUCTURES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct Settings {
        username: Vec<u8>,
        interests: Vec<u8>,
        max_feed: u128,
        max_paid_feed: u128,
        last_update: u64
    }

    impl Default for Settings {
        fn default() -> Settings {
            Settings {
                username: <Vec<u8>>::default(),
                interests: <Vec<u8>>::default(),
                max_feed: 1000,
                max_paid_feed: 1000,
                last_update: u64::default()
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct Following {
        following: Vec<AccountId>,
    }

    impl Default for Following {
        fn default() -> Following {
            Following {
              following: <Vec<AccountId>>::default(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct Blocked {
        blocked: Vec<AccountId>,
    }

    impl Default for Blocked {
        fn default() -> Blocked {
            Blocked {
              blocked: <Vec<AccountId>>::default(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct Messages {
        messages: Vec<Hash>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct PaidMessageDetails {
        message_id: Hash,
        from_acct: AccountId,
        username: Vec<u8>,
        message: Vec<u8>,
        link: Vec<u8>,
        link2: Vec<u8>,
        endorser_count: u128,
        timestamp: u64,
        paid_endorser_max: u128,
        endorser_payment: Balance,
        target_interests: Vec<u8>,
        total_staked: Balance,
        endorsers: Vec<AccountId>,
        staked_balance: Balance,
    }

    impl Default for PaidMessageDetails {
        fn default() -> PaidMessageDetails {
            PaidMessageDetails {
                message_id: Hash::default(),
                from_acct: AccountId::from([0x0; 32]),
                username: <Vec<u8>>::default(),
                message: <Vec<u8>>::default(),
                link: <Vec<u8>>::default(),
                link2: <Vec<u8>>::default(),
                endorser_count: 0,
                timestamp: u64::default(),
                paid_endorser_max: u128::default(),
                endorser_payment: Balance::default(),
                target_interests: <Vec<u8>>::default(),
                total_staked: Balance::default(),
                endorsers: <Vec<AccountId>>::default(),
                staked_balance: Balance::default(),
            }
        }
    }


    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct MessageDetails {
        message_id: Hash,
        reply_to: Hash,
        from_acct: AccountId,
        username: Vec<u8>,
        message: Vec<u8>,
        link: Vec<u8>,
        link2: Vec<u8>,
        endorser_count: u128,
        reply_count: u128,
        timestamp: u64,
    }

    impl Default for MessageDetails {
        fn default() -> MessageDetails {
            MessageDetails {
                message_id: Hash::default(),
                reply_to: Hash::default(),
                from_acct: AccountId::from([0x0; 32]),
                username: <Vec<u8>>::default(),
                message: <Vec<u8>>::default(),
                link: <Vec<u8>>::default(),
                link2: <Vec<u8>>::default(),
                endorser_count: 0,
                reply_count: 0,
                timestamp: u64::default(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct MyFeed {
        maxfeed: u128,
        blocked: Vec<AccountId>,
        myfeed: Vec<MessageDetails>,
    }

    impl Default for MyFeed {
        fn default() -> MyFeed {
            MyFeed {
              maxfeed: 1000,
              blocked: <Vec<AccountId>>::default(),
              myfeed: <Vec<MessageDetails>>::default(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct MyPaidFeed {
        maxfeed: u128,
        myinterests: Vec<u8>,
        blocked: Vec<AccountId>,
        mypaidfeed: Vec<PaidMessageDetails>,
    }

    impl Default for MyPaidFeed {
        fn default() -> MyPaidFeed {
            MyPaidFeed {
              maxfeed: 1000,
              myinterests: <Vec<u8>>::default(),
              blocked: <Vec<AccountId>>::default(),
              mypaidfeed: <Vec<PaidMessageDetails>>::default(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct SocialProfile {
        searched_account: AccountId,
        username: Vec<u8>,
        followers: u128,
        following: Vec<AccountId>,
        message_list: Vec<MessageDetails>,
    }

    impl Default for SocialProfile {
        fn default() -> SocialProfile {
            SocialProfile {
                searched_account: AccountId::from([0x0; 32]),
                username: <Vec<u8>>::default(),
                followers: 0,
                following: <Vec<AccountId>>::default(),
                message_list: <Vec<MessageDetails>>::default(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct RewardSettings {
        reward_on: u8,
        reward_root_set: u8,
        reward_root: AccountId,
        reward_interval: u128,
        reward_amount: Balance,
        reward_balance: Balance,
        reward_payouts: Balance,
        claim_counter: u128,
    }

    impl Default for RewardSettings {
        fn default() -> RewardSettings {
            RewardSettings {
                reward_on: u8::default(),
                reward_root_set: u8::default(),
                reward_root: AccountId::from([0x0; 32]),
                reward_interval: u128::default(),
                reward_amount: Balance::default(),
                reward_balance: Balance::default(),
                reward_payouts: Balance::default(),
                claim_counter: u128::default(),
            }
        }
    }


    // EVENT DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[ink(event)]
    // Writes a broadcast message to the blockchain 
    pub struct MessageBroadcast {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        message: Vec<u8>,
        #[ink(topic)]
        message_id: Hash,
        link: Vec<u8>,
        link2: Vec<u8>,
        reply_to: Hash,
        timestamp: u64
    }

    #[ink(event)]
    // Writes a paid broadcast message to the blockchain 
    pub struct PaidMessageBroadcast {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        message: Vec<u8>,
        #[ink(topic)]
        message_id: Hash,
        link: Vec<u8>,
        link2: Vec<u8>,
        timestamp: u64,
        paid_endorser_max: u128,
        endorser_payment: Balance,
        target_interests: Vec<u8>,
        total_staked: Balance
    }

    #[ink(event)]
    // Writes the new endorsement to the blockchain 
    pub struct MessageElevated {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        message_id: Hash,
        #[ink(topic)]
        endorser: AccountId
    }

    #[ink(event)]
    // Writes the new paid endorsement to the blockchain 
    pub struct PaidMessageElevated {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        message_id: Hash,
        #[ink(topic)]
        endorser: AccountId
    }

    #[ink(event)]
    // Writes the new follow to the blockchain 
    pub struct NewFollow {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        is_following: AccountId,
    }

    #[ink(event)]
    // Writes the new UNfollow to the blockchain 
    pub struct NewUnFollow {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        stopped_following: AccountId,
    }

    #[ink(event)]
    // Writes the new BLOCK to the blockchain 
    pub struct NewBlock {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        blocked: AccountId,
    }

    #[ink(event)]
    // Writes the new unBLOCK to the blockchain 
    pub struct NewUnBlock {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        unblocked: AccountId,
    }

    #[ink(event)]
    // Writes the new settings update to the blockchain 
    pub struct SettingsUpdated {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        username: Vec<u8>,
        #[ink(topic)]
        interests: Vec<u8>,
    }

    #[ink(event)]
    // Writes the new reward to the blockchain 
    pub struct AccountRewardedSocial {
        #[ink(topic)]
        claimant: AccountId,
        reward: Balance,
    }


    // ERROR DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        // Following an account that you follow already
        CannotFollow,
        // Unfollowing an account that you don't follow anyway
        NotFollowing,
        NotInFollowerList,
        // Blocking an account that you already blocked
        CannotBlock,
        // Unblocking an account that you never blocked
        NotBlocked,
        // Elevating a message that does not exist
        NonexistentMessage,
        // Elevating a paid message that does not exist
        NonexistentPaidMessage,
        // Elevating the same message twice
        DuplicateEndorsement,
        // trying to update your interest before 24 hours have past
        CannotUpdateInterestsWithin24Hours,
        // Too many interests in your list
        InterestsTooLong,
        // Trying to endorse a paid message outside your interests
        NoInterestMatch,
        // When a paid message has run out of available endorsements
        NoMorePaidEndorsementsAvailable,
        // if the contract has no money to pay
        ZeroBalance,
        // if the endorser payment fails
        EndorserPayoutFailed,
        // Reentrancy Guard error
        UsernameTaken,
        // if you are replying to a message that does not exist
        ReplyingToMessageDoesNotExist,
        // too much data in the inputs
        DataTooLarge,
        // paid message bid is too low to get into the set
        BidTooLow,
        // replies are full, cannot reply to this post
        RepliesFull,
        // Caller doee not have permission
        PermissionDenied,
        // reward account payout failed
        PayoutFailed,
    }


    // ACTUAL CONTRACT STORAGE STRUCT >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[ink(storage)]
    pub struct ContractStorage {
        account_settings_map: Mapping<AccountId, Settings>,
        account_following_map: Mapping<AccountId, Following>,
        account_followers_map: Mapping<AccountId, u128>,
        account_blocked_map: Mapping<AccountId, Blocked>,
        account_messages_map: Mapping<AccountId, Messages>,
        account_paid_messages_map: Mapping<AccountId, Messages>,
        account_elevated_map: Mapping<AccountId, Hash>,
        message_map: Mapping<Hash, MessageDetails>,
        reply_map: Mapping<Hash, MessageDetails>,
        paid_message_map: Mapping<Hash, PaidMessageDetails>,
        target_interests_map: Mapping<Vec<u8>, Messages>,
        message_reply_map: Mapping<Hash, Messages>,
        username_map: Mapping<Vec<u8>, AccountId>,
        reward_root_set: u8,
        reward_root: AccountId,
        reward_interval: u128,
        reward_amount: Balance,
        reward_on: u8,
        reward_balance: Balance,
        reward_payouts: Balance,
        claim_counter: u128,
    }


    // BEGIN CONTRACT LOGIC >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    impl ContractStorage {
        
        // CONSTRUCTORS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // Constructors are implicitly payable.

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                account_settings_map: Mapping::default(),
                account_following_map: Mapping::default(),
                account_followers_map: Mapping::default(),
                account_blocked_map: Mapping::default(),
                account_messages_map: Mapping::default(),
                account_paid_messages_map: Mapping::default(),
                account_elevated_map: Mapping::default(),
                message_map: Mapping::default(),
                reply_map: Mapping::default(),
                paid_message_map: Mapping::default(),
                target_interests_map: Mapping::default(),
                message_reply_map: Mapping::default(),
                username_map: Mapping::default(),
                reward_root_set: 0,
                reward_root: AccountId::from([0x0; 32]),
                reward_interval: 1000000,
                reward_amount: 0,
                reward_on: 0,
                reward_balance: 0,
                reward_payouts: 0,
                claim_counter: 0,
            }
        }


        // MESSAGE FUNCTIONS THAT CHANGE DATA IN THE CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>
        

        // 🟢 0 SEND MESSAGE PUBLIC (TOP LEVEL MESSAGE, NOT A REPLY)
        // sends a broadcast public message on the chain
        #[ink(message)]
        pub fn send_message_public (&mut self, 
            new_message: Vec<u8>, 
            photo_or_youtube_link: Vec<u8>, 
            website_or_document_link: Vec<u8>, 
        ) -> Result<(), Error> {

            // check data limits on all inputs:
            // message 300 characters (600 length), links 300 characters (600 length)
            if new_message.len() > 600 || photo_or_youtube_link.len() > 600
            || website_or_document_link.len() > 600 {
                // error - data too large
                return Err(Error::DataTooLarge);
            }

            let new_message_clone = new_message.clone();
            let new_message_clone2 = new_message.clone();
            let link_clone = photo_or_youtube_link.clone();
            let link2_clone = website_or_document_link.clone();

            // set up the data that will go into the new_message_id
            let from = Self::env().caller();
            let new_timestamp = self.env().block_timestamp();

            // create the new_message_id by hashing the above data
            let encodable = (from, new_message, new_timestamp); // Implements `scale::Encode`
            let mut new_message_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
            ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_message_id_u8);
            let new_message_id: Hash = Hash::from(new_message_id_u8);

            // SET UP THE MESSAGE DETAILS FOR THE NEW MESSAGE
            let caller = Self::env().caller();
            let fromusername = self.account_settings_map.get(caller).unwrap_or_default().username;
            let new_details = MessageDetails {
                message_id: new_message_id,
                reply_to: Hash::default(),
                from_acct: Self::env().caller(),
                username: fromusername,
                message: new_message_clone,
                link: photo_or_youtube_link,
                link2: website_or_document_link,
                endorser_count: 0,
                reply_count: 0,
                timestamp: self.env().block_timestamp(),
            };

            // UPDATE MESSAGE MAP AND VECTOR
            // add the message id and its details to the message_map
            if self.message_map.try_insert(&new_message_id, &new_details).is_err() {
                return Err(Error::DataTooLarge);
            }

            // UPDATE ACCOUNT MESSAGES MAP
            // get the messages vector for this account
            let caller = Self::env().caller();
            let mut current_messages = self.account_messages_map.get(&caller).unwrap_or_default();
            // Keep only the 3 most recent message hashes
            if current_messages.messages.len() > 2 {
                // get the id for the oldest message
                let oldest = current_messages.messages[0];
                // remove the oldest from the message_map
                self.message_map.remove(oldest);
                // remove the oldest message from account_messages_map
                current_messages.messages.remove(0);
                // remove all the replies to the oldest message from the reply_map 
                let replies = self.message_reply_map.get(oldest).unwrap_or_default();
                for id in replies.messages.iter() {
                    self.reply_map.remove(id);
                }
                // remove the oldest message from the message_reply_map
                self.message_reply_map.remove(oldest);
            }
            // add the new message to the end of the storage
            current_messages.messages.push(new_message_id);
            // update the account_messages_map
            self.account_messages_map.insert(&caller, &current_messages);

            // EMIT EVENT to register the post to the chain
            Self::env().emit_event(MessageBroadcast {
                from: Self::env().caller(),
                message: new_message_clone2,
                message_id: new_message_id,
                link: link_clone,
                link2: link2_clone,
                reply_to: Hash::default(),
                timestamp: self.env().block_timestamp()
            });

            // REWARD PROGRAM ACTIONS... update the claim_counter 
            self.claim_counter = self.claim_counter.wrapping_add(1);
            // IF conditions are met THEN payout a reward
            let min = self.reward_amount.saturating_add(10);
            let payout: Balance = self.reward_amount;
            if self.reward_on == 1 && self.reward_balance > payout && self.env().balance() > min
            && self.claim_counter.checked_rem_euclid(self.reward_interval) == Some(0) {
                // payout
                if self.env().transfer(caller, payout).is_err() {
                    return Err(Error::PayoutFailed);
                }
                // update reward_balance
                self.reward_balance = self.reward_balance.saturating_sub(payout);
                // update reward_payouts
                self.reward_payouts = self.reward_payouts.saturating_add(payout);
                // emit an event to register the reward to the chain
                Self::env().emit_event(AccountRewardedSocial {
                    claimant: caller,
                    reward: payout
                });
            }
            // END REWARD PROGRAM ACTIONS
            
            Ok(())
        }

        
        // 🟢 1 SEND PAID MESSAGE PUBLIC 
        // sends a paid public broadcast message post
        // and offers coin to the first X accounts to endorse/elevate the post
        #[ink(message, payable)]
        pub fn send_paid_message_public (&mut self, 
            new_message: Vec<u8>,
            photo_or_youtube_link: Vec<u8>,
            website_or_document_link: Vec<u8>,
            maximum_number_of_paid_endorsers: u128,
            payment_per_endorser: Balance,
            target_interests: Vec<u8>
        ) -> Result<(), Error> {

            // check that the inputs are not too long
            // message 300 characters (600 length), links: 300 characters (600 length)
            // target interests 50 characters (100 length)
            if photo_or_youtube_link.len() > 600 || target_interests.len() > 100 
            || target_interests.len() > 600 || website_or_document_link.len() > 600 {
                // error - data too large
                return Err(Error::DataTooLarge);
            }

            let caller = Self::env().caller();

            let new_message_clone = new_message.clone();
            let new_message_clone2 = new_message.clone();
            let interests_clone = target_interests.clone();
            let interests_clone2 = target_interests.clone();
            let link_clone = photo_or_youtube_link.clone();
            let link2_clone = website_or_document_link.clone();
            
            // CREATE THE MESSAGE ID HASH
            // set up the data that will go into the new_message_id
            let from = Self::env().caller();
            let new_timestamp = self.env().block_timestamp();
            // create the new_message_id by hashing the above data
            let encodable = (from, new_message, new_timestamp); // Implements `scale::Encode`
            let mut new_message_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
            ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_message_id_u8);
            let new_message_id: Hash = Hash::from(new_message_id_u8);

            // COLLECT PAYMENT FROM THE CALLER
            // the 'payable' tag on this message allows the user to send any amount
            // so we determine here what that will give each endorser
            let staked: Balance = self.env().transferred_value();

            // MAKE THE PAID MESSAGE DETAILS STRUCT
            let fromusername = self.account_settings_map.get(caller).unwrap_or_default().username;
            // set up the paid message details
            let new_details = PaidMessageDetails {
                    message_id: new_message_id,
                    from_acct: Self::env().caller(),
                    username: fromusername,
                    message: new_message_clone,
                    link: photo_or_youtube_link,
                    link2: website_or_document_link,
                    endorser_count: 0,
                    timestamp: self.env().block_timestamp(),
                    paid_endorser_max: maximum_number_of_paid_endorsers,
                    endorser_payment: payment_per_endorser,
                    target_interests: target_interests,
                    total_staked: staked,
                    endorsers: vec![Self::env().caller()],
                    staked_balance: staked,
            };
        
            // if the account paid messages are full, kick out the oldest from everywhere
            // get the messages vector for this account
            let mut current_messages = self.account_paid_messages_map.get(&caller).unwrap_or_default();
            // if the paid messages vector is full, remove the oldest message
            if current_messages.messages.len() > 54 {
                // get the id hash and interests for the oldest message
                let oldest = current_messages.messages[0];
                let old_interests = self.paid_message_map.get(oldest).unwrap_or_default().target_interests;
                let old_interests_clone = old_interests.clone();
                // remove the oldest from the paid_message_map
                self.paid_message_map.remove(oldest);
                // remove the oldest from the target_interests_map
                let mut old_interests_vec = self.target_interests_map.get(old_interests).unwrap_or_default();
                old_interests_vec.messages.retain(|value| *value != oldest);
                self.target_interests_map.insert(old_interests_clone, &old_interests_vec);
                // remove the oldest from the account_paid_messages_map
                current_messages.messages.remove(0);
            }
            
            // IF THERE ARE TOO MANY MESSAGES FOR THIS INTERESTS TARGET, THROW OUT THE LOW BIDDER
            // get the current set of messages that match this target
            let mut matching_messages = self.target_interests_map.get(&interests_clone).unwrap_or_default();
            // if there are > 55 messages for this target, remove the lowest bidder
            if matching_messages.messages.len() > 54 {
                // determine if this message bids high enough...
                // check the other bids and find the lowest
                let first_hash = matching_messages.messages[0];
                let mut low_bid: Balance = self.paid_message_map.get(first_hash).unwrap_or_default().endorser_payment;
                let mut low_index: usize = 0;
                for (i, ad) in matching_messages.messages.iter().enumerate() {
                    // get the bid and index
                    let bid: Balance = self.paid_message_map.get(ad).unwrap_or_default().endorser_payment;
                    if bid < low_bid { 
                        low_bid = bid;
                        low_index = i;
                    }
                }
                if payment_per_endorser > low_bid {
                    // kick out the low bidder 
                    matching_messages.messages.remove(low_index);
                    // we do not remove the low bidder out of the account_paid_messages_map
                    // or the paid_message_map becuase they will need to be able to get
                    // their money back by endorsing their own message.
                }
                else {
                    // error bid not high enough
                    return Err(Error::BidTooLow);
                }
            }

            // add the message id and its details to the paid message_map
            if self.paid_message_map.try_insert(&new_message_id, &new_details).is_err() {
                return Err(Error::DataTooLarge);
            }

            // add this message to the messages vector for this account
            current_messages.messages.push(new_message_id);
            // update the account_messages_map
            self.account_paid_messages_map.insert(&caller, &current_messages);

            // add the new message to the list for these target interests
            matching_messages.messages.push(new_message_id);
            // update the mapping
            self.target_interests_map.insert(&interests_clone, &matching_messages);

            // EMIT AN EVENT (to register the post to the chain)
            Self::env().emit_event(PaidMessageBroadcast {
                from: Self::env().caller(),
                message: new_message_clone2,
                message_id: new_message_id,
                link: link_clone,
                link2: link2_clone,
                timestamp: self.env().block_timestamp(),
                paid_endorser_max: maximum_number_of_paid_endorsers,
                endorser_payment: payment_per_endorser,
                target_interests: interests_clone2,
                total_staked: staked
            });

            // REWARD PROGRAM ACTIONS... update the claim_counter 
            self.claim_counter = self.claim_counter.wrapping_add(1);
            // IF conditions are met THEN payout a reward
            let min = self.reward_amount.saturating_add(10);
            let payout: Balance = self.reward_amount;
            if self.reward_on == 1 && self.reward_balance > payout && self.env().balance() > min
            && self.claim_counter.checked_rem_euclid(self.reward_interval) == Some(0) {
                // payout
                if self.env().transfer(caller, payout).is_err() {
                    return Err(Error::PayoutFailed);
                }
                // update reward_balance
                self.reward_balance = self.reward_balance.saturating_sub(payout);
                // update reward_payouts
                self.reward_payouts = self.reward_payouts.saturating_add(payout);
                // emit an event to register the reward to the chain
                Self::env().emit_event(AccountRewardedSocial {
                    claimant: caller,
                    reward: payout
                });
            }
            // END REWARD PROGRAM ACTIONS

            Ok(())

        }


        // 🟢 2 ELEVATE MESSAGE 
        // upvotes a public message by endorsing it on chain (unpaid) 
        #[ink(message)]
        pub fn elevate_message(&mut self, this_message_id: Hash) -> Result<(), Error> {
            
            // Does the message_id exist in the message_map? ...
            if self.message_map.contains(&this_message_id) {

                // Get the contract caller's Account ID
                let caller = Self::env().caller();
                // Get the details for this message_id from the message_map
                let current_details = self.message_map.get(&this_message_id).unwrap_or_default();
               
                // Is it your own message?... 
                if current_details.from_acct == caller {
                    // If TRUE, return an Error... DuplicateEndorsement
                    return Err(Error::DuplicateEndorsement)
                } 
                else {
                    // update the endorser count
                    let new_endorser_count = current_details.endorser_count.saturating_add(1);

                    // Update the details in storage for this message
                    let updated_details: MessageDetails = MessageDetails {
                        message_id: current_details.message_id,
                        reply_to: current_details.reply_to,
                        from_acct: current_details.from_acct,
                        username: current_details.username,
                        message: current_details.message,
                        link: current_details.link,
                        link2: current_details.link2,
                        endorser_count: new_endorser_count,
                        reply_count: current_details.reply_count,
                        timestamp: current_details.timestamp,
                    };

                    // Update the message_map
                    if self.message_map.try_insert(&this_message_id, &updated_details).is_err() {
                        return Err(Error::DataTooLarge);
                    }        

                    // Add this message to the account_elevated_map for this caller
                    self.account_elevated_map.insert(&caller, &this_message_id);

                    // Emit an event to register the endorsement to the chain...
                    Self::env().emit_event(MessageElevated {
                        from: updated_details.from_acct,
                        message_id: this_message_id,
                        endorser: Self::env().caller()
                    });

                    Ok(())
                }
            }

            else {
                // if the message_id does not exist ...Error: Nonexistent Message
                return Err(Error::NonexistentMessage);
            }

        }


        // 🟢 3 ELEVATE PAID MESSAGE 
        // endorses a paid message and pays the endorser accordingly
        #[ink(message)]
        pub fn elevate_paid_message(&mut self, this_message_id: Hash) -> Result<(), Error> {

            // Does the message_id exist in the paid_message_map? If TRUE then...
            if self.paid_message_map.contains(&this_message_id) {

                // Get the contract caller's Account ID
                let caller = Self::env().caller();
                // Get the details for this paid message...
                let mut current_details = self.paid_message_map.get(&this_message_id).unwrap_or_default();

                // Is the caller already in the endorsers list for this message? 
                if current_details.endorsers.contains(&caller) {
                    // If TRUE, return an Error... DuplicateEndorsement
                    return Err(Error::DuplicateEndorsement)
                } 
                else {
                    // If the caller is NOT already an endorser...
                    // Does the caller match the interests required for payment?
                    // Get the callers list of interests...
                    let caller_interests = self.account_settings_map.get(&caller).unwrap_or_default().interests;

                    // check to see if the caller's interests include the target_interests
                    let caller_interests_string = String::from_utf8(caller_interests).unwrap_or_default();
                    let targetvecu8 = current_details.target_interests.clone();
                    let target_string = String::from_utf8(targetvecu8).unwrap_or_default();
                    if caller_interests_string.contains(&target_string) {
                        
                        // Has this paid message hit its limit on paid endorsements?
                        let max_endorsements = current_details.paid_endorser_max;
                        let current_endorsement_number: u128 = current_details.endorser_count;
                        if current_endorsement_number < max_endorsements {
                            
                            // Pay the endorser the right amount from the contract
                            let mut paythis: Balance = current_details.endorser_payment;
                            // if the endorser is the advertiser, refund the remainder in full
                            if caller == current_details.from_acct {
                                paythis = current_details.staked_balance;
                            }
                            let contractmin: Balance = paythis.saturating_add(11);
                            // Check that there is a nonzero balance on the contract > existential deposit
                            // plus the payout plus one
                            if self.env().balance() > contractmin && current_details.staked_balance >= paythis {
                                // pay the endorser the amount paythis
                                if self.env().transfer(caller, paythis).is_err() {
                                    return Err(Error::EndorserPayoutFailed);
                                }
                            }
                            // if the balance is zero, Error (ZeroBalance)
                            else {
                                return Err(Error::ZeroBalance);
                            }

                            // update the endorsers vector...
                            // if there are already 400 endorsers, kick out the oldest endorser
                            if current_details.endorsers.len() > 399 {
                                current_details.endorsers.remove(0);
                            }
                            // Add this endorser to the vector of endorsing accounts
                            current_details.endorsers.push(caller);

                            // update the endorser count
                            let new_endorser_count = current_details.endorser_count.saturating_add(1);
                            // update the staked balance
                            let new_balance: Balance = current_details.staked_balance.saturating_sub(paythis);

                            // Update the details in storage for this paid message
                            let updated_details: PaidMessageDetails = PaidMessageDetails {
                                message_id: current_details.message_id,
                                from_acct: current_details.from_acct,
                                username: current_details.username,
                                message: current_details.message,
                                link: current_details.link,
                                link2: current_details.link2,
                                endorser_count: new_endorser_count,
                                timestamp: current_details.timestamp,
                                paid_endorser_max: current_details.paid_endorser_max,
                                endorser_payment: current_details.endorser_payment,
                                target_interests: current_details.target_interests,
                                total_staked: current_details.total_staked,
                                endorsers: current_details.endorsers,
                                staked_balance: new_balance,
                            };

                            // Update the paid_message_map
                            if self.paid_message_map.try_insert(&this_message_id, &updated_details).is_err() {
                                return Err(Error::DataTooLarge);
                            }                

                            // Emit an event to register the endorsement to the chain
                            // but only if the caller is not the advertiser
                            if caller != updated_details.from_acct {
                                Self::env().emit_event(PaidMessageElevated {
                                    from: updated_details.from_acct,
                                    message_id: this_message_id,
                                    endorser: Self::env().caller()
                                });
                            }

                        }
                        else {
                            // return an error that there are no endorsements available
                            return Err(Error::NoMorePaidEndorsementsAvailable)
                        }  
                        
                    }
                    else {
                        return Err(Error::NoInterestMatch);
                    }
                    
                    Ok(())
                }
            }

            else {
                // if the message_id does not exist ...Error: Nonexistent Paid Message
                return Err(Error::NonexistentPaidMessage);
            }

        }


        // 🟢 4 FOLLOW ACCOUNT
        // allows a user to follow another accountId's messages
        #[ink(message)]
        pub fn follow_account (&mut self, follow: AccountId
        ) -> Result<(), Error> {
            let caller = Self::env().caller();
            // Is this account already being followed? or is the caller trying to follow themselves?
            let mut current_follows = self.account_following_map.get(&caller).unwrap_or_default();
            if current_follows.following.contains(&follow) || caller == follow {
                return Err(Error::CannotFollow);
            }
            // Otherwise, update the account_following_map for this caller
            else {
                // if there are already > 98 accounts in the follow list, keep the most recent 99
                if current_follows.following.len() > 98 {
                    // kick out the oldest follow
                    current_follows.following.remove(0);
                }
                // add the new follow to the the vector of accounts caller is following
                current_follows.following.push(follow);
                // Update (overwrite) the account_following_map entry in the storage
                self.account_following_map.insert(&caller, &current_follows);
                // get the number of current followers for the followed account
                let mut current_followers = self.account_followers_map.get(&follow).unwrap_or_default(); 
                // add the caller to the count of followers for this account
                current_followers = current_followers.saturating_add(1);
                // Update (overwrite) the account_followers_map entry in the storage
                self.account_followers_map.insert(&follow, &current_followers);

                // Emit an event to register the follow to the chain
                // but only if the caller is not the follow
                if caller != follow {
                    Self::env().emit_event(NewFollow {
                        from: caller,
                        is_following: follow,
                    });
                }
                
            }
            Ok(())
        }


        // 🟢 5 UNFOLLOW ACCOUNT 
        // allows a user to unfollow an accountId they had previously followed
        #[ink(message)]
        pub fn unfollow_account (&mut self, unfollow: AccountId
        ) -> Result<(), Error> {

            // Is this account currently being followed? If TRUE, proceed...
            let caller = Self::env().caller();
            let mut current_follows = self.account_following_map.get(&caller).unwrap_or_default();
            if current_follows.following.contains(&unfollow) {
                // remove the unfollow from the the vector of accounts they are following
                // by keeping everyone other than that account... harsh, I know.
                current_follows.following.retain(|value| *value != unfollow);
                // Update (overwrite) the account_following_map entry in the storage
                self.account_following_map.insert(&caller, &current_follows);

                // reduce the followers count for the unfollow...
                let mut current_followers = self.account_followers_map.get(&unfollow).unwrap_or_default();
                current_followers = current_followers.saturating_sub(1);
                // update (overwrite) the account_followers_map entry in the storage
                self.account_followers_map.insert(&unfollow, &current_followers);

                // Emit an event to register the unfollow to the chain
                // but only if the caller is not the unfollow
                if caller != unfollow {
                    Self::env().emit_event(NewUnFollow {
                        from: caller,
                        stopped_following: unfollow,
                    });
                }
                
            }
            // If the account is not currently being followed, ERROR: Already Not Following
            else {
                return Err(Error::NotFollowing);
            }
            Ok(())
        }


        // 🟢 6 BLOCK AN ACCOUNT
        // allows a user to block another accountId's messages in the front end
        #[ink(message)]
        pub fn block_account (&mut self, block: AccountId
        ) -> Result<(), Error> {
            // Is this account already being blocked? OR is the blocked list full?
            let caller = Self::env().caller();
            let mut current_blocked = self.account_blocked_map.get(&caller).unwrap_or_default();
            if current_blocked.blocked.contains(&block) || caller == block {
                return Err(Error::CannotBlock);
            }
            // Otherwise, update the account_blocked_map for this caller
            else {
                // if the blocked vector is full, kick out the oldest
                if current_blocked.blocked.len() > 489 {
                    current_blocked.blocked.remove(0);
                }
                // add the new block to the the vector of accounts caller is blocking
                current_blocked.blocked.push(block);
                // Update (overwrite) the account_blocked_map entry in the storage
                self.account_blocked_map.insert(&caller, &current_blocked);

                // Emit an event to register the block to the chain
                // but only if the caller is not the block
                if caller != block {
                    Self::env().emit_event(NewBlock {
                        from: caller,
                        blocked: block,
                    });
                }
                
            }
            Ok(())
        }


        // 🟢 7 UNBLOCK AN ACCOUNT 
        // allows a user to unblock an accountId they had previously blocked
        #[ink(message)]
        pub fn unblock_account (&mut self, unblock: AccountId
        ) -> Result<(), Error> {
            // Is this account currently being blocked? If TRUE, proceed...
            let caller = Self::env().caller();
            let mut current_blocked = self.account_blocked_map.get(&caller).unwrap_or_default();
            if current_blocked.blocked.contains(&unblock) {
                // remove the unblock from the the vector of accounts they are blocking
                // by keeping everyone other than that account... 
                current_blocked.blocked.retain(|value| *value != unblock);
                // Update (overwrite) the account_blocked_map entry in the storage
                self.account_blocked_map.insert(&caller, &current_blocked);

                // Emit an event to register the block to the chain
                // but only if the caller is not the unblock
                if caller != unblock {
                    Self::env().emit_event(NewUnBlock {
                        from: caller,
                        unblocked: unblock,
                    });
                }
                
            }
            // If the account is not currently being followed, ERROR: Already Not Following
            else {
                return Err(Error::NotBlocked);
            }
            Ok(())
        }


        // 🟢 8 UPDATE SETTINGS 
        // lets a user to update their list of keyword interests and other settings 
        // overwrites the mapping in contract storage
        #[ink(message)]
        pub fn update_settings (&mut self, 
            my_username: Vec<u8>,
            my_interests: Vec<u8>,
            max_messages_in_my_feed: u128,
            max_messages_in_my_paid_feed: u128,
        ) -> Result<(), Error> {
            // get the current settings for this caller and prepare the update
            let caller = Self::env().caller();
            let current_settings = self.account_settings_map.get(&caller).unwrap_or_default();
            let oldname = current_settings.username;

            // check that this user has not updated their settings in 24 hours
            let time_since_last_update = self.env().block_timestamp().saturating_sub(current_settings.last_update);
            if time_since_last_update < 86400000 {
                // send an error that interest cannot be updated so soon
                return Err(Error::CannotUpdateInterestsWithin24Hours)
            }

            // check that the set of interest keywords and username are not too long
            // maximum length is 180 which would give us 90 characters
            if my_interests.len() > 180 || my_username.len() > 180 {
                // intrests are too long, send an error
                return Err(Error::DataTooLarge)
            }

            // prepare the update
            let settings_update: Settings = Settings {
                username: my_username.clone(),
                interests: my_interests.clone(),
                max_feed: max_messages_in_my_feed,
                max_paid_feed: max_messages_in_my_paid_feed,
                last_update: self.env().block_timestamp()
            };

            // If, they are changing their username...
            if my_username != oldname {

                // they are not, by definition, the current owner of the new name
                // so if the new name exists in the the username map, it is taken, send error
                if self.username_map.contains(my_username.clone()) {
                    // if the username belongs to someone else, send an error UsernameTaken
                    return Err(Error::UsernameTaken)
                }
                else {
                    // if the new name is not in the username map, they can have it
                    // then update the username map
                    self.username_map.insert(&my_username.clone(), &caller);
                    // release the old username if the oldname it exists in the map
                    if self.username_map.contains(oldname.clone()) {
                        self.username_map.remove(oldname);
                    }
                }

            }
            
            // update their settings
            self.account_settings_map.insert(&caller, &settings_update); 

            // Emit an event to register the update to the chain
            Self::env().emit_event(SettingsUpdated {
                from: caller,
                username: my_username,
                interests: my_interests,
            }); 

            Ok(())
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>> PRIMARY GET MESSAGES <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
 

        // 🟢 9 GET PUBLIC FEED
        // given an accountId, retuns the details of thhe 3 most recent public posts sent by 
        // each account they follow, and 1 most recent endorsed post, replies on request
        #[ink(message)]
        pub fn get_public_feed(&self) -> MyFeed {
            // identify the caller
            let caller = Self::env().caller();
            // get the list of accounts they are following as a vector of AccountIds
            let accountvec = self.account_following_map.get(&caller).unwrap_or_default().following;
            // set up the return data structure
            let mut message_list: Vec<MessageDetails> = Vec::new();

            // start with the caller, who will defacto follow themselves (posts only)
            let my_idvec = self.account_messages_map.get(&caller).unwrap_or_default().messages;
            // iterate over those messages to get the details for each
            for messageidhash in my_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
                // replies on request by a different message function
            }

            // iterate over the vector of AccountIds the user is following...
            for account in accountvec.iter() {
                // for each AccountId they follow, get the list of message_ids sent from that account
                let message_idvec = self.account_messages_map.get(account).unwrap_or_default().messages;
                // iterate over those messages to get the details for each
                for messageidhash in message_idvec.iter() {
                    // get the details for that message
                    let details = self.message_map.get(&messageidhash).unwrap_or_default();
                    // add the details to the message_list vector
                    message_list.push(details);
                    // replies on request by a different message function
                }
                // then get the most recently elevated message by that account and get the details
                let elevated_id = self.account_elevated_map.get(account).unwrap_or_default();
                // get the details for that message
                let details = self.message_map.get(&elevated_id).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
            }
            // At this point you should have 3 messages sent and 1 message elevated by
            // each account you follow. It will be up to the front end to limit the display
            // and to order them by timestamp, etc. Future events-based feeds can include more.

            // package the results
            let my_feed = MyFeed {
                maxfeed: self.account_settings_map.get(&caller).unwrap_or_default().max_feed,
                blocked: self.account_blocked_map.get(&caller).unwrap_or_default().blocked,
                myfeed: message_list
            };
            // return the results
            my_feed
           
        }


        // 🟢 10 GET PAID FEED 
        // given an accountId, returns the details of every paid message, sent by anyone, that matches 
        // the interests of the caller AND still has paid endorsements available AND sufficient staked balance
        #[ink(message)]
        pub fn get_paid_feed(&self, keyword: Vec<u8>) -> MyPaidFeed {
            // set up the return data structure
            let mut message_list: Vec<PaidMessageDetails> = Vec::new();
            // make a vector of all paid message id hashes that match this keyword
            // start by defining the caller
            let caller = Self::env().caller();
            // Get the callers list of interests...
            let caller_interests = self.account_settings_map.get(&caller).unwrap_or_default().interests;
            let caller_interests_string = String::from_utf8(caller_interests.clone()).unwrap_or_default();
            // compare them to the keyword they entered
            let target_string = String::from_utf8(keyword.clone()).unwrap_or_default();
            // check to see if the caller's interests include the keyword
            if caller_interests_string.contains(&target_string) {
                // get the vector of message id hashes for that target
                let message_idvec = self.target_interests_map.get(&keyword).unwrap_or_default().messages;
                // Are there messages for those keywords?
                if message_idvec.len() > 0 {
                    // iterate over that vector of message hashes...
                    for paidmessageid in message_idvec.iter() {
                        // check to see if that message has endorsements and balance available
                        // start by getting the details for that message
                        let details = self.paid_message_map.get(&paidmessageid).unwrap_or_default();
                        if details.endorser_count < details.paid_endorser_max && details.staked_balance > 0 {
                            // add the details to the message_list vector
                            message_list.push(details);
                        }
                        // else, if there are no paid endorsements left, or 0 balance, do nothing
                        // repeat for the rest of the paid message ids under that target interest
                    }
                }                        
            }   
            // if the caller's interests do not match the target, do nothing
            // At this point, you should have a complete list of messages and all their details
            // that match the caller's interests AND have paid endorsements & balance available.

            // package the results
            let my_paid_feed = MyPaidFeed {
                maxfeed: self.account_settings_map.get(&caller).unwrap_or_default().max_paid_feed,
                myinterests: self.account_settings_map.get(&caller).unwrap_or_default().interests,
                blocked: self.account_blocked_map.get(&caller).unwrap_or_default().blocked,
                mypaidfeed: message_list
            };
            // return the results
            my_paid_feed

        }  


        // 🟢 GET THE FULL SOCIAL APP PROFILE FOR ANY GIVEN ACCOUNT 
        // Followers, Following, all messages sent and replies and elevated/endorsed 
        #[ink(message)]
        pub fn get_account_profile(&self, user: AccountId) -> SocialProfile {
            // set up the return data structures
            let mut message_list: Vec<MessageDetails> = Vec::new();
            let user_name = self.account_settings_map.get(&user).unwrap_or_default().username;
            let followers_count = self.account_followers_map.get(&user).unwrap_or_default();
            let following_list = self.account_following_map.get(&user).unwrap_or_default().following;
            
            // get the vector of sent message_ids
            let message_idvec = self.account_messages_map.get(&user).unwrap_or_default().messages;
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
                // get the reply message IDs for that message
                let reply_idvec = self.message_reply_map.get(&messageidhash).unwrap_or_default().messages;
                // for each reply, get the details and add it to the return vector
                for replyidhash in reply_idvec.iter() {
                    // get the detials for that reply
                    let replydetails = self.reply_map.get(&replyidhash).unwrap_or_default();
                    // add the details to the message_list vector
                    message_list.push(replydetails);
                }
                // loop back and do the same for each top level message from this account
            }
            
            // get the vector of endorsed messasge_ids
            let elevated_id = self.account_elevated_map.get(&user).unwrap_or_default();
            // get the details for that message
            let details = self.message_map.get(&elevated_id).unwrap_or_default();
            // add the details to the message_list vector
            message_list.push(details);
            
            // package the results
            let social_profile = SocialProfile {
                searched_account: user,
                username: user_name,
                followers: followers_count,
                following: following_list,
                message_list: message_list,
            };
            // return the results
            social_profile

        }


        // 🟢 12 SEND REPLY MESSAGE (REPLIES ONLY)
        // sends a broadcast public message as a reply to a top level message on the chain
        #[ink(message)]
        pub fn send_reply_public (&mut self, 
            new_message: Vec<u8>, 
            photo_or_youtube_link: Vec<u8>, 
            website_or_document_link: Vec<u8>, 
            replying_to: Hash
        ) -> Result<(), Error> {

            // check data limits on all inputs:
            // message 300 characters (600 length), links 300 characters (600 length)
            if new_message.len() > 600 || photo_or_youtube_link.len() > 600
            || website_or_document_link.len() > 600 {
                // error - data too large
                return Err(Error::DataTooLarge);
            }

            // Does the message exist in the top level messages? if so proceed
            if self.message_map.contains(&replying_to) {
                // get the vector of reply IDs for the original message
                let mut current_replies = self.message_reply_map.get(&replying_to).unwrap_or_default();
                
                let new_message_clone = new_message.clone();
                let new_message_clone2 = new_message.clone();
                let link_clone = photo_or_youtube_link.clone();
                let link2_clone = website_or_document_link.clone();

                // set up the data that will go into the new_message_id
                let from = Self::env().caller();
                let new_timestamp = self.env().block_timestamp();

                // create the new_message_id by hashing the above data
                let encodable = (from, new_message, new_timestamp); // Implements `scale::Encode`
                let mut new_message_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_message_id_u8);
                let new_message_id: Hash = Hash::from(new_message_id_u8);

                // SET UP THE MESSAGE DETAILS FOR THE NEW REPLY
                let caller = Self::env().caller();
                let fromusername = self.account_settings_map.get(caller).unwrap_or_default().username;
                let new_details = MessageDetails {
                    message_id: new_message_id,
                    reply_to: replying_to,
                    from_acct: Self::env().caller(),
                    username: fromusername,
                    message: new_message_clone,
                    link: photo_or_youtube_link,
                    link2: website_or_document_link,
                    endorser_count: 0,
                    reply_count: 0,
                    timestamp: self.env().block_timestamp(),
                };
                
                // WE KEEP ONLY THE most recent 100 REPLIES TO ANY ONE MESSAGE
                // if there are already 100 replies stored, kick out the oldest
                if current_replies.messages.len() > 99 {
                    let oldest = current_replies.messages[0];
                    current_replies.messages.remove(0);
                    // remove the odlest from reply_map
                    self.reply_map.remove(oldest);
                }
                // UPDATE MESSAGE_REPLY_MAP FOR ORIGINAL MESSAGE WITH THIS REPLY HASH ID
                current_replies.messages.push(new_message_id);
                // update the message_reply_map with this message hash id
                self.message_reply_map.insert(&replying_to, &current_replies);
                
                // UPDATE MESSAGE_MAP FOR THE ORIGINAL MESSAGE
                let mut original_message_details = self.message_map.get(&replying_to).unwrap_or_default();
                // increment the number of replies to the original message
                original_message_details.reply_count = original_message_details.reply_count.saturating_add(1);
                // update the message_map with the updated details for the top level message 
                if self.message_map.try_insert(&replying_to, &original_message_details).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // UPDATE THE REPLY_MAP WITH THIS REPLY'S DETAILS
                if self.reply_map.try_insert(&new_message_id, &new_details).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // EMIT EVENT to register the post to the chain
                Self::env().emit_event(MessageBroadcast {
                    from: Self::env().caller(),
                    message: new_message_clone2,
                    message_id: new_message_id,
                    link: link_clone,
                    link2: link2_clone,
                    reply_to: replying_to,
                    timestamp: self.env().block_timestamp()
                });

            }
            else {
                // if the replying_to message hash does not exist, send an error
                return Err(Error::ReplyingToMessageDoesNotExist)
            }
            
            Ok(())
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>> SECONDARY GET MESSAGES <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

        // 🟢 13 VERIFY THAT AN ACCOUNT HAS UPDATED THEIR SETTINGS AT LEAST ONCE 
        #[ink(message)]
        pub fn verify_account(&self, verify: AccountId) -> u8 {
            let mut result: u8 = 0;
            if self.account_settings_map.contains(verify) {
                result = 1;
            }
            result
        }

        // 🟢 14 GET ACCOUNT PAID MESSAGES
        // given an accountId, returns the details of stored paid messages sent by that account
        #[ink(message)]
        pub fn get_account_paid_messages(&self, user: AccountId) -> Vec<PaidMessageDetails> {
            let message_idvec = self.account_paid_messages_map.get(&user).unwrap_or_default().messages;
            let mut message_list: Vec<PaidMessageDetails> = Vec::new();
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.paid_message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
            }
            // return the vector of message details
            message_list
        }

        // 🟢 15 get the stored vector of accounts followed by a given AccountId
        #[ink(message)]
        pub fn get_account_following(&self, user: AccountId) -> Vec<AccountId> {
            self.account_following_map.get(&user).unwrap_or_default().following
        }

        // 🟢 16 Get the stored details on a paid message post, given the message_id hash.  
        #[ink(message)]
        pub fn get_details_for_paid_message(&self, message_id: Hash
        ) -> PaidMessageDetails {

            // get the details for this message
            let details = self.paid_message_map.get(&message_id).unwrap_or_default();
            // return the restuls
            details
        }

        // 🟢 17 Get the stored details on a public message post, given the message_id hash.  
        #[ink(message)]
        pub fn get_details_for_message(&self, message_id: Hash
        ) -> MessageDetails {

            // get the details for this message
            let details = self.message_map.get(&message_id).unwrap_or_default();
            // return the results
            details
        }

        // 🟢 18 Get all the replies to a single message_id hash.
        #[ink(message)]
        pub fn get_replies_for_message(&self, message_id: Hash
        ) -> Vec<MessageDetails> {
            // set up the results vector
            let mut results: Vec<MessageDetails> = Vec::new();
            // get the replies for this message
            let replies = self.message_reply_map.get(&message_id).unwrap_or_default();
            // get the details
            for id in replies.messages.iter() {
                let details = self.reply_map.get(id).unwrap_or_default();
                results.push(details);
            }
            // return the results
            results
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>> REWARD PROGRAM MESSAGES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>


        // 🟢 19 Rewards - Set Or Update Reward Root Account [RESTRICTED: ROOT]
        #[ink(message)]
        pub fn set_reward_root(&mut self, newroot: AccountId) -> Result<(), Error> {
            let caller = Self::env().caller();
            // if the root is already set, send an error
            if self.reward_root_set != 1 || self.reward_root == caller {
                // proceed - set the roots and update the storage
                self.reward_root = newroot;
                self.reward_root_set = 1;
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }

            Ok(())
        }


        // 🟢 20 Rewards - Set/Update Reward Interval and Amount [RESTRICTED: ROOT]
        // Reward coin will be given to the account that makes the Xth claim in the system
        #[ink(message)]
        pub fn set_reward(&mut self, on: u8, interval: u128, amount: Balance) -> Result<(), Error> {
            let caller = Self::env().caller();
            if self.reward_root == caller {
                // proceed to set the reward program paramteters
                self.reward_on = on;
                self.reward_interval = interval;
                self.reward_amount = amount;
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }
            
            Ok(())
        }

        // 🟢 21 ADD COIN TO REWARD ACCOUNT [RESTRICTED: ROOT]
        #[ink(message, payable)]
        pub fn add_reward_balance(&mut self) -> Result<(), Error> {
            let caller = Self::env().caller();
            if self.reward_root == caller {
                // add the paid in value to the reward_balance
                let staked: Balance = self.env().transferred_value();
                let newbalance: Balance = self.reward_balance.saturating_add(staked);
                self.reward_balance = newbalance;
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }
            
            Ok(())
        }


        // 🟢 22 RETRIEVE COIN FROM REWARD ACCOUNT [RESTRICTED: ROOT]
        // turns reward program off and returns funds to the root
        #[ink(message)]
        pub fn shut_down_reward(&mut self) -> Result<(), Error> {
            let caller = Self::env().caller();
            if self.reward_root == caller {
                // set the reward program to off
                self.reward_on = 0;
                // refund the coin to the reward root
                // Check that there is a nonzero balance on the contract > existential deposit
                if self.env().balance() > 10 && self.reward_balance > 0 {
                    // pay the root the reward_balance minus 10
                    let payout: Balance = self.reward_balance.saturating_sub(10);
                    if self.env().transfer(caller, payout).is_err() {
                        return Err(Error::PayoutFailed);
                    }
                }
                // if the balance is < 10, Error (ZeroBalance)
                else {
                    return Err(Error::ZeroBalance);
                }
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }
            
            Ok(())
        }


        // 🟢 23 GET CURRENT REWARD BALANCE AND SETTINGS [RESTRICTED: ROOT]
        #[ink(message)]
        pub fn get_reward_settings(&self) -> RewardSettings {
            let caller = Self::env().caller();
            let mut results = RewardSettings::default();
            if self.reward_root == caller {
                let settings = RewardSettings {
                    reward_on: self.reward_on,
                    reward_root_set: self.reward_root_set,
                    reward_root: self.reward_root,
                    reward_interval: self.reward_interval,
                    reward_amount: self.reward_amount,
                    reward_balance: self.reward_balance,
                    reward_payouts: self.reward_payouts,
                    claim_counter: self.claim_counter,
                };
                results = settings;
            }

            results
        }

        // END OF MESSAGE LIST

    }
    // END OF CONTRACT STORAGE

}
