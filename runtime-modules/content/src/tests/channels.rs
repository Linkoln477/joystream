#![cfg(test)]

use super::curators;
use super::mock::*;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::Currency;
use crate::*;
use frame_support::{assert_err, assert_ok};

#[test]
fn successful_channel_deletion() {
    with_default_mock_builder(|| {
        // Run to block one to see emitted events
        run_to_block(1);

        // create an account with enought balance
        let _ = balances::Module::<Test>::deposit_creating(
            &FIRST_MEMBER_ORIGIN,
            <Test as balances::Trait>::Balance::from(100u32),
        );

        // 3 assets
        let assets = NewAssets::<Test>::Upload(CreationUploadParameters {
            object_creation_list: vec![
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"first".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"second".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"third".to_vec(),
                },
            ],
            expected_data_size_fee: storage::DataObjectPerMegabyteFee::<Test>::get(),
        });

        let channel_id = NextChannelId::<Test>::get();

        // create channel
        create_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            ChannelCreationParametersRecord {
                assets: assets,
                meta: vec![],
                reward_account: None,
            },
            Ok(()),
        );

        // attempt to delete channel with non zero assets
        delete_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            channel_id,
            Err(Error::<Test>::ChannelContainsAssets.into()),
        );

        // delete assets
        let assets_to_delete = [0u64, 1u64, 2u64]
            .iter()
            .map(|&x| x)
            .collect::<BTreeSet<_>>();

        // delete channel assets
        delete_channel_assets_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            channel_id,
            assets_to_delete,
            Ok(()),
        );

        // successful deletion
        delete_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            channel_id,
            Ok(()),
        );
    })
}

#[test]
fn successful_channel_assets_deletion() {
    with_default_mock_builder(|| {
        // Run to block one to see emitted events
        run_to_block(1);

        // create an account with enought balance
        let _ = balances::Module::<Test>::deposit_creating(
            &FIRST_MEMBER_ORIGIN,
            <Test as balances::Trait>::Balance::from(100u32),
        );

        // 3 assets
        let assets = NewAssets::<Test>::Upload(CreationUploadParameters {
            object_creation_list: vec![
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"first".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"second".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"third".to_vec(),
                },
            ],
            expected_data_size_fee: storage::DataObjectPerMegabyteFee::<Test>::get(),
        });

        let channel_id = NextChannelId::<Test>::get();
        // create channel
        create_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            ChannelCreationParametersRecord {
                assets: assets,
                meta: vec![],
                reward_account: None,
            },
            Ok(()),
        );

        // delete assets
        let assets_to_delete = [0u64, 1u64].iter().map(|&x| x).collect::<BTreeSet<_>>();

        // delete channel assets
        delete_channel_assets_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            channel_id,
            assets_to_delete,
            Ok(()),
        );
    })
}

#[test]
fn succesful_channel_update() {
    with_default_mock_builder(|| {
        // Run to block one to see emitted events
        run_to_block(1);

        // create an account with enought balance
        let _ = balances::Module::<Test>::deposit_creating(
            &FIRST_MEMBER_ORIGIN,
            <Test as balances::Trait>::Balance::from(100u32),
        );

        // 2 + 1 assets to be uploaded
        let assets = NewAssets::<Test>::Upload(CreationUploadParameters {
            object_creation_list: vec![
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"first".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"second".to_vec(),
                },
            ],
            expected_data_size_fee: storage::DataObjectPerMegabyteFee::<Test>::get(),
        });

        let new_assets = NewAssets::<Test>::Upload(CreationUploadParameters {
            object_creation_list: vec![
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"first".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"second".to_vec(),
                },
            ],
            expected_data_size_fee: storage::DataObjectPerMegabyteFee::<Test>::get(),
        });

        let channel_id = NextChannelId::<Test>::get();
        // create channel
        create_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            ChannelCreationParametersRecord {
                assets: assets,
                meta: vec![],
                reward_account: None,
            },
            Ok(()),
        );

        // update channel
        update_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            channel_id,
            ChannelUpdateParametersRecord {
                assets: Some(new_assets),
                new_meta: None,
                reward_account: None,
            },
            Ok(()),
        );

        // update with 0 assets
        update_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            channel_id,
            ChannelUpdateParametersRecord {
                assets: None,
                new_meta: None,
                reward_account: None,
            },
            Ok(()),
        );
    })
}

