use codec::{Decode, Encode};
use frame_support::{
    dispatch::{fmt::Debug, DispatchError, DispatchResult},
    ensure,
    traits::Get,
};
use sp_arithmetic::traits::{Saturating, Unsigned, Zero};
use sp_runtime::traits::{Convert, Hash};
use sp_runtime::Permill;
use sp_std::{
    cmp::max,
    collections::btree_map::{BTreeMap, Iter},
    convert::TryInto,
    iter::Sum,
};

use storage::{BagId, DataObjectCreationParameters};

// crate imports
use crate::{errors::Error, Trait};

/// Source of tokens subject to vesting that were acquired by an account
/// either through purchase or during initial issuance
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum VestingSource {
    InitialIssuance,
    Sale(TokenSaleId),
}

/// Represent's account's split staking status
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct StakingStatus<Balance> {
    // Id of the split
    // TODO: SplitId
    split_id: u32,

    // The amount staked for the split
    amount: Balance,
}

impl<Balance: Copy> StakingStatus<Balance> {
    pub(crate) fn locks<BlockNumber>(&self, _b: BlockNumber) -> Balance {
        // TODO: Implement
        self.amount
    }
}

/// Info for the account
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct AccountData<VestingSchedule, Balance, StakingStatus> {
    /// Map that represents account's vesting schedules indexed by source.
    /// Account's total unvested (locked) balance at current block (b)
    /// can be calculated by summing `v.locks()` of all
    /// VestingSchedule (v) instances in the map.
    pub(crate) vesting_schedules: BTreeMap<VestingSource, VestingSchedule>,

    /// Represents total amount of tokens held by the account, including
    /// unvested and staked tokens.
    pub(crate) amount: Balance,

    /// Account's current split staking status
    pub(crate) split_staking_status: Option<StakingStatus>,
}

/// Info for the token
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, Debug)]
pub struct TokenData<Balance, Hash, BlockNumber, TokenSale> {
    /// Current token's total supply (tokens_issued - tokens_burned)
    pub(crate) total_supply: Balance,

    /// Total number of tokens issued
    pub(crate) tokens_issued: Balance,

    // TODO: Limit number of sales per token?
    /// Number of sales initialized, also serves as unique identifier
    /// of the current sale (`last_sale`) if any.
    pub(crate) sales_initialized: TokenSaleId,

    /// Last token sale (upcoming / ongoing / past)
    pub(crate) last_sale: Option<TokenSale>,

    /// Transfer policy
    pub(crate) transfer_policy: TransferPolicy<Hash>,

    /// Symbol used to identify token
    pub(crate) symbol: Hash,

    /// Patronage Information
    pub(crate) patronage_info: PatronageData<Balance, BlockNumber>,
}

/// Patronage information, patronage configuration = set of values for its fields
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, Debug)]
pub struct PatronageData<Balance, BlockNumber> {
    /// Patronage rate
    pub(crate) rate: Balance,

    /// Tally count for the outstanding credit before latest patronage config change
    pub(crate) unclaimed_patronage_tally_amount: Balance,

    /// Last block the patronage configuration was updated
    pub(crate) last_unclaimed_patronage_tally_block: BlockNumber,
}

/// The two possible transfer policies
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum TransferPolicy<Hash> {
    /// Permissionless
    Permissionless,

    /// Permissioned transfer with whitelist commitment
    Permissioned(Hash),
}

