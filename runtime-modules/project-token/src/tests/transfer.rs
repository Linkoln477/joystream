#![cfg(test)]
use frame_support::{assert_noop, assert_ok, StorageDoubleMap};

use crate::tests::mock::*;
use crate::tests::test_utils::TokenDataBuilder;
use crate::types::Output;
use crate::{account, balance, last_event_eq, merkle_root, token, Error, RawEvent};

// some helpers
macro_rules! outputs {
    [$(($a:expr, $b: expr)),*] => {
        Outputs::new(vec![$(Output::<_, _> {beneficiary: $a, amount: $b},)*])
    };
}

macro_rules! origin {
    ($a: expr) => {
        Origin::signed($a)
    };
}

// permissionless transfer tests
#[test]
fn permissionless_transfer_fails_with_non_existing_token() {
    let token_id = token!(1);
    let config = GenesisConfigBuilder::new_empty().build();
    let origin = origin!(account!(1));
    let out = outputs![(account!(2), balance!(1))];

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin, token_id, out);

        assert_noop!(result, Error::<Test>::TokenDoesNotExist,);
    })
}

#[test]
fn permissionless_transfer_fails_with_non_existing_source() {
    let token_id = token!(1);
    let origin = origin!(account!(1));
    let (dst, amount) = (account!(2), balance!(100));

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(dst, amount, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin, token_id, outputs![(dst, amount)]);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn permissionless_transfer_fails_with_non_existing_destination() {
    let token_id = token!(1);
    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let src = account!(1);
    let (dst, amount) = (account!(2), balance!(100));

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs![(dst, amount)]);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn permissionless_transfer_fails_with_source_not_having_sufficient_free_balance() {
    let token_id = token!(1);
    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let (dst, amount) = (account!(1), balance!(100));
    let (src, src_balance) = (account!(2), amount - balance!(1));

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs![(dst, amount)]);

        assert_noop!(result, Error::<Test>::InsufficientFreeBalanceForTransfer);
    })
}

#[test]
fn permissionless_transfer_ok() {
    let token_id = token!(1);
    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let src = account!(1);
    let (dst, amount) = (account!(2), balance!(100));

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .with_account(dst, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs![(dst, amount)]);

        assert_ok!(result);
    })
}

#[test]
fn permissionless_transfer_ok_with_event_deposit() {
    let token_id = token!(1);
    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let src = account!(1);
    let (dst, amount) = (account!(2), balance!(100));
    let outputs = outputs![(dst, amount)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .with_account(dst, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs.clone());

        last_event_eq!(RawEvent::TokenAmountTransferred(
            token_id,
            src,
            outputs.into()
        ));
    })
}

#[test]
fn permissionless_transfer_ok_with_ex_deposit_and_without_src_removal() {
    let token_id = token!(1);
    let existential_deposit = balance!(10);
    let (dst, amount) = (account!(2), balance!(100));
    let (src, src_balance) = (account!(1), existential_deposit + amount);
    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs![(dst, amount)]);

        assert_eq!(
            src_balance.saturating_sub(amount),
            Token::account_info_by_token_and_account(token_id, src).free_balance
        );
    })
}

#[test]
fn permissionless_transfer_ok_with_ex_deposit_and_with_src_removal() {
    let token_id = token!(1);
    let existential_deposit = balance!(10u32);
    let (src, dst, amount) = (account!(1), account!(2), balance!(100));

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .with_account(dst, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs![(dst, amount)]);

        assert!(!<crate::AccountInfoByTokenAndAccount<Test>>::contains_key(
            token_id, src
        ));
    })
}

#[test]
fn permissionless_transfer_ok_with_destination_receiving_funds() {
    let token_id = token!(1);
    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let (src, amount) = (account!(1), balance!(100));
    let dst = account!(2);

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .with_account(dst, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs![(dst, amount)]);

        assert_eq!(
            Token::account_info_by_token_and_account(token_id, dst).free_balance,
            amount
        );
    })
}