#[test]
fn succesful_channel_creation() {
    with_default_mock_builder(|| {
        // Run to block one to see emitted events
        run_to_block(1);

        // create an account with enought balance
        let _ = balances::Module::<Test>::deposit_creating(
            &FIRST_MEMBER_ORIGIN,
            <Test as balances::Trait>::Balance::from(100u32),
        );

        // 3 assets to be uploaded
        let assets = NewAssets::<Test>::Upload(CreationUploadParameters {
            object_creation_list: vec![
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"first".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"second".to_vec(),
                },
                DataObjectCreationParameters {
                    size: 3,
                    ipfs_content_id: b"third".to_vec(),
                },
            ],
            expected_data_size_fee: storage::DataObjectPerMegabyteFee::<Test>::get(),
        });

        // create channel
        create_channel_mock(
            FIRST_MEMBER_ORIGIN,
            ContentActor::Member(FIRST_MEMBER_ID),
            ChannelCreationParametersRecord {
                assets: assets,
                meta: vec![],
                reward_account: None,
            },
            Ok(()),
        );
    })
}

#[test]
fn lead_cannot_create_channel() {
    with_default_mock_builder(|| {
        assert_err!(
            Content::create_channel(
                Origin::signed(LEAD_ORIGIN),
                ContentActor::Lead,
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ),
            Error::<Test>::ActorNotAuthorized
        );
    })
}

#[test]
fn curator_owned_channels() {
    with_default_mock_builder(|| {
        // Run to block one to see emitted events
        run_to_block(1);

        // Curator group doesn't exist yet
        assert_err!(
            Content::create_channel(
                Origin::signed(FIRST_CURATOR_ORIGIN),
                ContentActor::Curator(FIRST_CURATOR_GROUP_ID, FIRST_CURATOR_ID),
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ),
            Error::<Test>::CuratorGroupIsNotActive
        );

        let group_id = curators::add_curator_to_new_group(FIRST_CURATOR_ID);
        assert_eq!(FIRST_CURATOR_GROUP_ID, group_id);

        // Curator from wrong group
        assert_err!(
            Content::create_channel(
                Origin::signed(SECOND_CURATOR_ORIGIN),
                ContentActor::Curator(FIRST_CURATOR_GROUP_ID, SECOND_CURATOR_ID),
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ),
            Error::<Test>::CuratorIsNotAMemberOfGivenCuratorGroup
        );

        // Curator in correct active group, but wrong origin
        assert_err!(
            Content::create_channel(
                Origin::signed(SECOND_CURATOR_ORIGIN),
                ContentActor::Curator(FIRST_CURATOR_GROUP_ID, FIRST_CURATOR_ID),
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ),
            Error::<Test>::CuratorAuthFailed
        );

        let channel_id = Content::next_channel_id();

        // Curator in correct active group, with correct origin
        assert_ok!(Content::create_channel(
            Origin::signed(FIRST_CURATOR_ORIGIN),
            ContentActor::Curator(FIRST_CURATOR_GROUP_ID, FIRST_CURATOR_ID),
            ChannelCreationParametersRecord {
                assets: NewAssets::<Test>::Urls(vec![]),
                meta: vec![],
                reward_account: None,
            }
        ));

        assert_eq!(
            System::events().last().unwrap().event,
            MetaEvent::content(RawEvent::ChannelCreated(
                ContentActor::Curator(FIRST_CURATOR_GROUP_ID, FIRST_CURATOR_ID),
                channel_id,
                ChannelRecord {
                    owner: ChannelOwner::CuratorGroup(FIRST_CURATOR_GROUP_ID),
                    is_censored: false,
                    reward_account: None,
                    deletion_prize_source_account_id: FIRST_CURATOR_ORIGIN,
                    num_assets: 0,
                    num_videos: 0,
                },
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ))
        );

        // Curator can update channel
        assert_ok!(Content::update_channel(
            Origin::signed(FIRST_CURATOR_ORIGIN),
            ContentActor::Curator(FIRST_CURATOR_GROUP_ID, FIRST_CURATOR_ID),
            channel_id,
            ChannelUpdateParametersRecord {
                assets: None,
                new_meta: None,
                reward_account: None,
            }
        ));

        // Lead can update curator owned channels
        assert_ok!(Content::update_channel(
            Origin::signed(LEAD_ORIGIN),
            ContentActor::Lead,
            channel_id,
            ChannelUpdateParametersRecord {
                assets: None,
                new_meta: None,
                reward_account: None,
            }
        ));
    })
}