impl<Hash> Default for TransferPolicy<Hash> {
    fn default() -> Self {
        TransferPolicy::<Hash>::Permissionless
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
pub struct VestingScheduleParams<BlockNumber> {
    // Vesting duration
    pub(crate) duration: BlockNumber,
    // Number of blocks before the linear vesting begins
    pub(crate) blocks_before_cliff: BlockNumber,
    // Initial, instantly vested amount once linear vesting begins (percentage of total amount)
    pub(crate) cliff_amount_percentage: Permill,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
pub struct VestingSchedule<BlockNumber, Balance> {
    // Block at which the vesting begins
    pub(crate) start_block: BlockNumber,
    // Linear vesting duration
    pub(crate) duration: BlockNumber,
    // Amount instantly vested at "start_block"
    pub(crate) cliff_amount: Balance,
    // Total amount to be vested linearly over "duration" (after "start_block")
    pub(crate) post_cliff_total_amount: Balance,
}

impl<BlockNumber, Balance> VestingSchedule<BlockNumber, Balance>
where
    BlockNumber: Saturating + PartialOrd + Copy,
    Balance: Saturating + Clone + Copy + From<u32> + Unsigned + TryInto<u32> + TryInto<u64> + Ord,
{
    pub(crate) fn from_params(
        init_block: BlockNumber,
        amount: Balance,
        params: VestingScheduleParams<BlockNumber>,
    ) -> Self {
        let cliff_amount = params.cliff_amount_percentage * amount;
        Self {
            start_block: init_block.saturating_add(params.blocks_before_cliff),
            duration: params.duration,
            cliff_amount,
            post_cliff_total_amount: amount.saturating_sub(cliff_amount),
        }
    }

    pub(crate) fn locks<T: Trait<BlockNumber = BlockNumber, Balance = Balance>>(
        &self,
        b: BlockNumber,
    ) -> Balance {
        let end_block = self.start_block.saturating_add(self.duration);
        // Vesting not yet started
        if self.start_block > b {
            return self.total_amount();
        }
        // Vesting period is ongoing
        if end_block > b {
            let remaining_vesting_blocks = end_block.saturating_sub(b);
            let remaining_vesting_percentage = Permill::from_rational_approximation(
                T::BlockNumberToBalance::convert(remaining_vesting_blocks),
                T::BlockNumberToBalance::convert(self.duration),
            );
            return remaining_vesting_percentage * self.post_cliff_total_amount;
        }
        // Vesting period has finished
        Balance::zero()
    }

    pub(crate) fn is_finished(&self, b: BlockNumber) -> bool {
        self.start_block.saturating_add(self.duration) <= b
    }

    pub(crate) fn total_amount(&self) -> Balance {
        self.cliff_amount
            .saturating_add(self.post_cliff_total_amount)
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct SingleDataObjectUploadParams<JOYBalance> {
    pub object_creation_params: DataObjectCreationParameters,
    pub expected_data_size_fee: JOYBalance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct UploadContext<AccountId, BagId> {
    pub uploader_account: AccountId,
    pub bag_id: BagId,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct WhitelistParams<Hash, SingleDataObjectUploadParams> {
    /// Whitelist merkle root
    pub commitment: Hash,
    /// Optional payload data to upload to storage
    pub payload: Option<SingleDataObjectUploadParams>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct WhitelistedSaleParticipant<AccountId, Balance> {
    // Participant's address
    pub address: AccountId,
    // Cap on number of tokens participant can purchase on given sale
    pub cap: Option<Balance>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct SaleAccessProof<WhitelistedSaleParticipant, MerkleProof> {
    // Sale whitelisted participant's data to verify
    pub participant: WhitelistedSaleParticipant,
    // Merkle proof to verify against sale's whitelist commitment
    pub proof: MerkleProof,
}

impl<AccountId, Balance, Hasher>
    SaleAccessProof<WhitelistedSaleParticipant<AccountId, Balance>, MerkleProof<Hasher>>
where
    Hasher: Hash,
    AccountId: Clone + Encode + PartialEq,
    Balance: Clone + Encode,
{
    pub(crate) fn verify<T: Trait>(
        &self,
        sender: &AccountId,
        commit: Hasher::Output,
    ) -> DispatchResult {
        self.proof
            .verify::<T, WhitelistedSaleParticipant<AccountId, Balance>>(
                &self.participant,
                commit,
            )?;

        ensure!(
            self.participant.address == sender.clone(),
            Error::<T>::SaleAccessProofParticipantIsNotSender
        );

        Ok(())
    }
}

/// Input parameters describing the sale accessibility (public/private)
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum SaleAccessibilityParams<Balance, WhitelistParams> {
    /// Public sale with optional purchase cap per member
    PublicSale(Option<Balance>),
    /// Private sale (only accessible for whitelisted participants) with whitelist parameters
    PrivateSale(WhitelistParams),
}

/// Existing sale's accessibility (public/private)
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum SaleAccessibility<Balance, Hash> {
    /// Public sale with optional purchase cap per member
    PublicSale(Option<Balance>),
    /// Private sale (only accessible for whitelisted participants) with a whitelist commitment
    PrivateSale(Hash),
}

impl<Balance, Hash, SingleDataObjectUploadParams>
    From<SaleAccessibilityParams<Balance, WhitelistParams<Hash, SingleDataObjectUploadParams>>>
    for SaleAccessibility<Balance, Hash>
{
    fn from(
        params: SaleAccessibilityParams<
            Balance,
            WhitelistParams<Hash, SingleDataObjectUploadParams>,
        >,
    ) -> Self {
        match params {
            SaleAccessibilityParams::PublicSale(cap_per_member) => Self::PublicSale(cap_per_member),
            SaleAccessibilityParams::PrivateSale(whitelist_params) => {
                Self::PrivateSale(whitelist_params.commitment)
            }
        }
    }
}

impl<Balance, Hash> Default for SaleAccessibility<Balance, Hash> {
    fn default() -> Self {
        Self::PublicSale(None)
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct TokenSaleParams<
    JOYBalance,
    Balance,
    BlockNumber,
    VestingScheduleParams,
    SaleAccessibilityParams,
    AccountId,
> {
    /// Account that acts as the source of the tokens on sale
    pub tokens_source: AccountId,
    /// Token's unit price in JOY
    pub unit_price: JOYBalance,
    /// Number of tokens on sale
    pub upper_bound_quantity: Balance,
    /// Optional block in the future when the sale should start (by default: starts immediately)
    pub starts_at: Option<BlockNumber>,
    /// Sale duration in blocks
    pub duration: BlockNumber,
    /// Sale accessibility parameters (public/private + optional purchase caps)
    pub accessibility: SaleAccessibilityParams,
    /// Optional vesting schedule for all tokens on sale
    pub vesting_schedule: Option<VestingScheduleParams>,
    /// Optional sale metadata
    pub metadata: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
pub struct TokenSale<
    JOYBalance,
    Balance,
    BlockNumber,
    VestingScheduleParams,
    SaleAccessibility,
    AccountId,
> {
    /// Token's unit price in JOY
    pub unit_price: JOYBalance,
    /// Number of tokens still on sale (if any)
    pub quantity_left: Balance,
    /// Account that acts as the source of the tokens on sale
    pub tokens_source: AccountId,
    /// Block at which the sale started / will start
    pub start_block: BlockNumber,
    /// Sale duration (in blocks)
    pub duration: BlockNumber,
    /// Optional vesting schedule for all tokens on sale
    pub vesting_schedule: Option<VestingScheduleParams>,
    /// Sale accessibility (public/private + optional purchase caps)
    pub accessibility: SaleAccessibility,
}

impl<JOYBalance, Balance, BlockNumber, Hash, AccountId>
    TokenSale<JOYBalance, Balance, BlockNumber, VestingScheduleParams<BlockNumber>, Hash, AccountId>
where
    BlockNumber: Saturating + Zero + Copy + Clone + PartialOrd,
    Balance: Saturating + Clone + Copy + From<u32> + Unsigned + TryInto<u32> + TryInto<u64> + Ord,
{
    pub(crate) fn try_from_params<T: Trait>(
        params: TokenSaleParamsOf<T>,
    ) -> Result<TokenSaleOf<T>, DispatchError> {
        let current_block = <frame_system::Module<T>>::block_number();
        let start_block = params.starts_at.unwrap_or(current_block);

        ensure!(
            start_block >= current_block,
            Error::<T>::SaleStartingBlockInThePast
        );

        Ok(TokenSale {
            start_block,
            duration: params.duration,
            unit_price: params.unit_price,
            quantity_left: params.upper_bound_quantity,
            vesting_schedule: params.vesting_schedule,
            tokens_source: params.tokens_source,
            accessibility: params.accessibility.into(),
        })
    }

    pub(crate) fn get_vesting_schedule(
        &self,
        amount: Balance,
    ) -> VestingSchedule<BlockNumber, Balance> {
        self.vesting_schedule.as_ref().map_or(
            // Default VestingSchedule when none specified (distribute all tokens right after sale ends)
            VestingSchedule::<BlockNumber, Balance> {
                start_block: self.start_block.saturating_add(self.duration),
                cliff_amount: amount,
                post_cliff_total_amount: Balance::zero(),
                duration: BlockNumber::zero(),
            },
            |vs| {
                VestingSchedule::<BlockNumber, Balance>::from_params(
                    self.start_block.saturating_add(self.duration),
                    amount,
                    vs.clone(),
                )
            },
        )
    }
}

/// The possible issuance variants: This is a stub
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum OfferingState<TokenSale> {
    /// Idle state
    Idle,

    /// Upcoming sale state
    UpcomingSale(TokenSale),

    /// Active sale state
    Sale(TokenSale),

    /// state for IBCO, it might get decorated with the JOY reserve
    /// amount for the token
    BondingCurve,
}

/// Encapsules validation + IssuanceState construction
impl<TokenSale> OfferingState<TokenSale> {
    pub(crate) fn of<T: crate::Trait>(token: &TokenDataOf<T>) -> OfferingStateOf<T> {
        token
            .last_sale
            .as_ref()
            .map_or(OfferingStateOf::<T>::Idle, |sale| {
                let current_block = <frame_system::Module<T>>::block_number();
                if current_block < sale.start_block {
                    OfferingStateOf::<T>::UpcomingSale(sale.clone())
                } else if current_block >= sale.start_block
                    && current_block < sale.start_block.saturating_add(sale.duration)
                {
                    OfferingStateOf::<T>::Sale(sale.clone())
                } else {
                    OfferingStateOf::<T>::Idle
                }
            })
    }

    pub(crate) fn ensure_idle_of<T: crate::Trait>(token: &TokenDataOf<T>) -> DispatchResult {
        match Self::of::<T>(&token) {
            OfferingStateOf::<T>::Idle => Ok(()),
            _ => Err(Error::<T>::TokenIssuanceNotInIdleState.into()),
        }
    }

    pub(crate) fn ensure_upcoming_sale_of<T: crate::Trait>(
        token: &TokenDataOf<T>,
    ) -> Result<TokenSaleOf<T>, DispatchError> {
        match Self::of::<T>(&token) {
            OfferingStateOf::<T>::UpcomingSale(sale) => Ok(sale),
            _ => Err(Error::<T>::NoUpcomingSale.into()),
        }
    }

    pub(crate) fn ensure_sale_of<T: crate::Trait>(
        token: &TokenDataOf<T>,
    ) -> Result<TokenSaleOf<T>, DispatchError> {
        match Self::of::<T>(&token) {
            OfferingStateOf::<T>::Sale(sale) => Ok(sale),
            _ => Err(Error::<T>::NoActiveSale.into()),
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct InitialAllocation<AddressId, Balance, VestingScheduleParams> {
    pub(crate) address: AddressId,
    pub(crate) amount: Balance,
    pub(crate) vesting_schedule: Option<VestingScheduleParams>,
}

/// Builder for the token data struct
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, Debug)]
pub struct TokenIssuanceParameters<Balance, Hash, InitialAllocation> {
    /// Initial issuance
    pub(crate) initial_allocation: InitialAllocation,

    /// Token Symbol
    pub(crate) symbol: Hash,

    /// Initial transfer policy:
    pub(crate) transfer_policy: TransferPolicy<Hash>,

    /// Initial Patronage rate
    pub(crate) patronage_rate: Balance,
}

/// Utility enum used in merkle proof verification
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Copy)]
pub enum MerkleSide {
    /// This element appended to the right of the subtree hash
    Right,

    /// This element appended to the left of the subtree hash
    Left,
}

/// Wrapper around a merkle proof path
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct MerkleProof<Hasher: Hash>(pub Option<Vec<(Hasher::Output, MerkleSide)>>);

/// Information about a payment
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Payment<Balance> {
    /// Ignored by runtime
    pub remark: Vec<u8>,

    /// Amount
    pub amount: Balance,
}

/// Wrapper around BTreeMap<AccountId, Payment<Balance>>
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Transfers<AccountId, Balance>(pub BTreeMap<AccountId, Payment<Balance>>);

/// Default trait for Merkle Side
impl Default for MerkleSide {
    fn default() -> Self {
        MerkleSide::Right
    }
}

// implementation

/// Default trait for OfferingState
impl<TokenSale> Default for OfferingState<TokenSale> {
    fn default() -> Self {
        OfferingState::Idle
    }
}

/// Default trait for InitialAllocation
impl<AddressId: Default, Balance: Zero, VestingScheduleParams> Default
    for InitialAllocation<AddressId, Balance, VestingScheduleParams>
{
    fn default() -> Self {
        InitialAllocation {
            address: AddressId::default(),
            amount: Balance::zero(),
            vesting_schedule: None,
        }
    }
}

/// Default trait for AccountData
impl<VestingSchedule, Balance: Zero, StakingStatus> Default
    for AccountData<VestingSchedule, Balance, StakingStatus>
{
    fn default() -> Self {
        Self {
            vesting_schedules: BTreeMap::new(),
            split_staking_status: None,
            amount: Balance::zero(),
        }
    }
}

impl<Balance, BlockNumber>
    AccountData<VestingSchedule<BlockNumber, Balance>, Balance, StakingStatus<Balance>>
where
    Balance: Clone
        + Zero
        + From<u32>
        + TryInto<u32>
        + Unsigned
        + Saturating
        + Sum
        + PartialOrd
        + Ord
        + TryInto<u64>
        + Copy,
    BlockNumber: Copy + Clone + PartialOrd + Ord + Saturating + From<u32> + Unsigned,
{
    /// Check whether an account is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.amount.is_zero()
    }

    /// Calculate account's unvested balance at block `b`
    pub(crate) fn unvested<T: Trait<Balance = Balance, BlockNumber = BlockNumber>>(
        &self,
        b: BlockNumber,
    ) -> Balance {
        self.vesting_schedules
            .values()
            .map(|vs| vs.locks::<T>(b))
            .sum()
    }

    /// Calculate account's staked balance at block `b`
    pub(crate) fn staked<T: Trait<Balance = Balance, BlockNumber = BlockNumber>>(
        &self,
        b: BlockNumber,
    ) -> Balance {
        self.split_staking_status
            .as_ref()
            .map_or(Balance::zero(), |s| s.locks(b))
    }

    /// Calculate account's transferrable balance at block `b`
    pub(crate) fn transferrable<T: Trait<Balance = Balance, BlockNumber = BlockNumber>>(
        &self,
        b: BlockNumber,
    ) -> Balance {
        self.amount
            .saturating_sub(max(self.unvested::<T>(b), self.staked::<T>(b)))
    }

    pub(crate) fn ensure_can_add_or_update_vesting_schedule<
        T: Trait<Balance = Balance, BlockNumber = BlockNumber>,
    >(
        &self,
        b: BlockNumber,
        source: VestingSource,
    ) -> Result<Option<VestingSource>, DispatchError> {
        let new_entry_required = !self.vesting_schedules.contains_key(&source);
        let cleanup_required =
            self.vesting_schedules.len() == T::MaxVestingBalancesPerAccountPerToken::get() as usize;
        let cleanup_candidate = self
            .vesting_schedules
            .iter()
            .find(|(_, schedule)| schedule.is_finished(b))
            .map(|(key, _)| key.clone());

        if new_entry_required && cleanup_required && cleanup_candidate.is_none() {
            return Err(Error::<T>::MaxVestingSchedulesPerAccountPerTokenReached.into());
        }

        if cleanup_required {
            Ok(cleanup_candidate)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn add_or_update_vesting_schedule(
        &mut self,
        source: VestingSource,
        new_schedule: VestingSchedule<BlockNumber, Balance>,
        cleanup_candidate: Option<VestingSource>,
    ) {
        let existing_schedule = self.vesting_schedules.get_mut(&source);

        if let Some(vs) = existing_schedule {
            // Update existing schedule - increase amounts
            vs.cliff_amount = vs.cliff_amount.saturating_add(new_schedule.cliff_amount);
            vs.post_cliff_total_amount = vs
                .post_cliff_total_amount
                .saturating_add(new_schedule.post_cliff_total_amount);
        } else {
            // Perform cleanup if needed
            if let Some(key) = cleanup_candidate {
                self.vesting_schedules.remove(&key);
            }

            // Insert new vesting schedule
            self.vesting_schedules.insert(source, new_schedule.clone());
        }

        self.increase_amount_by(new_schedule.total_amount());
    }

    /// Increase account's total tokens amount by given amount
    pub(crate) fn increase_amount_by(&mut self, amount: Balance) {
        self.amount = self.amount.saturating_add(amount);
    }

    /// Decrease account's total tokens amount by given amount
    pub(crate) fn decrease_amount_by(&mut self, amount: Balance) {
        self.amount = self.amount.saturating_sub(amount);
    }

    /// Ensure that given amount of tokens can be transferred from the account at block `b`
    pub(crate) fn ensure_can_transfer<T: Trait<Balance = Balance, BlockNumber = BlockNumber>>(
        &self,
        b: BlockNumber,
        amount: Balance,
    ) -> DispatchResult {
        ensure!(
            self.transferrable::<T>(b) >= amount,
            crate::Error::<T>::InsufficientTransferrableBalance,
        );
        Ok(())
    }
}
/// Token Data implementation
impl<
        JOYBalance,
        Balance: Zero + Saturating + Copy,
        Hash,
        BlockNumber: PartialOrd + Saturating + Copy,
        VestingScheduleParams,
        AccountId,
        SaleAccessibility,
    >
    TokenData<
        Balance,
        Hash,
        BlockNumber,
        TokenSale<
            JOYBalance,
            Balance,
            BlockNumber,
            VestingScheduleParams,
            SaleAccessibility,
            AccountId,
        >,
    >
{
    // increase total issuance
    pub(crate) fn increase_issuance_by(&mut self, amount: Balance) {
        self.tokens_issued = self.tokens_issued.saturating_add(amount);
        self.total_supply = self.total_supply.saturating_add(amount);
    }

    // decrease total issuance (use when tokens are burned for any reason)
    #[allow(dead_code)]
    pub(crate) fn decrease_supply_by(&mut self, amount: Balance) {
        self.total_supply = self.total_supply.saturating_sub(amount);
    }

    pub fn set_unclaimed_tally_patronage_at_block(&mut self, amount: Balance, block: BlockNumber) {
        self.patronage_info.last_unclaimed_patronage_tally_block = block;
        self.patronage_info.unclaimed_patronage_tally_amount = amount;
    }

    pub(crate) fn unclaimed_patronage<BlockNumberToBalance: Convert<BlockNumber, Balance>>(
        &self,
        block: BlockNumber,
    ) -> Balance {
        // period * rate * supply + tally
        self.patronage_info
            .unclaimed_patronage_percent::<BlockNumberToBalance>(block)
            .saturating_mul(self.total_supply)
            .saturating_add(self.patronage_info.unclaimed_patronage_tally_amount)
    }

    pub fn set_new_patronage_rate_at_block<BlockNumberToBalance: Convert<BlockNumber, Balance>>(
        &mut self,
        new_rate: Balance,
        block: BlockNumber,
    ) {
        // update tally according to old rate
        self.patronage_info.unclaimed_patronage_tally_amount =
            self.unclaimed_patronage::<BlockNumberToBalance>(block);
        self.patronage_info.last_unclaimed_patronage_tally_block = block;
        self.patronage_info.rate = new_rate;
    }

    // Returns number of tokens that remain unpurchased & reserved in the the sale's
    // `tokens_source` account (if any)
    pub(crate) fn last_sale_remaining_tokens(&self) -> Balance {
        self.last_sale
            .as_ref()
            .map_or(Balance::zero(), |last_sale| last_sale.quantity_left)
    }

    pub(crate) fn from_params<T: crate::Trait>(
        params: TokenIssuanceParametersOf<T>,
    ) -> TokenDataOf<T> {
        let current_block = <frame_system::Module<T>>::block_number();

        let patronage_info =
            PatronageData::<<T as Trait>::Balance, <T as frame_system::Trait>::BlockNumber> {
                last_unclaimed_patronage_tally_block: current_block,
                unclaimed_patronage_tally_amount: <T as Trait>::Balance::zero(),
                rate: params.patronage_rate,
            };

        TokenData {
            symbol: params.symbol,
            total_supply: params.initial_allocation.amount,
            tokens_issued: params.initial_allocation.amount,
            last_sale: None,
            transfer_policy: params.transfer_policy,
            patronage_info,
            sales_initialized: 0,
        }
    }
}

impl<Hasher: Hash> MerkleProof<Hasher> {
    pub(crate) fn verify<T, S>(&self, data: &S, commit: Hasher::Output) -> DispatchResult
    where
        T: crate::Trait,
        S: Encode,
    {
        match &self.0 {
            None => Err(crate::Error::<T>::MerkleProofNotProvided.into()),
            Some(vec) => {
                let init = Hasher::hash_of(data);
                let proof_result = vec.iter().fold(init, |acc, (hash, side)| match side {
                    MerkleSide::Left => Hasher::hash_of(&(hash, acc)),
                    MerkleSide::Right => Hasher::hash_of(&(acc, hash)),
                });

                ensure!(
                    proof_result == commit,
                    crate::Error::<T>::MerkleProofVerificationFailure,
                );

                Ok(())
            }
        }
    }
}

impl<Balance: Zero + Copy + Saturating, BlockNumber: Copy + Saturating + PartialOrd>
    PatronageData<Balance, BlockNumber>
{
    pub fn unclaimed_patronage_percent<BlockNumberToBalance: Convert<BlockNumber, Balance>>(
        &self,
        block: BlockNumber,
    ) -> Balance {
        let period = block.saturating_sub(self.last_unclaimed_patronage_tally_block);
        BlockNumberToBalance::convert(period).saturating_mul(self.rate)
    }
}

impl<AccountId, Balance: Sum + Copy> Transfers<AccountId, Balance> {
    pub fn total_amount(&self) -> Balance {
        self.0.iter().map(|(_, payment)| payment.amount).sum()
    }

    pub fn iter(&self) -> Iter<'_, AccountId, Payment<Balance>> {
        self.0.iter()
    }
}

impl<AccountId, Balance> From<Transfers<AccountId, Balance>>
    for BTreeMap<AccountId, Payment<Balance>>
{
    fn from(v: Transfers<AccountId, Balance>) -> Self {
        v.0
    }
}

// Aliases

/// Alias for Staking Status
pub(crate) type StakingStatusOf<T> = StakingStatus<<T as Trait>::Balance>;

/// Alias for Account Data
pub(crate) type AccountDataOf<T> =
    AccountData<VestingScheduleOf<T>, <T as Trait>::Balance, StakingStatusOf<T>>;

/// Alias for Token Data
pub(crate) type TokenDataOf<T> = TokenData<
    <T as crate::Trait>::Balance,
    <T as frame_system::Trait>::Hash,
    <T as frame_system::Trait>::BlockNumber,
    TokenSaleOf<T>,
>;

/// Alias for InitialAllocation
pub(crate) type InitialAllocationOf<T> = InitialAllocation<
    <T as frame_system::Trait>::AccountId,
    <T as crate::Trait>::Balance,
    VestingScheduleParamsOf<T>,
>;

/// Alias for Token Issuance Parameters
pub(crate) type TokenIssuanceParametersOf<T> = TokenIssuanceParameters<
    <T as crate::Trait>::Balance,
    <T as frame_system::Trait>::Hash,
    InitialAllocationOf<T>,
>;

/// Alias for TransferPolicy
pub(crate) type TransferPolicyOf<T> = TransferPolicy<<T as frame_system::Trait>::Hash>;

/// Alias for the Merkle Proof type
pub(crate) type MerkleProofOf<T> = MerkleProof<<T as frame_system::Trait>::Hashing>;

/// Alias for VestingScheduleParams
pub(crate) type VestingScheduleParamsOf<T> =
    VestingScheduleParams<<T as frame_system::Trait>::BlockNumber>;

/// Alias for VestingSchedule
pub(crate) type VestingScheduleOf<T> =
    VestingSchedule<<T as frame_system::Trait>::BlockNumber, <T as Trait>::Balance>;

/// Alias for SingleDataObjectUploadParams
pub(crate) type SingleDataObjectUploadParamsOf<T> =
    SingleDataObjectUploadParams<<T as balances::Trait>::Balance>;

/// Alias for WhitelistParams
pub(crate) type WhitelistParamsOf<T> =
    WhitelistParams<<T as frame_system::Trait>::Hash, SingleDataObjectUploadParamsOf<T>>;

/// Alias for SaleAccessibilityParams
pub(crate) type SaleAccessibilityParamsOf<T> =
    SaleAccessibilityParams<<T as Trait>::Balance, WhitelistParamsOf<T>>;

/// Alias for SaleAccessibility
pub(crate) type SaleAccessibilityOf<T> =
    SaleAccessibility<<T as Trait>::Balance, <T as frame_system::Trait>::Hash>;

/// Alias for TokenSaleParams
pub(crate) type TokenSaleParamsOf<T> = TokenSaleParams<
    <T as balances::Trait>::Balance,
    <T as crate::Trait>::Balance,
    <T as frame_system::Trait>::BlockNumber,
    VestingScheduleParamsOf<T>,
    SaleAccessibilityParamsOf<T>,
    <T as frame_system::Trait>::AccountId,
>;

/// Alias for TokenSale
pub(crate) type TokenSaleOf<T> = TokenSale<
    <T as balances::Trait>::Balance,
    <T as crate::Trait>::Balance,
    <T as frame_system::Trait>::BlockNumber,
    VestingScheduleParamsOf<T>,
    SaleAccessibilityOf<T>,
    <T as frame_system::Trait>::AccountId,
>;

/// Alias for OfferingState
pub(crate) type OfferingStateOf<T> = OfferingState<TokenSaleOf<T>>;

/// Alias for UploadContext
pub(crate) type UploadContextOf<T> = UploadContext<<T as frame_system::Trait>::AccountId, BagId<T>>;

/// TokenSaleId
pub(crate) type TokenSaleId = u32;

/// Alias for Transfers
pub(crate) type TransfersOf<T> =
    Transfers<<T as frame_system::Trait>::AccountId, <T as crate::Trait>::Balance>;

/// Alias for WhitelistedSaleParticipant
pub(crate) type WhitelistedSaleParticipantOf<T> =
    WhitelistedSaleParticipant<<T as frame_system::Trait>::AccountId, <T as Trait>::Balance>;

/// Alias for SaleAccessProof
pub(crate) type SaleAccessProofOf<T> =
    SaleAccessProof<WhitelistedSaleParticipantOf<T>, MerkleProofOf<T>>;
