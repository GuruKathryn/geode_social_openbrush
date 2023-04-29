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

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod geode_social {

    use ink::prelude::vec::Vec;
    use ink::prelude::vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::env::hash::{Sha2x256, HashOutput};
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
        message_id: Hash,
        from_acct: AccountId,
        username: Vec<u8>,
        message: Vec<u8>,
        link: Vec<u8>,
        endorser_count: u128,
        timestamp: u64,
        paid_endorser_max: u128,
        endorser_payment: Balance,
        target_interests: Vec<u8>,
        total_staked: Balance,
        endorsers: Vec<AccountId>,
    }

    impl Default for PaidMessageDetails {
        fn default() -> PaidMessageDetails {
            PaidMessageDetails {
                message_id: Hash::default(),
                from_acct: ZERO_ADDRESS.into(),
                username: <Vec<u8>>::default(),
                message: <Vec<u8>>::default(),
                link: <Vec<u8>>::default(),
                endorser_count: 0,
                timestamp: u64::default(),
                paid_endorser_max: u128::default(),
                endorser_payment: Balance::default(),
                target_interests: <Vec<u8>>::default(),
                total_staked: Balance::default(),
                endorsers: <Vec<AccountId>>::default(),
            }
        }
    }


    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct MessageDetails {
        message_id: Hash,
        reply_to: Hash,
        from_acct: AccountId,
        username: Vec<u8>,
        message: Vec<u8>,
        link: Vec<u8>,
        endorser_count: u128,
        reply_count: u128,
        timestamp: u64,
        endorsers: Vec<AccountId>
    }

    impl Default for MessageDetails {
        fn default() -> MessageDetails {
            MessageDetails {
                message_id: Hash::default(),
                reply_to: Hash::default(),
                from_acct: ZERO_ADDRESS.into(),
                username: <Vec<u8>>::default(),
                message: <Vec<u8>>::default(),
                link: <Vec<u8>>::default(),
                endorser_count: 0,
                reply_count: 0,
                timestamp: u64::default(),
                endorsers: <Vec<AccountId>>::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
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

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
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

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct SocialProfile {
        searched_account: AccountId,
        username: Vec<u8>,
        followers: Vec<AccountId>,
        following: Vec<AccountId>,
        message_list: Vec<MessageDetails>,
    }

    impl Default for SocialProfile {
        fn default() -> SocialProfile {
            SocialProfile {
                searched_account: ZERO_ADDRESS.into(),
                username: <Vec<u8>>::default(),
                followers: <Vec<AccountId>>::default(),
                following: <Vec<AccountId>>::default(),
                message_list: <Vec<MessageDetails>>::default(),
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
        // Blocking an account that you already blocked
        DuplicateBlock,
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
        ReentrancyError(ReentrancyGuardError),
        // Returned if the username already belongs to someone else.
        UsernameTaken,
        // if you are replying to a message that does not exist
        ReplyingToMessageDoesNotExist
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
        account_blocked_map: Mapping<AccountId, Blocked>,
        all_blocked: Vec<AccountId>,
        account_messages_map: Mapping<AccountId, Messages>,
        account_paid_messages_map: Mapping<AccountId, Messages>,
        account_elevated_map: Mapping<AccountId, Elevated>,
        account_paid_elevated_map: Mapping<AccountId, Elevated>,
        message_map: Mapping<Hash, MessageDetails>,
        message_vec: Vec<Hash>,
        paid_message_map: Mapping<Hash, PaidMessageDetails>,
        target_interests_map: Mapping<Vec<u8>, Messages>,
        target_interests_vec: Vec<Vec<u8>>,
        message_reply_map: Mapping<Hash, Messages>,
        username_map: Mapping<Vec<u8>, AccountId>,
        all_accounts_with_settings: Vec<AccountId>,
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
                account_blocked_map: Mapping::default(),
                all_blocked: <Vec<AccountId>>::default(),
                account_messages_map: Mapping::default(),
                account_paid_messages_map: Mapping::default(),
                account_elevated_map: Mapping::default(),
                account_paid_elevated_map: Mapping::default(),
                message_map: Mapping::default(),
                message_vec: <Vec<Hash>>::default(),
                paid_message_map: Mapping::default(),
                target_interests_map: Mapping::default(),
                target_interests_vec: <Vec<Vec<u8>>>::default(),
                message_reply_map: Mapping::default(),
                username_map: Mapping::default(),
                all_accounts_with_settings: <Vec<AccountId>>::default(),
            }
        }


        // MESSAGE FUNCTIONS THAT CHANGE DATA IN THE CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>
        

        // 🟢 SEND MESSAGE PUBLIC
        // sends a broadcast public message on the chain
        #[ink(message)]
        pub fn send_message_public (&mut self, 
            new_message: Vec<u8>, photo_or_other_link: Vec<u8>, replying_to: Hash
        ) -> Result<(), Error> {

            let new_message_clone = new_message.clone();
            let new_message_clone2 = new_message.clone();
            let link_clone = photo_or_other_link.clone();

            // set up the data that will go into the new_message_id
            let from = Self::env().caller();
            let new_timestamp = self.env().block_timestamp();

            // create the new_message_id by hashing the above data
            let encodable = (from, new_message, new_timestamp); // Implements `scale::Encode`
            let mut new_message_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
            ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_message_id_u8);
            let new_message_id: Hash = Hash::from(new_message_id_u8);

            // UPDATE MESSAGE REPLY MAP IF REPLYING_TO IS FILLED IN
            // if the replying_to input is blank, do nothing
            // convert the hash to a sting to use the is_empty method
            let blank: Hash = Hash::default();
            if replying_to == blank {
                // do nothing here, proceed to the next step
            }
            else {
                // if the replying_to input is not blank, check to make sure it is legit
                // if the replying_to is legit, update the message reply map and increment the reply count
                if self.message_map.contains(&replying_to) {
                    // get the vector of replies for the original message
                    let mut current_replies = self.message_reply_map.get(&replying_to).unwrap_or_default();
                    // add this message to the replies vector for that original message
                    current_replies.messages.push(new_message_id);
                    // update the message_reply_map
                    self.message_reply_map.insert(&replying_to, &current_replies);
                    // get the details for the original message
                    let original_message_details = self.message_map.get(&replying_to).unwrap_or_default();
                    // increment the number of replies to the original message
                    let new_reply_count = original_message_details.reply_count + 1;
                    // set up the updated message details for that original message
                    let orig_msg_details_update = MessageDetails {
                        message_id: original_message_details.message_id,
                        reply_to: original_message_details.reply_to,
                        from_acct: original_message_details.from_acct,
                        username: original_message_details.username,
                        message: original_message_details.message,
                        link: original_message_details.link,
                        endorser_count: original_message_details.endorser_count,
                        reply_count: new_reply_count,
                        timestamp: original_message_details.timestamp,
                        endorsers: original_message_details.endorsers
                    };
                    // update the message_map with the updated details
                    self.message_map.insert(&replying_to, &orig_msg_details_update);
                }
                else {
                    // if the replying_to message hash does not exist, send an error
                    return Err(Error::ReplyingToMessageDoesNotExist)
                }
             
            }

            // SET UP THE MESSAGE DETAILS FOR THE NEW MESSAGE
            let caller = Self::env().caller();
            let fromusername = self.account_settings_map.get(caller).unwrap_or_default().username;
            let new_details = MessageDetails {
                message_id: new_message_id,
                reply_to: replying_to,
                from_acct: Self::env().caller(),
                username: fromusername,
                message: new_message_clone,
                link: photo_or_other_link,
                endorser_count: 0,
                reply_count: 0,
                timestamp: self.env().block_timestamp(),
                endorsers: vec![Self::env().caller()]
            };

            // UPDATE MESSAGE MAP AND VECTOR
            // add the message id and its details to the message_map
            self.message_map.insert(&new_message_id, &new_details);
            self.message_vec.push(new_message_id);

            // UPDATE ACCOUNT MESSAGES MAP
            // get the messages vector for this account
            let caller = Self::env().caller();
            let mut current_messages = self.account_messages_map.get(&caller).unwrap_or_default();
            // add this message to the messages vector for this account
            current_messages.messages.push(new_message_id);
            // update the account_messages_map
            self.account_messages_map.insert(&caller, &current_messages);

            // EMIT EVENT to register the post to the chain
            Self::env().emit_event(MessageBroadcast {
                from: Self::env().caller(),
                message: new_message_clone2,
                message_id: new_message_id,
                link: link_clone,
                reply_to: replying_to,
                timestamp: self.env().block_timestamp()
            });
            
            Ok(())
        }

        
        // 🟢 SEND PAID MESSAGE PUBLIC
        // sends a paid public broadcast message post
        // and offers coin to the first X accounts to endorse/elevate the post
        #[ink(message, payable)]
        pub fn send_paid_message_public (&mut self, 
            new_message: Vec<u8>,
            photo_or_other_link: Vec<u8>,
            maximum_number_of_paid_endorsers: u128,
            target_interests: Vec<u8>
        ) -> Result<(), Error> {

            let new_message_clone = new_message.clone();
            let new_message_clone2 = new_message.clone();
            let interests_clone = target_interests.clone();
            let interests_clone2 = target_interests.clone();
            let link_clone = photo_or_other_link.clone();
            
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

            // SET PAYMENT PER ENDORSER (based on the actual staked amount)
            let payment_per_endorser: Balance = staked / maximum_number_of_paid_endorsers;

            // UPDATE THE PAID MESSAGES MAP WITH THE DETAILS
            let caller = Self::env().caller();
            let fromusername = self.account_settings_map.get(caller).unwrap_or_default().username;
            // set up the paid message details
            let new_details = PaidMessageDetails {
                    message_id: new_message_id,
                    from_acct: Self::env().caller(),
                    username: fromusername,
                    message: new_message_clone,
                    link: photo_or_other_link,
                    endorser_count: 0,
                    timestamp: self.env().block_timestamp(),
                    paid_endorser_max: maximum_number_of_paid_endorsers,
                    endorser_payment: payment_per_endorser,
                    target_interests: target_interests,
                    total_staked: staked,
                    endorsers: vec![Self::env().caller()],
            };
            // add the message id and its details to the paid message_map
            self.paid_message_map.insert(&new_message_id, &new_details);

            // UPDATE THE ACCOUNT MESSAGES MAP
            // get the messages vector for this account
            let caller = Self::env().caller();
            let mut current_messages = self.account_paid_messages_map.get(&caller).unwrap_or_default();
            // add this message to the messages vector for this account
            current_messages.messages.push(new_message_id);
            // update the account_messages_map
            self.account_paid_messages_map.insert(&caller, &current_messages);

            // UPDATE THE TARGET INTERESTS MAP
            // get the current set of messages that match this target
            let mut matching_messages = self.target_interests_map.get(&interests_clone).unwrap_or_default();
            // add the new message to the list
            matching_messages.messages.push(new_message_id);
            // update the mapping
            self.target_interests_map.insert(&interests_clone, &matching_messages);

            // UPDATE THE TARGET INTERESTS VECTOR
            // check to see if this target_interests already exists in the vector
            if self.target_interests_vec.contains(&interests_clone) {
                // if it already, exists, do nothing... but if you have to do something
                // let x = 0;
            }
            // else, if it does not already exist, add it to the target_interests_vec
            else {
                self.target_interests_vec.push(interests_clone);
            }

            // EMIT AN EVENT (to register the post to the chain)
            Self::env().emit_event(PaidMessageBroadcast {
                from: Self::env().caller(),
                message: new_message_clone2,
                message_id: new_message_id,
                link: link_clone,
                timestamp: self.env().block_timestamp(),
                paid_endorser_max: maximum_number_of_paid_endorsers,
                endorser_payment: payment_per_endorser,
                target_interests: interests_clone2,
                total_staked: staked
            });

            Ok(())

        }


        // 🟢 ELEVATE MESSAGE 
        // upvotes a public message by endorsing it on chain (unpaid) 
        #[ink(message)]
        pub fn elevate_message(&mut self, this_message_id: Hash) -> Result<(), Error> {
            
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
                    // update the endorser count
                    let new_endorser_count = current_details.endorser_count + 1;

                    // Update the details in storage for this message
                    let updated_details: MessageDetails = MessageDetails {
                        message_id: current_details.message_id,
                        reply_to: current_details.reply_to,
                        from_acct: current_details.from_acct,
                        username: current_details.username,
                        message: current_details.message,
                        link: current_details.link,
                        endorser_count: new_endorser_count,
                        reply_count: current_details.reply_count,
                        timestamp: current_details.timestamp,
                        endorsers: current_details.endorsers
                    };

                    // Update the message_map
                    self.message_map.insert(&this_message_id, &updated_details);

                    // Add this message to the account_elevated_map for this caller
                    let mut current_elevated = self.account_elevated_map.get(&caller).unwrap_or_default();
                    current_elevated.elevated.push(this_message_id);
                    self.account_elevated_map.insert(&caller, &current_elevated);

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


        // 🟢 ELEVATE PAID MESSAGE
        // endorses a paid message and pays the endorser accordingly
        #[ink(message)]
        #[openbrush::modifiers(non_reentrant)]
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
                            
                            // Add this endorser to the vector of endorsing accounts
                            current_details.endorsers.push(caller);

                            // update the endorser count
                            let new_endorser_count = current_details.endorser_count + 1;

                            // Update the details in storage for this paid message
                            let updated_details: PaidMessageDetails = PaidMessageDetails {
                                message_id: current_details.message_id,
                                from_acct: current_details.from_acct,
                                username: current_details.username,
                                message: current_details.message,
                                link: current_details.link,
                                endorser_count: new_endorser_count,
                                timestamp: current_details.timestamp,
                                paid_endorser_max: current_details.paid_endorser_max,
                                endorser_payment: current_details.endorser_payment,
                                target_interests: current_details.target_interests,
                                total_staked: current_details.total_staked,
                                endorsers: current_details.endorsers
                            };

                            // Update the paid_message_map
                            self.paid_message_map.insert(&this_message_id, &updated_details);

                            // add this message to the account_paid_elevated_map
                            let mut current_elevated = self.account_paid_elevated_map.get(&caller).unwrap_or_default();
                            current_elevated.elevated.push(this_message_id);
                            self.account_paid_elevated_map.insert(&caller, &current_elevated);

                            // Emit an event to register the endorsement to the chain
                            Self::env().emit_event(PaidMessageElevated {
                                from: updated_details.from_acct,
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
        ) -> Result<(), Error> {
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


        // 🟢 BLOCK AN ACCOUNT 
        // allows a user to follow another accountId's messages
        #[ink(message)]
        pub fn block_account (&mut self, block: AccountId
        ) -> Result<(), Error> {
            // Is this account already being blocked? If TRUE, send ERROR
            let caller = Self::env().caller();
            let mut current_blocked = self.account_blocked_map.get(&caller).unwrap_or_default();
            if current_blocked.blocked.contains(&block) {
                return Err(Error::DuplicateBlock);
            }
            // Otherwise, update the account_blocked_map for this caller
            else {
                // add the new block to the the vector of accounts caller is blocking
                current_blocked.blocked.push(block);
                // Update (overwrite) the account_blocked_map entry in the storage
                self.account_blocked_map.insert(&caller, &current_blocked);
                // add this account to the vector of all blocked accounts
                self.all_blocked.push(block);
            }
            
            Ok(())
        }


        // 🟢 UNBLOCK AN ACCOUNT 
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
            }
            // If the account is not currently being followed, ERROR: Already Not Following
            else {
                return Err(Error::NotBlocked);
            }

            Ok(())
        }


        // 🟢 UPDATE SETTINGS 
        // lets a user to update their list of keyword interests and other settings 
        // overwrites the mapping in contract storage
        #[ink(message)]
        pub fn update_settings (&mut self, 
            my_username: Vec<u8>,
            my_interests: Vec<u8>,
            max_messages_in_my_feed: u128,
            max_messages_in_my_paid_feed: u128,
        ) -> Result<(), Error> {

            let username_clone1 = my_username.clone();
            let username_clone2 = my_username.clone();
            let username_clone3 = my_username.clone();
            let interests_clone = my_interests.clone();

            // get the current settings for this caller and prepare the update
            let caller = Self::env().caller();
            let current_settings = self.account_settings_map.get(&caller).unwrap_or_default();
            let settings_update: Settings = Settings {
                username: my_username,
                interests: my_interests,
                max_feed: max_messages_in_my_feed,
                max_paid_feed: max_messages_in_my_paid_feed,
                last_update: self.env().block_timestamp()
            };
            
            // check that this user has not updated their settings in 24 hours
            let time_since_last_update = self.env().block_timestamp() - current_settings.last_update;
            if time_since_last_update < 86400000 {
                // send an error that interest cannot be updated so soon
                return Err(Error::CannotUpdateInterestsWithin24Hours)
            }
            else {
                // check that the set of interest keywords are not too long
                // maximum length is 600 which would give us 300 characters
                let interests_length = interests_clone.len();
                if interests_length > 600 {
                    // intrests are too long, send an error
                    return Err(Error::InterestsTooLong)
                }
                else {
                    // check that the username is not taken by someone else...
                    // if the username is in use already...
                    if self.username_map.contains(username_clone1) {
                        // get the account that owns that username
                        let current_owner = self.username_map.get(&username_clone2).unwrap();
                        // if the caller owns that username, update the storage maps
                        if current_owner == caller {
                            self.account_settings_map.insert(&caller, &settings_update);
                            // add this account to the vector of accounts with settings
                            if self.all_accounts_with_settings.contains(&caller) {
                                // do nothing
                            }
                            else {
                                // add the caller to the vector of accounts with settings
                                self.all_accounts_with_settings.push(caller);
                            }
                            
                        }
                        else {
                            // if the username belongs to someone else, send an error UsernameTaken
                            return Err(Error::UsernameTaken)
                        }
                    }
                    else {
                        // if the username is not already in use, update the storage map
                        self.account_settings_map.insert(&caller, &settings_update);
                        // then update the username map
                        self.username_map.insert(&username_clone3, &caller);
                        // then add this account to the vector of accounts with settings
                        if self.all_accounts_with_settings.contains(&caller) {
                            // do nothing
                        }
                        else {
                            // add the caller to the vector of accounts with settings
                            self.all_accounts_with_settings.push(caller);
                        }
                    }
                }
                
            }
            
            Ok(())
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>> PRIMARY GET MESSAGES <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
 

        // 🟢 GET PUBLIC FEED
        // given an accountId, retuns the details of all public posts sent by all accounts they follow
        // any messages elevated/endorsed by those accounts and all replies to those posts by anyone
        #[ink(message)]
        pub fn get_public_feed(&self) -> MyFeed {
            // identify the caller
            let caller = Self::env().caller();
            // get the list of accounts they are following as a vector of AccountIds
            let accountvec = self.account_following_map.get(&caller).unwrap_or_default().following;
            // set up the return data structure
            let mut message_list: Vec<MessageDetails> = Vec::new();

            // iterate over the vector of AccountIds...
            for account in accountvec.iter() {
                // for each AccountId they follow, get the list of message_ids sent from that account
                let message_idvec = self.account_messages_map.get(account).unwrap_or_default().messages;
                // iterate over those messages to get the details for each
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
                        let replydetails = self.message_map.get(&replyidhash).unwrap_or_default();
                        // add the details to the message_list vector
                        message_list.push(replydetails);
                    }
                    // loop back and do the same for each account the user is following
                }
                // then get the list of messages elevated by that account and get the details for those
                let elevated_idvec = self.account_elevated_map.get(account).unwrap_or_default().elevated;
                for messageidhash in elevated_idvec.iter() {
                    // get the details for that message
                    let details = self.message_map.get(&messageidhash).unwrap_or_default();
                    // add the details to the message_list vector
                    message_list.push(details);
                    // get the reply message IDs for that message
                    let reply_idvec = self.message_reply_map.get(&messageidhash).unwrap_or_default().messages;
                    // for each reply, get the details and add the details to the return vector
                    for replyidhash in reply_idvec.iter() {
                        // get the detials for that reply
                        let replydetails = self.message_map.get(&replyidhash).unwrap_or_default();
                        // add the details to the message_list vector
                        message_list.push(replydetails);
                    }
                    // loop back and do the same for each account
                }
                // At this point you should have all the messages sent and all the messages elevated by
                // every account you follow, and all the replies to those messages by anyone. It will be 
                // up to the front end to limit the display and to order them by timestamp, etc.
            }

            // package the results
            let my_feed = MyFeed {
                maxfeed: self.account_settings_map.get(&caller).unwrap_or_default().max_feed,
                blocked: self.account_blocked_map.get(&caller).unwrap_or_default().blocked,
                myfeed: message_list
            };
            // return the results
            my_feed
           
        }


        // 🟢 GET PAID FEED
        // given an accountId, returns the details of every paid message, sent by anyone, that matches 
        // the interests of the given accountId AND still has paid endorsements available
        #[ink(message)]
        pub fn get_paid_feed(&self) -> MyPaidFeed {
            // set up the return data structure
            let mut message_list: Vec<PaidMessageDetails> = Vec::new();
            // make a vector of all paid message id hashes that match this account's interests
            // start by defining the caller
            let caller = Self::env().caller();
            // Get the callers list of interests...
            let caller_interests = self.account_settings_map.get(&caller).unwrap_or_default().interests;
            
            // for every interest in the target_interests_map (as represented by the target interests vector)...
            for target in self.target_interests_vec.iter() {
                // check to see if the caller's interests include the target_interests
                let caller_interests_string = String::from_utf8(caller_interests.clone()).unwrap_or_default();
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
                        let endorsers_length = details.endorser_count;
                        let max_endorsements = details.paid_endorser_max;
                       
                        if endorsers_length < max_endorsements {
                            // add the details to the message_list vector
                            message_list.push(details);
                        }
                        // else, if there are no paid endorsements left, do nothing
                        // repeat for the rest of the paid message ids under that target interest
                    }

                    // if the caller's interests do not match the target, do nothing
                }   

                // repeat for the rest of the targets in the target_interest_map
            }
            // At this point, you should have a complete list of messages and all their details
            // that match the caller's interests AND have paid endorsements available.
            // It is possible that the caller has already endorsed it, but that will be caught
            // in the elevate paid message function should they try to endorse it a second time. 
            // Meanwhile, the advertiser gets a bonus by putting this message in front of this
            // user repeatedly for free until the total endorsements have run out. 

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
        // Followers, Following, all messages sent and elevated/endorsed
        #[ink(message)]
        pub fn get_account_profile(&self, user: AccountId) -> SocialProfile {
            // set up the return data structures
            let mut message_list: Vec<MessageDetails> = Vec::new();
            let user_name = self.account_settings_map.get(&user).unwrap_or_default().username;
            let followers_list = self.account_followers_map.get(&user).unwrap_or_default().followers;
            let following_list = self.account_following_map.get(&user).unwrap_or_default().following;
            // get the vector of sent message_ids
            let message_idvec = self.account_messages_map.get(&user).unwrap_or_default().messages;
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
            }
            // get the vector of endorsed messasge_ids
            let elevated_idvec = self.account_elevated_map.get(&user).unwrap_or_default().elevated;
            for messageidhash in elevated_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
            }
            // package the results
            let social_profile = SocialProfile {
                searched_account: user,
                username: user_name,
                followers: followers_list,
                following: following_list,
                message_list: message_list,
            };
            // return the results
            social_profile

        }


        // 🟢 GET REPLIES TO A SINGLE MESSAGE
        // given a message ID hash, reutrns all messages that replied to that message
        #[ink(message)]
        pub fn get_replies(&self, message_id: Hash) -> Vec<MessageDetails> {
            // set up the return data structure
            let mut message_list: Vec<MessageDetails> = Vec::new();
            // get the set of message ids that replied to the given message id
            let message_idvec = self.message_reply_map.get(&message_id).unwrap_or_default().messages;
            // iterate over those messages to get the details for each
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
            }

            // return the results
            message_list

        }


        // 🟢 SEARCH MESSAGES BY KEYWORD
        // Returns all the messages that include a given a keyword or phrase
        #[ink(message)]
        pub fn get_messages_by_keyword(&self, keywords: Vec<u8>) -> Vec<MessageDetails> {
            // set up your results vector
            let mut matching_messages: Vec<MessageDetails> = Vec::new();
            // iterate over the messages_vec
            for messagehash in self.message_vec.iter() {
                // get the details
                let details = self.message_map.get(&messagehash).unwrap_or_default();
                // check to see if the keywords are in the message
                let message_string = String::from_utf8(details.message.clone()).unwrap_or_default();
                let targetvecu8 = keywords.clone();
                let target_string = String::from_utf8(targetvecu8).unwrap_or_default();
                // if the target_string is in the message_string
                if message_string.contains(&target_string) {
                    // add it to the results vector
                    matching_messages.push(details);
                }
                //continue iterating
            }
            // return the results
            matching_messages
        }


        // 🟢 GET ACCOUNT INTERESTS KEYWORDS FOR ANALYSIS
        // returns all the data avialable about interest keywords so that the front end can 
        // offer analysis of the most popular interests, frequency per word or phrase, and
        // a search option to see how many accounts have a given interest
        #[ink(message)]
        pub fn get_interests_data(&self) -> Vec<Vec<u8>> {
            // set up the results vector
            let mut interests_data: Vec<Vec<u8>> = Vec::new();
            // for each account with settings, add their interests to the results vector
            for acct in self.all_accounts_with_settings.iter() {
                // get their interests
                let interests = self.account_settings_map.get(&acct).unwrap_or_default().interests;
                let interests_clone = interests.clone();
                // add their interests to the results vector
                interests_data.push(interests_clone);
            }
            // return the results
            interests_data
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>> SECONDARY GET MESSAGES <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

        // GET THE FULL LIST OF ALL ACCOUNTS WHO HAVE DATA IN THIER SETTINGS 
        #[ink(message)]
        pub fn get_all_accounts_with_settings(&self) -> Vec<AccountId> {
            let accts = self.all_accounts_with_settings.clone();
            accts
        }

        // GET THE FULL LIST OF ALL ACCOUNTS EVER BLOCKED BY ANYONE
        // An account might show up several times if they were blocked by several people or
        // if they were blocked repeatedly by the same person 
        #[ink(message)]
        pub fn get_all_blocked(&self) -> Vec<AccountId> {
            let all_blocked = self.all_blocked.clone();
            all_blocked
        }

        // SEE HOW MANY TIMES AN ACCOUNT WAS BLOCKED
        // the user can input an accountID and see how many times it was ever blocked
        // even if it is not currently blocked by anyone
        #[ink(message)]
        pub fn get_account_block_count(&self, user: AccountId) -> u128 {
            // set up the return data structure
            let mut block_count:u128 = 0;
            // iterate over the all_blocked vector to detect the user
            for account in self.all_blocked.iter() {
                // if the account is the one you are looking for,
                let test_account = *account; 
                if test_account == user {
                    // add one to the count
                    block_count +=1;
                }
            }
            // return the block count
            block_count
        }

        // GET ACCOUNT MESSAGES
        // given an accountId, returns the details of every unpaid message sent by that account
        #[ink(message)]
        pub fn get_account_messages(&self, user: AccountId) -> Vec<MessageDetails> {
            let message_idvec = self.account_messages_map.get(&user).unwrap_or_default().messages;
            let mut message_list: Vec<MessageDetails> = Vec::new();
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
            }
            // return the vector of message details
            message_list
        }


        // GET ACCOUNT PAID MESSAGES
        // given an accountId, returns the details of every paid message sent by that account
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


        // GET ACCOUNT ENDORSED MESSAGES
        // given an accountId, returns the details of every unpaid message they endorsed/elevated
        #[ink(message)]
        pub fn get_account_endorsed_messages(&self, user: AccountId) -> Vec<MessageDetails> {
            let message_idvec:Vec<Hash> = self.account_elevated_map.get(&user).unwrap_or_default().elevated;
            let mut message_list: Vec<MessageDetails> = Vec::new();
            for messageidhash in message_idvec.iter() {
                // get the details for that message
                let details = self.message_map.get(&messageidhash).unwrap_or_default();
                // add the details to the message_list vector
                message_list.push(details);
            }
            // return the vector of message details
            message_list
        }


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

        // Get the details on a paid message post, given the message_id hash.  
        #[ink(message)]
        pub fn get_details_for_paid_message(&self, message_id: Hash
        ) -> PaidMessageDetails {

            // get the details for this message
            let details = self.paid_message_map.get(&message_id).unwrap_or_default();
            // return the restuls
            details
        }

        // Get the details on a public message post, given the message_id hash.  
        #[ink(message)]
        pub fn get_details_for_message(&self, message_id: Hash
        ) -> MessageDetails {

            // get the details for this message
            let details = self.message_map.get(&message_id).unwrap_or_default();
            // return the results
            details
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

        // END OF MESSAGE LIST

    }
    // END OF CONTRACT STORAGE

}