#[test]
fn member_owned_channels() {
    with_default_mock_builder(|| {
        // Run to block one to see emitted events
        run_to_block(1);

        // Not a member
        assert_err!(
            Content::create_channel(
                Origin::signed(UNKNOWN_ORIGIN),
                ContentActor::Member(MEMBERS_COUNT + 1),
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ),
            Error::<Test>::MemberAuthFailed
        );

        let channel_id_1 = Content::next_channel_id();

        // Member can create the channel
        assert_ok!(Content::create_channel(
            Origin::signed(FIRST_MEMBER_ORIGIN),
            ContentActor::Member(FIRST_MEMBER_ID),
            ChannelCreationParametersRecord {
                assets: NewAssets::<Test>::Urls(vec![]),
                meta: vec![],
                reward_account: None,
            }
        ));

        assert_eq!(
            System::events().last().unwrap().event,
            MetaEvent::content(RawEvent::ChannelCreated(
                ContentActor::Member(FIRST_MEMBER_ID),
                channel_id_1,
                ChannelRecord {
                    owner: ChannelOwner::Member(FIRST_MEMBER_ID),
                    is_censored: false,
                    reward_account: None,
                    deletion_prize_source_account_id: FIRST_MEMBER_ORIGIN,
                    num_assets: 0,
                    num_videos: 0,
                },
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ))
        );

        let channel_id_2 = Content::next_channel_id();

        // Member can create the channel
        assert_ok!(Content::create_channel(
            Origin::signed(SECOND_MEMBER_ORIGIN),
            ContentActor::Member(SECOND_MEMBER_ID),
            ChannelCreationParametersRecord {
                assets: NewAssets::<Test>::Urls(vec![]),
                meta: vec![],
                reward_account: None,
            }
        ));

        assert_eq!(
            System::events().last().unwrap().event,
            MetaEvent::content(RawEvent::ChannelCreated(
                ContentActor::Member(SECOND_MEMBER_ID),
                channel_id_2,
                ChannelRecord {
                    owner: ChannelOwner::Member(SECOND_MEMBER_ID),
                    is_censored: false,
                    reward_account: None,
                    deletion_prize_source_account_id: SECOND_MEMBER_ORIGIN,
                    num_assets: 0,
                    num_videos: 0,
                },
                ChannelCreationParametersRecord {
                    assets: NewAssets::<Test>::Urls(vec![]),
                    meta: vec![],
                    reward_account: None,
                }
            ))
        );

        // Update channel
        assert_ok!(Content::update_channel(
            Origin::signed(FIRST_MEMBER_ORIGIN),
            ContentActor::Member(FIRST_MEMBER_ID),
            channel_id_1,
            ChannelUpdateParametersRecord {
                assets: None,
                new_meta: None,
                reward_account: None,
            }
        ));

        assert_eq!(
            System::events().last().unwrap().event,
            MetaEvent::content(RawEvent::ChannelUpdated(
                ContentActor::Member(FIRST_MEMBER_ID),
                channel_id_1,
                ChannelRecord {
                    owner: ChannelOwner::Member(FIRST_MEMBER_ID),
                    is_censored: false,
                    reward_account: None,
                    deletion_prize_source_account_id: FIRST_MEMBER_ORIGIN,
                    num_assets: 0,
                    num_videos: 0,
                },
                ChannelUpdateParametersRecord {
                    assets: None,
                    new_meta: None,
                    reward_account: None,
                }
            ))
        );

        // Member cannot update a channel they do not own
        assert_err!(
            Content::update_channel(
                Origin::signed(FIRST_MEMBER_ORIGIN),
                ContentActor::Member(FIRST_MEMBER_ID),
                channel_id_2,
                ChannelUpdateParametersRecord {
                    assets: None,
                    new_meta: None,
                    reward_account: None,
                }
            ),
            Error::<Test>::ActorNotAuthorized
        );
    })
}

