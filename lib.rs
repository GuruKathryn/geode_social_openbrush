/*
ABOUT THIS CONTRACT...
This contract allows users to post public broadcast messages to the Geode Blockchain. 
In this contract, to endorse a message is to upvote it (a kind of like button that might 
boost it's visibility in the front end). This contract also allows users to:
- follow and unfollow specific accounts,
- reply to posts, 
- declare their interests, 
- see paid messages that fit their interests, and 
- be paid in GEODE to endorse or upvote a paid message 
*/ 

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod geode_social {

    use ink::prelude::vec::Vec;
    use ink::prelude::vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use openbrush::{
        contracts::{
            reentrancy_guard::*,
            traits::errors::ReentrancyGuardError,
        },
        traits::{
            Storage,
            ZERO_ADDRESS
        },
    };

    // PRELIMINARY STORAGE STRUCTURES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct Settings {
        interests: Vec<u8>,
        password: Vec<u8>,
        bio: Vec<u8>,
        photo_url: Vec<u8>,
    }

    impl Default for Settings {
        fn default() -> Settings {
            Settings {
              interests: <Vec<u8>>::default(),
              password: <Vec<u8>>::default(),
              bio: <Vec<u8>>::default(),
              photo_url: <Vec<u8>>::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct PublicSettings {
        interests: Vec<u8>,
        bio: Vec<u8>,
        photo_url: Vec<u8>,
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
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

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct Followers {
        followers: Vec<AccountId>,
    }

    impl Default for Followers {
        fn default() -> Followers {
            Followers {
              followers: <Vec<AccountId>>::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct Messages {
        messages: Vec<Hash>,
    }

    impl Default for Messages {
        fn default() -> Messages {
            Messages {
              messages: <Vec<Hash>>::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct Elevated {
        elevated: Vec<Hash>,
    }

    impl Default for Elevated {
        fn default() -> Elevated {
            Elevated {
              elevated: <Vec<Hash>>::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct PaidMessageDetails {
        from: AccountId,
        message: Vec<u8>,
        endorsers: Vec<AccountId>,
        timestamp: u64,
        paid_endorser_max: u128,
        endorser_payment: Balance,
        target_interests: Vec<u8>,
        total_staked: Balance,
    }

    impl Default for PaidMessageDetails {
        fn default() -> PaidMessageDetails {
            PaidMessageDetails {
                from: ZERO_ADDRESS.into(),
                message: <Vec<u8>>::default(),
                endorsers: <Vec<AccountId>>::default(),
                timestamp: u64::default(),
                paid_endorser_max: u128::default(),
                endorser_payment: Balance::default(),
                target_interests: <Vec<u8>>::default(),
                total_staked: Balance::default(),
            }
        }
    }


    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct ReturnPaidMessageDetails {
        message_id: Hash,
        from: AccountId,
        message: Vec<u8>,
        timestamp: u64,
        paid_endorser_max: u128,
        endorser_payment: Balance,
        target_interests: Vec<u8>,
        total_staked: Balance,
        endorser_count: u128,
        endorsers: Vec<AccountId>,
    }


    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct MessageDetails {
        from: AccountId,
        message: Vec<u8>,
        endorsers: Vec<AccountId>,
        timestamp: u64
    }

    impl Default for MessageDetails {
        fn default() -> MessageDetails {
            MessageDetails {
                from: ZERO_ADDRESS.into(),
                message: <Vec<u8>>::default(),
                endorsers: <Vec<AccountId>>::default(),
                timestamp: u64::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct ReturnMessageDetails {
        message_id: Hash,
        from: AccountId,
        message: Vec<u8>,
        timestamp: u64,
        endorser_count: u128,
        endorsers: Vec<AccountId>,
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
        timestamp: u64
    }

    #[ink(event)]
    // Writes a broadcast reply message to the blockchain 
    pub struct ReplyMessageBroadcast {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        message: Vec<u8>,
        #[ink(topic)]
        message_id: Hash,
        reply_to_message_id: Hash,
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


    // ERROR DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        // Following an account that you follow already
        DuplicateFollow,
        // Unfollowing an account that you don't follow anyway
        NotFollowing,
        NotInFollowerList,
        // Elevating a message that does not exist
        NonexistentMessage,
        // Elevating a paid message that does not exist
        NonexistentPaidMessage,
        // Elevating the same message twice
        DuplicateEndorsement,
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
        ReentrancyError(ReentrancyGuardError),
    }

    impl From<ReentrancyGuardError> for Error {
        fn from(error:ReentrancyGuardError) -> Self {
            Error::ReentrancyError(error)
        }
    }


    // ACTUAL CONTRACT STORAGE STRUCT >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct ContractStorage {
        #[storage_field]
        guard: reentrancy_guard::Data,
        account_settings_map: Mapping<AccountId, Settings>,
        account_following_map: Mapping<AccountId, Following>,
        account_followers_map: Mapping<AccountId, Followers>,
        account_messages_map: Mapping<AccountId, Messages>,
        account_paid_messages_map: Mapping<AccountId, Messages>,
        account_elevated_map: Mapping<AccountId, Elevated>,
        account_paid_elevated_map: Mapping<AccountId, Elevated>,
        message_map: Mapping<Hash, MessageDetails>,
        paid_message_map: Mapping<Hash, PaidMessageDetails>,
        target_interests_map: Mapping<Vec<u8>, Messages>,
        target_interests_vec: Vec<Vec<u8>>,
        message_reply_map: Mapping<Hash, Messages>,
    }


    // BEGIN CONTRACT LOGIC >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    impl ContractStorage {
        
        // CONSTRUCTORS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // Constructors are implicitly payable.

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                guard: Default::default(),
                account_settings_map: Mapping::default(),
                account_following_map: Mapping::default(),
                account_followers_map: Mapping::default(),
                account_messages_map: Mapping::default(),
                account_paid_messages_map: Mapping::default(),
                account_elevated_map: Mapping::default(),
                account_paid_elevated_map: Mapping::default(),
                message_map: Mapping::default(),
                paid_message_map: Mapping::default(),
                target_interests_map: Mapping::default(),
                target_interests_vec: <Vec<Vec<u8>>>::default(),
                message_reply_map: Mapping::default(),
            }
        }


        // MESSAGE FUNCTIONS THAT CHANGE DATA IN THE CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>
        
        // 🟢 SEND MESSAGE PUBLIC
        // sends a broadcast public message on the chain
        // new_message_id is any unique hash 
        #[ink(message)]
        pub fn send_message_public (&mut self, new_message: Vec<u8>, new_message_id: Hash
        ) -> Result<(), crate::geode_social::Error> {
            // set up the message details
            let new_message_clone = new_message.clone();
            let new_details = MessageDetails {
                from: Self::env().caller(),
                message: new_message_clone,
                endorsers: vec![Self::env().caller()],
                timestamp: self.env().block_timestamp()
            };

            // add the message id and its details to the message_map
            self.message_map.insert(&new_message_id, &new_details);

            // get the messages vector for this account
            let caller = Self::env().caller();
            let mut current_messages = self.account_messages_map.get(&caller).unwrap_or_default();
            
            // add this message to the messages vector for this account
            current_messages.messages.push(new_message_id);

            // update the account_messages_map
            self.account_messages_map.insert(&caller, &current_messages);

            // emit an event to register the post to the chain
            Self::env().emit_event(MessageBroadcast {
                from: Self::env().caller(),
                message: new_message,
                message_id: new_message_id,
                timestamp: self.env().block_timestamp()
            });
            
            Ok(())
        }


        // 🟢 SEND REPLY PUBLIC 
        // sends a reply to a specific public message on the chain
        // new_message_id is any unique hash 
        #[ink(message)]
        pub fn send_reply_public (&mut self, 
            reply_to: Hash,
            new_message: Vec<u8>, 
            new_message_id: Hash
        ) -> Result<(), crate::geode_social::Error> {
            
            // UPDATE THE MESSAGE_MAP
            // set up the message details
            let new_message_clone = new_message.clone();
            let new_details = MessageDetails {
                from: Self::env().caller(),
                message: new_message_clone,
                endorsers: vec![Self::env().caller()],
                timestamp: self.env().block_timestamp()
            };
            // add the message id and its details to the message_map
            self.message_map.insert(&new_message_id, &new_details);

            // UPDATE THE ACCOUNT_MESSAGES_MAP
            // get the messages vector for this account
            let caller = Self::env().caller();
            let mut current_messages = self.account_messages_map.get(&caller).unwrap_or_default();
            // add this message to the messages vector for this account
            current_messages.messages.push(new_message_id);
            // update the account_messages_map
            self.account_messages_map.insert(&caller, &current_messages);

            // UPDATE THE MESSAGE_REPLY_MAP
            // get the vector of replies for the original message
            let mut current_replies = self.message_reply_map.get(&reply_to).unwrap_or_default();
            // add this message to the replies vector
            current_replies.messages.push(new_message_id);
            // update the message_reply_map
            self.message_reply_map.insert(&reply_to, &current_replies);
            
            // emit an event to register the post to the chain
            Self::env().emit_event(ReplyMessageBroadcast {
                from: Self::env().caller(),
                message: new_message,
                message_id: new_message_id,
                reply_to_message_id: reply_to,
                timestamp: self.env().block_timestamp()
            });
            
            Ok(())
        }

        
        // 🟢 SEND PAID MESSAGE PUBLIC
        // sends a public broadcast message post
        // and offers coin to the first X accounts to endorse the post
        // new_message_id is any unique hash
        #[ink(message, payable)]
        pub fn send_paid_message_public (&mut self, 
            new_message: Vec<u8>, 
            new_message_id: Hash,
            new_paid_endorser_max: u128,
            interests: Vec<u8>
        ) -> Result<(), crate::geode_social::Error> {

            // collect payment from the caller to fund this paid post
            // the 'payable' tag on this message allows the user to
            // send as much or as little as they like so the front end
            // will have to show the user how much that will be per endorser
            let staked: Balance = self.env().transferred_value();

            // set the payment per endorser based on the actual staked amount
            let payment_per_endorser: Balance = staked / new_paid_endorser_max;

            // set up the paid message details
            let interestsclone = interests.clone();
            let new_message_clone = new_message.clone();
            let new_details = PaidMessageDetails {
                    from: Self::env().caller(),
                    message: new_message_clone,
                    endorsers: vec![Self::env().caller()],
                    timestamp: self.env().block_timestamp(),
                    paid_endorser_max: new_paid_endorser_max,
                    endorser_payment: payment_per_endorser,
                    target_interests: interestsclone,
                    total_staked: staked,
            };
            
            // add the message id and its details to the paid message_map
            self.paid_message_map.insert(&new_message_id, &new_details);
            // get the messages vector for this account
            let caller = Self::env().caller();
            let mut current_messages = self.account_paid_messages_map.get(&caller).unwrap_or_default();
            // add this message to the messages vector for this account
            current_messages.messages.push(new_message_id);
            // update the account_messages_map
            self.account_paid_messages_map.insert(&caller, &current_messages);

            // update the target_interests_map
            // get the current set of messages that match this target
            let mut matching_messages = self.target_interests_map.get(&interests).unwrap_or_default();
            // add the new message to the list
            matching_messages.messages.push(new_message_id);
            // update the mapping
            self.target_interests_map.insert(&interests, &matching_messages);

            // update the target_interests_vec
            // check to see if this target_interests already exists in the vector
            if self.target_interests_vec.contains(&interests) {
                // if it already, exists, do nothing... but if you have to do something
                // let x = 0;
            }
            // else, if it does not already exist, add it to the target_interests_vec
            else {
                self.target_interests_vec.push(interests);
            }

            // emit an event to register the post to the chain
            Self::env().emit_event(PaidMessageBroadcast {
                from: Self::env().caller(),
                message: new_message,
                message_id: new_message_id,
                timestamp: self.env().block_timestamp(),
                paid_endorser_max: new_paid_endorser_max,
                endorser_payment: payment_per_endorser,
                target_interests: interests,
                total_staked: staked
            });

            Ok(())

        }


        // 🟢 ELEVATE MESSAGE 
        // upvotes a public message by endorsing it on chain (unpaid) 
        #[ink(message)]
        pub fn elevate_message(
            &mut self,
            owner: AccountId,
            this_message_id: Hash
        ) -> Result<(), crate::geode_social::Error> {
            
            // Does the message_id exist in the message_map? ...
            if self.message_map.contains(&this_message_id) {

                // Get the contract caller's Account ID
                let caller = Self::env().caller();
                // Get the details for this message_id from the message_map
                let mut current_details = self.message_map.get(&this_message_id).unwrap_or_default();
               
                // Is the caller already in the endorsers list for this message?... 
                if current_details.endorsers.contains(&caller) {
                    // If TRUE, return an Error... DuplicateEndorsement
                    return Err(Error::DuplicateEndorsement)
                } 

                else {
                    // If the caller is NOT already an endorser...
                    // Update the MessageDetails for this message in the message_map
                    // Add this endorser to the vector of endorsing accounts
                    current_details.endorsers.push(caller);
                    // Update the message_map
                    self.message_map.insert(&this_message_id, &current_details);

                    // Add this message to the account_elevated_map for this caller
                    let mut current_elevated = self.account_elevated_map.get(&caller).unwrap_or_default();
                    current_elevated.elevated.push(this_message_id);
                    self.account_elevated_map.insert(&caller, &current_elevated);

                    // Emit an event to register the endorsement to the chain...
                    Self::env().emit_event(MessageElevated {
                        from: owner,
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


        // 🟢 ELEVATE PAID MESSAGE
        // endorses a paid message and pays the endorser accordingly
        // 🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑 
        // expected `Result<(), ReentrancyGuardError>` because of return type
        // expected enum `Result<_, ReentrancyGuardError>` found enum `Result<_, geode_social::Error>`
        #[ink(message)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn elevate_paid_message(&mut self, owner: AccountId, this_message_id: Hash
        ) -> Result<(), ReentrancyGuardError> {

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
                        let current_endorsement_number: u128 = current_details.endorsers.len().try_into().unwrap();
                        if current_endorsement_number < max_endorsements {
                            
                            // Add this endorser to the vector of endorsing accounts
                            current_details.endorsers.push(caller);
                            // Update the paid_message_map
                            self.paid_message_map.insert(&this_message_id, &current_details);

                            // add this message to the account_paid_elevated_map
                            let mut current_elevated = self.account_paid_elevated_map.get(&caller).unwrap_or_default();
                            current_elevated.elevated.push(this_message_id);
                            self.account_paid_elevated_map.insert(&caller, &current_elevated);

                            // Emit an event to register the endorsement to the chain
                            Self::env().emit_event(PaidMessageElevated {
                                from: owner,
                                message_id: this_message_id,
                                endorser: Self::env().caller()
                            });

                            // Pay the endorser the right amount from the contract
                            // This is where the reentrancy guard comes in handy

                            // Check that there is a nonzero balance on the contract > existential deposit
                            if self.env().balance() > 10 {
                                // pay the endorser the amount of endorser_payout
                                let endorser_payout: Balance = current_details.endorser_payment;
                                self.env().transfer(caller, endorser_payout).expect("payout failed");
                                if self.env().transfer(caller, endorser_payout).is_err() {
                                    return Err(Error::EndorserPayoutFailed);
                                }
                            }
                            // if the balance is zero, Error (ZeroBalance)
                            else {
                                return Err(Error::ZeroBalance);
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


        // 🟢 FOLLOW ACCOUNT 
        // allows a user to follow another accountId's messages
        #[ink(message)]
        pub fn follow_account (&mut self, follow: AccountId
        ) -> Result<(), crate::geode_social::Error> {
            // Is this account already being followed? If TRUE, send ERROR
            let caller = Self::env().caller();
            let mut current_follows = self.account_following_map.get(&caller).unwrap_or_default();
            if current_follows.following.contains(&follow) {
                return Err(Error::DuplicateFollow);
            }
            // Otherwise, update the account_following_map for this caller
            // and the account_followers_map for this newly followed account
            else {
                // add the new follow to the the vector of accounts caller is following
                current_follows.following.push(follow);
                // Update (overwrite) the account_following_map entry in the storage
                self.account_following_map.insert(&caller, &current_follows);
                // get the vector of current followers for the followed account
                let mut current_followers = self.account_followers_map.get(&follow).unwrap_or_default(); 
                // add the caller to the vector of followers for this account
                current_followers.followers.push(caller);
                // Update (overwrite) the account_followers_map entry in the storage
                self.account_followers_map.insert(&follow, &current_followers);
            }
            
            Ok(())
        }


        // 🟢 UNFOLLOW ACCOUNT 
        // allows a user to unfollow an accountId they had previously followed
        #[ink(message)]
        pub fn unfollow_account (&mut self, unfollow: AccountId
        ) -> Result<(), crate::geode_social::Error> {

            // Is this account currently being followed? If TRUE, proceed...
            let caller = Self::env().caller();
            let mut current_follows = self.account_following_map.get(&caller).unwrap_or_default();
            if current_follows.following.contains(&unfollow) {
                // remove the unfollow from the the vector of accounts they are following
                // by keeping everyone other than that account... harsh, I know.
                current_follows.following.retain(|value| *value != unfollow);
                // Update (overwrite) the account_following_map entry in the storage
                self.account_following_map.insert(&caller, &current_follows);
            }
            // If the account is not currently being followed, ERROR: Already Not Following
            else {
                return Err(Error::NotFollowing);
            }
            
            // Is the caller in the vector of followers for the unfollow? If TRUE...
            let mut current_followers = self.account_followers_map.get(&unfollow).unwrap_or_default(); 
            if current_followers.followers.contains(&caller) {
                // remove the caller from the vector of followers
                current_followers.followers.retain(|value| *value != caller);
                // update (overwrite) the account_followers_map entry in the storage
                self.account_followers_map.insert(&unfollow, &current_followers);
            }
            else {
                return Err(Error::NotInFollowerList);
            }

            Ok(())
        }


        // 🟢 UPDATE SETTINGS 
        // allows a user to update their list of keyword interests, bio and photo_url 
        // overwrites the mapping in contract storage
        #[ink(message)]
        pub fn update_settings (&mut self, 
            updated_interests: Vec<u8>,
            updated_password: Vec<u8>,
            updated_bio: Vec<u8>,
            updated_photo_url: Vec<u8>,
        ) -> Result<(), crate::geode_social::Error> {
            // Update (overwrite) the settings for this caller
            let caller = Self::env().caller();
            let settings_update: Settings = Settings {
                interests: updated_interests,
                password: updated_password,
                bio: updated_bio,
                photo_url: updated_photo_url,
            };
            self.account_settings_map.insert(&caller, &settings_update);
            
            Ok(())
        }


        // MESSAGE FUNCTIONS THAT FETCH DATA FROM CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>

        // get the message_ids for all the messages sent by a given AccountId
        #[ink(message)]
        pub fn get_messages_sent_by_account(&self, user: AccountId) -> Vec<Hash> {
            self.account_messages_map.get(&user).unwrap_or_default().messages
        }

        // get number of messages sent by a given AccountId
        #[ink(message)]
        pub fn get_number_messages_sent_by_account(&self, user: AccountId) -> u128 {
            self.account_messages_map.get(&user).unwrap_or_default().messages.len().try_into().unwrap()
        }

        // get the PAID message_ids for all the messages sent by a given AccountId
        #[ink(message)]
        pub fn get_paid_messages_sent_by_account(&self, user: AccountId) -> Vec<Hash> {
            self.account_paid_messages_map.get(&user).unwrap_or_default().messages
        }

        // get the number of PAID messages sent by a given AccountId
        #[ink(message)]
        pub fn get_number_paid_messages_sent_by_account(&self, user: AccountId) -> u128 {
            self.account_paid_messages_map.get(&user).unwrap_or_default().messages.len().try_into().unwrap()
        }

        // get the message_ids for all the messages elevated by a given AccountId
        #[ink(message)]
        pub fn get_messages_elevated_by_account(&self, user: AccountId) -> Vec<Hash> {
            self.account_elevated_map.get(&user).unwrap_or_default().elevated
        }

        // get number of messages elevated by a given AccountId
        #[ink(message)]
        pub fn get_number_messages_elevated_by_account(&self, user: AccountId) -> u128 {
            self.account_elevated_map.get(&user).unwrap_or_default().elevated.len().try_into().unwrap()
        }

        // get the PAID message_ids for all the PAID messages elevated by a given AccountId
        #[ink(message)]
        pub fn get_paid_messages_elevated_by_account(&self, user: AccountId) -> Vec<Hash> {
            self.account_paid_elevated_map.get(&user).unwrap_or_default().elevated
        }

        // get the number of PAID messages elevated by a given AccountId
        #[ink(message)]
        pub fn get_number_paid_messages_elevated_by_account(&self, user: AccountId) -> u128 {
            self.account_paid_elevated_map.get(&user).unwrap_or_default().elevated.len().try_into().unwrap()
        }

        // get the vector of accounts followed by a given AccountId
        #[ink(message)]
        pub fn get_account_following(&self, user: AccountId) -> Vec<AccountId> {
            self.account_following_map.get(&user).unwrap_or_default().following
        }

        // get the number of accounts followed by a given AccountId
        #[ink(message)]
        pub fn get_account_following_count(&self, user: AccountId) -> u128 {
            self.account_following_map.get(&user).unwrap_or_default().following.len().try_into().unwrap()
        }

        // get the vector of accounts that are followers of a given AccountId
        #[ink(message)]
        pub fn get_followers(&self, user: AccountId) -> Vec<AccountId> {
            self.account_followers_map.get(&user).unwrap_or_default().followers
        }

        // get the number of accounts that are followers of a given AccountId
        #[ink(message)]
        pub fn get_followers_count(&self, user: AccountId) -> u128 {
            self.account_followers_map.get(&user).unwrap_or_default().followers.len().try_into().unwrap()
        }

        /* Get the details on a paid message post, given the message_id hash. 
        NOTE: The vector of endorsers is given in a separate message to not overwhelm
        the return data. Here we return the number of endorsers instead.
        
        RETURN DATA IN ORDER:
        From: AccountId
        Message: Vec<u8>, 
        # of Endorsers: u128, 
        timestamp: u64, 
        max # of paid endorsers: u128, 
        endorser payment: Balance, 
        target interests: Vec<u8>, 
        total staked: Balance
        number of paid endorsements left: u128
        */ 
        #[ink(message)]
        pub fn get_details_for_paid_message(&self, message_id: Hash
        ) -> (AccountId, Vec<u8>, u128, u64, u128, Balance, Vec<u8>, Balance, u128) {

            // get the details for this message
            let details = self.paid_message_map.get(&message_id).unwrap_or_default();
            // package it as an array to deliver to the front end
            let a = details.from;
            let b = details.message;
            let c: u128 = details.endorsers.len().try_into().unwrap();
            let d = details.timestamp;
            let e = details.paid_endorser_max;
            let f = details.endorser_payment;
            let g = details.target_interests;
            let h = details.total_staked;

            if c < e {
                let i: u128 = e - c;
                (a, b, c, d, e, f, g, h, i)
            } 
            else {
                let i: u128 = 0;
                (a, b, c, d, e, f, g, h, i)
            }
            
        }

        /* Get the details on a message post, given the message_id hash. 
        NOTE: The vector of endorsers is given in a separate message to not overwhelm
        the return data. Here we return the number of endorsers instead.
        
        RETURN DATA IN ORDER:
        From: AccountId
        Message: Vec<u8>, 
        # of Endorsers: u128, 
        timestamp: u64, 
        */ 
        #[ink(message)]
        pub fn get_details_for_message(&self, message_id: Hash
        ) -> (AccountId, Vec<u8>, u128, u64) {

            // get the details for this message
            let details = self.message_map.get(&message_id).unwrap_or_default();
            // package it as an array to deliver to the front end
            let a = details.from;
            let b = details.message;
            let c :u128 = details.endorsers.len().try_into().unwrap();
            let d = details.timestamp;
            (a, b, c, d)

        }

        // get the vector of endorsers for a given message
        #[ink(message)]
        pub fn get_endorser_list_for_message(&self, message_id: Hash) -> Vec<AccountId> {
            self.message_map.get(&message_id).unwrap_or_default().endorsers
        }

        // get the vector of endorsers for a given PAID message
        #[ink(message)]
        pub fn get_endorser_list_for_paid_message(&self, message_id: Hash) -> Vec<AccountId> {
            self.paid_message_map.get(&message_id).unwrap_or_default().endorsers
        }

        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>> UPDATED GET MESSAGES <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
 
        // 🟢 GET ACCOUNT SETTINGS
        // get the current settings for a given AccountId (but don't show the password)
        #[ink(message)]
        pub fn get_account_settings(&self, user: AccountId) -> PublicSettings {
            let current_settings = self.account_settings_map.get(&user).unwrap_or_default();
            let current_interests = current_settings.interests;
            let current_bio = current_settings.bio;
            let current_photo_url = current_settings.photo_url;
            let public_settings = PublicSettings {
                interests: current_interests,
                bio: current_bio,
                photo_url: current_photo_url,
            };
            public_settings
        }

        // 🟢 GET ACCOUNT MESSAGES
        // given an accountId, returns the details of every unpaid message sent by that account
        #[ink(message)]
        pub fn get_account_messages(&self, user: AccountId) -> Vec<ReturnMessageDetails> {
            let message_idvec = self.account_messages_map.get(&user).unwrap_or_default().messages;
            let mut message_list: Vec<ReturnMessageDetails> = Vec::new();
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                let endorsers_length = details.endorsers.len();
                let endorsement_count = u128::try_from(endorsers_length).unwrap() - 1;
                // package them in the ReturnMessageDetails format
                let return_details = ReturnMessageDetails {
                    message_id: *messageidhash,
                    from: details.from,
                    message: details.message,
                    timestamp: details.timestamp,
                    endorser_count: endorsement_count,
                    endorsers: details.endorsers,
                };
                // add the details to the message_list vector
                message_list.push(return_details);
            }
            // return the vector of message details
            message_list
        }


        // 🟢 GET ACCOUNT PAID MESSAGES
        // given an accountId, returns the details of every paid message sent by that account
        #[ink(message)]
        pub fn get_account_paid_messages(&self, user: AccountId) -> Vec<ReturnPaidMessageDetails> {
            let message_idvec = self.account_paid_messages_map.get(&user).unwrap_or_default().messages;
            let mut message_list: Vec<ReturnPaidMessageDetails> = Vec::new();
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.paid_message_map.get(&messageidhash).unwrap_or_default();
                let endorsers_length = details.endorsers.len();
                let endorsement_count = u128::try_from(endorsers_length).unwrap() - 1;
                // package them in the ReturnPaidMessageDetails format
                let return_details = ReturnPaidMessageDetails {
                    message_id: *messageidhash,
                    from: details.from,
                    message: details.message,
                    timestamp: details.timestamp,
                    paid_endorser_max: details.paid_endorser_max,
                    endorser_payment: details.endorser_payment,
                    target_interests: details.target_interests,
                    total_staked: details.total_staked,
                    endorser_count: endorsement_count,
                    endorsers: details.endorsers,
                };
                // add the details to the message_list vector
                message_list.push(return_details);
            }
            // return the vector of message details
            message_list
        }


        // 🟢 GET ACCOUNT ENDORSED MESSAGES
        // given an accountId, returns the details of every unpaid message they endorsed/elevated
        #[ink(message)]
        pub fn get_account_endorsed_messages(&self, user: AccountId) -> Vec<ReturnMessageDetails> {
            let message_idvec:Vec<Hash> = self.account_elevated_map.get(&user).unwrap_or_default().elevated;
            let mut message_list: Vec<ReturnMessageDetails> = Vec::new();
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                let endorsers_length = details.endorsers.len();
                let endorsement_count = u128::try_from(endorsers_length).unwrap() - 1;
                // package them in the ReturnMessageDetails format
                let return_details = ReturnMessageDetails {
                    message_id: *messageidhash,
                    from: details.from,
                    message: details.message,
                    timestamp: details.timestamp,
                    endorser_count: endorsement_count,
                    endorsers: details.endorsers,
                };
                // add the details to the message_list vector
                message_list.push(return_details);
            }
            // return the vector of message details
            message_list
        }

        // GET REPLIES
        // given a message ID hash, reutrns all messages that replied to that message
        #[ink(message)]
        pub fn get_replies(&self, message_id: Hash) -> Vec<ReturnMessageDetails> {
            // set up the return data structure
            let mut message_list: Vec<ReturnMessageDetails> = Vec::new();
            // get the set of message ids that replied to the given message id
            let message_idvec = self.message_reply_map.get(&message_id).unwrap_or_default().messages;
            // iterate over those messages to get the details for each
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                let endorsers_length = details.endorsers.len();
                let endorsement_count = u128::try_from(endorsers_length).unwrap() - 1;
                // package them in the ReturnMessageDetails format
                let return_details = ReturnMessageDetails {
                    message_id: *messageidhash,
                    from: details.from,
                    message: details.message,
                    timestamp: details.timestamp,
                    endorser_count: endorsement_count,
                    endorsers: details.endorsers,
                };
                // add the details to the message_list vector
                message_list.push(return_details);
            }

            // return the results
            message_list

        }


        // 🟢 GET PUBLIC FEED
        // given an accountId, retuns the details of all public posts sent by all accounts they follow
        #[ink(message)]
        pub fn get_public_feed(&self) -> Vec<ReturnMessageDetails> {
            // get the list of accounts they are following as a vector of AccountIds
            let caller = Self::env().caller();
            let accountvec = self.account_following_map.get(&caller).unwrap_or_default().following;
            // set up the return data structure
            let mut message_list: Vec<ReturnMessageDetails> = Vec::new();
            // iterate over the vector of AccountIds...
            for account in accountvec.iter() {
                // for each AccountId they follow, get the list of message_ids from that account
                let message_idvec = self.account_messages_map.get(account).unwrap_or_default().messages;
                // iterate over those messages to get the details for each
                for messageidhash in message_idvec.iter() {
                    // get the details for that message
                    let details = self.message_map.get(&messageidhash).unwrap_or_default();
                    let endorsers_length = details.endorsers.len();
                    let endorsement_count = u128::try_from(endorsers_length).unwrap() - 1;
                    // package them in the ReturnMessageDetails format
                    let return_details = ReturnMessageDetails {
                        message_id: *messageidhash,
                        from: details.from,
                        message: details.message,
                        timestamp: details.timestamp,
                        endorser_count: endorsement_count,
                        endorsers: details.endorsers,
                    };
                    // add the details to the message_list vector
                    message_list.push(return_details);
                }
                // loop back and do the same for each account
            }
            // return the results
            message_list

        }


        // 🟢 GET PAID FEED
        // given an accountId, returns the details of every paid message, sent by anyone, that matches 
        // the interests of the given accountId AND still has paid endorsements available
        #[ink(message)]
        pub fn get_paid_feed(&self) -> Vec<ReturnPaidMessageDetails> {
            // set up the return data structure
            let mut message_list: Vec<ReturnPaidMessageDetails> = Vec::new();
            // make a vector of all paid message id hashes that match this account's interests
            // start by defining the caller
            let caller = Self::env().caller();
            // Get the callers list of interests...
            let caller_interests = self.account_settings_map.get(&caller).unwrap_or_default().interests;
            
            // for every interest in the target_interests_map (as represented by the target interests vector)...
            for target in self.target_interests_vec.iter() {
                // check to see if the caller's interests include the target_interests
                let caller_interests_string = String::from_utf8(caller_interests).unwrap_or_default();
                let targetvecu8 = target.clone();
                let target_string = String::from_utf8(targetvecu8).unwrap_or_default();
                
                if caller_interests_string.contains(&target_string) {
                    
                    // get the vector of message id hashes for that target
                    let message_idvec = self.target_interests_map.get(&target).unwrap_or_default().messages;
                    
                    // iterate over that vector of message hashes...
                    for paidmessageid in message_idvec.iter() {
                        
                        // check to see if that message has any endorsements available
                        // start by getting the details for that message
                        let details = self.paid_message_map.get(&paidmessageid).unwrap_or_default();
                        let endorsers_length = details.endorsers.len();
                        let endorsement_count = u128::try_from(endorsers_length).unwrap() - 1;
                        let max_endorsements = details.paid_endorser_max;
                       
                        if endorsement_count < max_endorsements {
                            
                            // package the message details and add it to the return vector
                            let return_details = ReturnPaidMessageDetails {
                                message_id: *paidmessageid,
                                from: details.from,
                                message: details.message,
                                timestamp: details.timestamp,
                                paid_endorser_max: details.paid_endorser_max,
                                endorser_payment: details.endorser_payment,
                                target_interests: details.target_interests,
                                total_staked: details.total_staked,
                                endorser_count: endorsement_count,
                                endorsers: details.endorsers,
                            };
                            // add the details to the message_list vector
                            message_list.push(return_details);
                        }
                        // else, if there are no paid endorsements left, do nothing
                        // repeat for the rest of the paid message ids under that target interest
                    }

                    // if the caller's interests do not match the target, do nothing
                }   

                // repeat for the rest of the targets in the target_interest_map
            }
            // at this point, you should have a complete list of messages and all their details
            // that match the caller's interests AND have paid endorsements avaialble
            // it is possible that the caller has already endorsed it, but that will be caught
            // in the endorse function should they try to endorse it a second time. 
            // meanwhile, the advertiser gets a bonus by putting this message in front of this
            // user repeatedly for free until the total endorsements have run out. 

            // return the vector of message details for display
            message_list

        }      

        // END OF MESSAGE LIST

    }
    // END OF CONTRACT STORAGE

}