#[test]
fn permissionless_transfer_ok_with_ex_deposit_and_dust_removal_from_issuance() {
    let token_id = token!(1);
    let dust = balance!(5);
    let (dst, amount) = (account!(2), balance!(100));
    let (src, src_balance) = (account!(1), dust + amount);
    let existential_deposit = balance!(10);

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs![(dst, amount)]);

        assert_eq!(
            src_balance.saturating_sub(dust),
            Token::token_info_by_id(token_id).current_total_issuance,
        );
    })
}

// multi output
#[test]
fn multiout_transfer_fails_with_non_existing_token() {
    let token_id = token!(1);
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty().build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::TokenDoesNotExist);
    })
}

#[test]
fn multiout_transfer_fails_with_non_existing_source() {
    let token_id = token!(1);
    let token_info = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_info)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn multiout_transfer_fails_with_non_existing_destination() {
    let token_id = token!(1);
    let token_info = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_info)
        .with_account(dst1, 0, 0)
        .with_account(src, amount1 + amount2, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn multiout_transfer_ok() {
    let token_id = token!(1);
    let token_info = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_info)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .with_account(src, amount1 + amount2, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_ok!(result);
    })
}

#[test]
fn multiout_transfer_ok_with_event_deposit() {
    let token_id = token!(1);
    let token_info = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_info)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .with_account(src, amount1 + amount2, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs.clone());

        last_event_eq!(RawEvent::TokenAmountTransferred(
            token_id,
            src,
            outputs.into()
        ));
    })
}

#[test]
fn multiout_transfer_fails_with_source_having_insufficient_balance() {
    let token_id = token!(1);
    let token_info = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let (src, src_balance) = (account!(1), amount1 + amount2 - 1);
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_info)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .with_account(src, src_balance, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::InsufficientFreeBalanceForTransfer);
    })
}

#[test]
fn multiout_transfer_fails_with_same_source_and_destination() {
    let token_id = token!(1);
    let token_info = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissionless)
        .build();
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_info)
        .with_account(dst1, amount1 + amount2, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(dst1), token_id, outputs);

        assert_noop!(result, Error::<Test>::SameSourceAndDestinationLocations);
    })
}

#[test]
fn permissioned_transfer_ok() {
    let token_id = token!(1);
    let (src, amount) = (account!(2), balance!(100));
    let src = account!(1);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_ok!(result);
    })
}

#[test]
fn permissioned_transfer_ok_with_event_deposit() {
    let token_id = token!(1);
    let amount = balance!(100);
    let src = account!(1);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs.clone());

        last_event_eq!(RawEvent::TokenAmountTransferred(
            token_id,
            src,
            outputs.into()
        ));
    })
}

#[test]
fn permissioned_transfer_fails_with_invalid_src() {
    let token_id = token!(1);
    let amount = balance!(100);
    let src = account!(1);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn permissioned_transfer_fails_with_invalid_destination() {
    let token_id = token!(1);
    let amount = balance!(100);
    let src = account!(1);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn permissioned_transfer_fails_with_insufficient_balance() {
    let token_id = token!(1);
    let amount = balance!(100);
    let (src, src_balance) = (account!(1), amount - 1);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::InsufficientFreeBalanceForTransfer);
    })
}

#[test]
fn permissioned_transfer_ok_without_src_removal() {
    let token_id = token!(1);
    let amount = balance!(100);
    let existential_deposit = balance!(10);
    let (src, src_balance) = (account!(1), amount + existential_deposit);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs);

        assert_eq!(
            src_balance.saturating_sub(amount),
            Token::account_info_by_token_and_account(token_id, src).free_balance
        );
    })
}

#[test]
fn permissioned_transfer_ok_with_ex_deposit_and_with_src_removal() {
    let token_id = token!(1);
    let amount = balance!(100);
    let existential_deposit = balance!(20);
    let dust = balance!(10);
    let (src, src_balance) = (account!(1), amount + dust);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs);

        assert!(!<crate::AccountInfoByTokenAndAccount<Test>>::contains_key(
            token_id, src
        ));
    })
}