#[test]
fn channel_censoring() {
    with_default_mock_builder(|| {
        // Run to block one to see emitted events
        run_to_block(1);

        let channel_id = Content::next_channel_id();
        assert_ok!(Content::create_channel(
            Origin::signed(FIRST_MEMBER_ORIGIN),
            ContentActor::Member(FIRST_MEMBER_ID),
            ChannelCreationParametersRecord {
                assets: NewAssets::<Test>::Urls(vec![]),
                meta: vec![],
                reward_account: None,
            }
        ));

        let group_id = curators::add_curator_to_new_group(FIRST_CURATOR_ID);

        // Curator can censor channels
        let is_censored = true;
        assert_ok!(Content::update_channel_censorship_status(
            Origin::signed(FIRST_CURATOR_ORIGIN),
            ContentActor::Curator(group_id, FIRST_CURATOR_ID),
            channel_id,
            is_censored,
            vec![]
        ));

        assert_eq!(
            System::events().last().unwrap().event,
            MetaEvent::content(RawEvent::ChannelCensorshipStatusUpdated(
                ContentActor::Curator(group_id, FIRST_CURATOR_ID),
                channel_id,
                is_censored,
                vec![]
            ))
        );

        let channel = Content::channel_by_id(channel_id);

        assert!(channel.is_censored);

        // Curator can un-censor channels
        let is_censored = false;
        assert_ok!(Content::update_channel_censorship_status(
            Origin::signed(FIRST_CURATOR_ORIGIN),
            ContentActor::Curator(group_id, FIRST_CURATOR_ID),
            channel_id,
            is_censored,
            vec![]
        ));

        assert_eq!(
            System::events().last().unwrap().event,
            MetaEvent::content(RawEvent::ChannelCensorshipStatusUpdated(
                ContentActor::Curator(group_id, FIRST_CURATOR_ID),
                channel_id,
                is_censored,
                vec![]
            ))
        );

        let channel = Content::channel_by_id(channel_id);

        assert!(!channel.is_censored);

        // Member cannot censor channels
        let is_censored = true;
        assert_err!(
            Content::update_channel_censorship_status(
                Origin::signed(FIRST_MEMBER_ORIGIN),
                ContentActor::Member(FIRST_MEMBER_ID),
                channel_id,
                is_censored,
                vec![]
            ),
            Error::<Test>::ActorNotAuthorized
        );

        let curator_channel_id = Content::next_channel_id();

        // create curator channel
        assert_ok!(Content::create_channel(
            Origin::signed(FIRST_CURATOR_ORIGIN),
            ContentActor::Curator(group_id, FIRST_CURATOR_ID),
            ChannelCreationParametersRecord {
                assets: NewAssets::<Test>::Urls(vec![]),
                meta: vec![],
                reward_account: None,
            }
        ));

        // Curator cannot censor curator group channels
        assert_err!(
            Content::update_channel_censorship_status(
                Origin::signed(FIRST_CURATOR_ORIGIN),
                ContentActor::Curator(group_id, FIRST_CURATOR_ID),
                curator_channel_id,
                is_censored,
                vec![]
            ),
            Error::<Test>::CannotCensoreCuratorGroupOwnedChannels
        );

        // Lead can still censor curator group channels
        assert_ok!(Content::update_channel_censorship_status(
            Origin::signed(LEAD_ORIGIN),
            ContentActor::Lead,
            curator_channel_id,
            is_censored,
            vec![]
        ));
    })
}