#[test]
fn permissioned_transfer_ok_with_ex_deposit_and_decrease_in_issuance() {
    let token_id = token!(1);
    let amount = balance!(100);
    let existential_deposit = balance!(20);
    let dust = balance!(10);
    let (src, src_balance) = (account!(1), amount + dust);
    let (dst1, dst2) = (account!(2), account!(3));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs);

        assert_eq!(
            Token::token_info_by_id(token_id).current_total_issuance,
            src_balance - dust
        );
    })
}

#[test]
fn permissioned_multi_out_transfer_fails_with_invalid_token_id() {
    let token_id = token!(1);
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, TokenDataBuilder::new_empty().build())
        .with_account(src, amount1 + amount2, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id + 1, outputs);

        assert_noop!(result, Error::<Test>::TokenDoesNotExist);
    })
}

#[test]
fn permissioned_multi_out_transfer_fails_with_invalid_source_account() {
    let token_id = token!(1);
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn permissioned_multi_out_transfer_fails_with_invalid_destination_account() {
    let token_id = token!(1);
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount1 + amount2, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_noop!(result, Error::<Test>::AccountInformationDoesNotExist);
    })
}

#[test]
fn permissioned_multi_out_ok() {
    let token_id = token!(1);
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount1 + amount2, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let result = Token::transfer(origin!(src), token_id, outputs);

        assert_ok!(result);
    })
}

#[test]
fn permissioned_multi_out_ok_with_event_deposit() {
    let token_id = token!(1);
    let existential_deposit = balance!(20);
    let src = account!(1);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, amount1 + amount2, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs.clone());

        last_event_eq!(RawEvent::TokenAmountTransferred(
            token_id,
            src,
            outputs.into(),
        ));
    })
}

#[test]
fn permissioned_multi_out_ok_with_ex_deposit_and_without_source_removal() {
    let token_id = token!(1);
    let existential_deposit = balance!(20);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let (src, src_balance) = (account!(1), amount1 + amount2 + existential_deposit);
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs.clone());

        assert_eq!(
            Token::account_info_by_token_and_account(token_id, src).free_balance,
            src_balance - amount1 - amount2,
        );
    })
}

#[test]
fn permissioned_multi_out_ok_with_ex_deposit_and_source_removal() {
    let token_id = token!(1);
    let existential_deposit = balance!(20);
    let dust = balance!(5);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let (src, src_balance) = (account!(1), amount1 + amount2 + dust);
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs);

        assert!(!<crate::AccountInfoByTokenAndAccount<Test>>::contains_key(
            token_id, src
        ));
    })
}

#[test]
fn permissioned_multi_out_ok_with_ex_deposit_and_source_removal_and_issuance_decrease() {
    let token_id = token!(1);
    let existential_deposit = balance!(20);
    let dust = balance!(5);
    let (dst1, amount1) = (account!(2), balance!(1));
    let (dst2, amount2) = (account!(3), balance!(1));
    let (src, src_balance) = (account!(1), amount1 + amount2 + dust);
    let commit = merkle_root![dst1, dst2];
    let outputs = outputs![(dst1, amount1), (dst2, amount2)];

    let token_data = TokenDataBuilder::new_empty()
        .with_transfer_policy(Policy::Permissioned(commit))
        .with_existential_deposit(existential_deposit)
        .build();

    let config = GenesisConfigBuilder::new_empty()
        .with_token(token_id, token_data)
        .with_account(src, src_balance, 0)
        .with_account(dst1, 0, 0)
        .with_account(dst2, 0, 0)
        .build();

    build_test_externalities(config).execute_with(|| {
        let _ = Token::transfer(origin!(src), token_id, outputs);

        assert_eq!(
            Token::token_info_by_id(token_id).current_total_issuance,
            src_balance - dust
        );
    })
}
