// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unused_unit)]
#![cfg_attr(
    not(any(test, feature = "runtime-benchmarks")),
    deny(clippy::panic),
    deny(clippy::panic_in_result_fn),
    deny(clippy::unwrap_used),
    deny(clippy::expect_used),
    deny(clippy::indexing_slicing),
    deny(clippy::integer_arithmetic),
    deny(clippy::match_on_vec_items),
    deny(clippy::unreachable)
)]

#[cfg(not(any(test, feature = "runtime-benchmarks")))]
#[allow(unused_imports)]
#[macro_use]
extern crate common;

use common::bloat_bond::{RepayableBloatBond, RepayableBloatBondOf};
use common::costs::{burn_from_usable, has_sufficient_balance_for_fees, pay_fee};
use common::to_kb;
use frame_support::BoundedBTreeSet;
#[cfg(feature = "std")]
pub use serde::{Deserialize, Serialize};

use codec::{Codec, Decode, Encode, MaxEncodedLen};
pub use frame_support::dispatch::DispatchResult;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get, PalletId, Parameter,
};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_arithmetic::traits::{BaseArithmetic, One, Zero};
pub use sp_io::storage::clear_prefix;
use sp_runtime::traits::{AccountIdConversion, MaybeSerialize, Member, Saturating};
use sp_runtime::DispatchError;
use sp_runtime::SaturatedConversion;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::TryInto;
use sp_std::fmt::Debug;
use sp_std::prelude::*;

use common::membership::MemberOriginValidator;
use common::working_group::WorkingGroupAuthenticator;

mod benchmarking;
mod mock;
mod tests;
pub mod weights;
pub use weights::WeightInfo;

/// Type for keeping track of number of posts in a thread
pub type NumberOfPosts = u64;

/// Moderator ID alias for the actor of the system.
pub type ModeratorId<T> = common::ActorId<T>;

/// Forum user ID alias for the member of the system.
pub type ForumUserId<T> = common::MemberId<T>;

type WeightInfoForum<T> = <T as Config>::WeightInfo;

/// Balance alias for `balances` module.
pub type BalanceOf<T> = <T as balances::Config>::Balance;

/// Alias for stckied thread ids (bounded btreeset)
pub type StickiedThreadIds<T> =
    BoundedBTreeSet<<T as Config>::ThreadId, <T as Config>::MaxStickiedThreads>;

/// Alias for the category
pub type CategoryOf<T> =
    Category<<T as Config>::CategoryId, <T as frame_system::Config>::Hash, StickiedThreadIds<T>>;

/// Alias for the thread
pub type ThreadOf<T> = Thread<ForumUserId<T>, <T as Config>::CategoryId, RepayableBloatBondOf<T>>;

/// Alias for the post
pub type PostOf<T> = Post<
    ForumUserId<T>,
    <T as Config>::ThreadId,
    <T as frame_system::Config>::Hash,
    <T as frame_system::Config>::BlockNumber,
    RepayableBloatBondOf<T>,
>;

/// Type alias for `ExtendedPostIdObject`
pub type ExtendedPostId<T> =
    ExtendedPostIdObject<<T as Config>::CategoryId, <T as Config>::ThreadId, <T as Config>::PostId>;

/// Extended post id representation
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, TypeInfo)]
pub struct ExtendedPostIdObject<CategoryId, ThreadId, PostId> {
    pub category_id: CategoryId,
    pub thread_id: ThreadId,
    pub post_id: PostId,
}

pub trait Config:
    frame_system::Config
    + pallet_timestamp::Config
    + common::membership::MembershipTypes
    + balances::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type CategoryId: Parameter
        + Member
        + BaseArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq
        + From<u64>
        + Into<u64>
        + MaxEncodedLen;

    type ThreadId: Parameter
        + Member
        + BaseArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq
        + From<u64>
        + Into<u64>
        + MaxEncodedLen;

    type PostId: Parameter
        + Member
        + BaseArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq
        + From<u64>
        + Into<u64>
        + MaxEncodedLen;

    /// Base deposit for any thread (note: thread creation also needs a `PostDeposit` since
    /// creating a thread means also creating a post)
    type ThreadDeposit: Get<Self::Balance>;

    /// Deposit needed to create a post
    type PostDeposit: Get<Self::Balance>;

    /// Maximum depth for nested categories
    type MaxCategoryDepth: Get<u64>;

    /// Maximum number of category's stickied threads
    type MaxStickiedThreads: Get<u32>;

    /// Maximum number of blocks before a post can be erased by anyone
    type PostLifeTime: Get<Self::BlockNumber>;

    /// Type defining the limits for different Storage items in the forum pallet
    type MapLimits: StorageLimits;

    /// The forum module Id, used to derive the account Id to hold the thread bounty
    type ModuleId: Get<PalletId>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    /// Working group pallet integration.
    type WorkingGroup: common::working_group::WorkingGroupAuthenticator<Self>;

    /// Validates member id and origin combination
    type MemberOriginValidator: MemberOriginValidator<
        Self::Origin,
        common::MemberId<Self>,
        Self::AccountId,
    >;

    fn calculate_hash(text: &[u8]) -> Self::Hash;
}

/// Upper bounds for storage maps and double maps.
pub trait StorageLimits {
    /// Maximum direct subcategories in a category
    type MaxDirectSubcategoriesInCategory: Get<u64>;

    /// Maximum moderator count for a single category
    type MaxModeratorsForCategory: Get<u64>;

    /// Maximum total of all existing categories
    type MaxTotalCategories: Get<u64>;
}

/// Represents a thread post
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
    Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct Post<ForumUserId, ThreadId, Hash, BlockNumber, RepayableBloatBond> {
    /// Id of thread to which this post corresponds.
    pub thread_id: ThreadId,

    /// Hash of current text
    pub text_hash: Hash,

    /// Author of post.
    pub author_id: ForumUserId,

    /// Cleanup pay off
    pub cleanup_pay_off: RepayableBloatBond,

    /// When it was created or last edited
    pub last_edited: BlockNumber,
}

/// Represents a thread
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug, Eq, TypeInfo, MaxEncodedLen)]
pub struct Thread<ForumUserId, CategoryId, RepayableBloatBond> {
    /// Category in which this thread lives
    pub category_id: CategoryId,

    /// Author of post.
    pub author_id: ForumUserId,

    /// Pay off by deleting
    pub cleanup_pay_off: RepayableBloatBond,

    /// Number of posts in the thread
    pub number_of_editable_posts: NumberOfPosts,
}

/// Represents a category
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct Category<CategoryId, Hash, StickiedThreadIds> {
    /// Title
    pub title_hash: Hash,

    /// Description
    pub description_hash: Hash,

    /// Whether category is archived.
    pub archived: bool,

    /// Number of subcategories, needed for emptiness checks when trying to delete category
    pub num_direct_subcategories: u32,

    // Number of threads in category, needed for emptiness checks when trying to delete category
    pub num_direct_threads: u32,

    pub num_direct_moderators: u32,

    /// Parent category, if child of another category, otherwise this category is a root category
    pub parent_category_id: Option<CategoryId>,

    /// Sticky threads list
    pub sticky_thread_ids: StickiedThreadIds,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum PrivilegedActor<T: Config> {
    Lead,
    Moderator(ModeratorId<T>),
}

impl<T: Config> core::fmt::Debug for PrivilegedActor<T> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PrivilegedActor::Lead => write!(formatter, "PrivilegedActor {{ Lead }}"),
            PrivilegedActor::Moderator(moderator_id) => {
                write!(formatter, "PrivilegedActor {{ {:?} }}", moderator_id)
            }
        }
    }
}

/// Represents a sequence of categories which have child-parent relatioonship
/// where last element is final ancestor, or root, in the context of the category tree.
type CategoryTreePath<T> = Vec<(<T as Config>::CategoryId, CategoryOf<T>)>;

type CategoryTreePathArg<T> = [(<T as Config>::CategoryId, CategoryOf<T>)];

decl_error! {
    /// Forum predefined errors
    pub enum Error for Module<T: Config> {
        /// Unexpected arithmetic error (overflow / underflow)
        ArithmeticError,

        /// Origin doesn't correspond to any lead account
        OriginNotForumLead,

        /// Forum user id not match its account.
        ForumUserIdNotMatchAccount,

        /// Moderator id not match its account.
        ModeratorIdNotMatchAccount,

        // Errors about thread.

        /// Thread not authored by the given user.
        AccountDoesNotMatchThreadAuthor,

        /// Thread does not exist
        ThreadDoesNotExist,

        /// Moderator can't moderate category containing thread.
        ModeratorModerateOriginCategory,

        /// Moderator can't moderate destination category.
        ModeratorModerateDestinationCategory,

        /// Origin is the same as the destination.
        ThreadMoveInvalid,

        /// Thread not being updated.
        ThreadNotBeingUpdated,

        /// Not enough balance to create thread
        InsufficientBalanceForThreadCreation,

        /// A thread with outstanding posts cannot be removed
        CannotDeleteThreadWithOutstandingPosts,

        // Errors about post.

        /// Post does not exist.
        PostDoesNotExist,

        /// Account does not match post author.
        AccountDoesNotMatchPostAuthor,

        /// Not enough balance to post
        InsufficientBalanceForPost,

        // Errors about category.

        /// Category not being updated.
        CategoryNotBeingUpdated,

        /// Ancestor category immutable, i.e. deleted or archived
        AncestorCategoryImmutable,

        /// Maximum valid category depth exceeded.
        MaxValidCategoryDepthExceeded,

        /// Category does not exist.
        CategoryDoesNotExist,

        /// Provided moderator is not given category moderator
        CategoryModeratorDoesNotExist,

        /// Category still contains some threads.
        CategoryNotEmptyThreads,

        /// Category still contains some subcategories.
        CategoryNotEmptyCategories,

        /// No permissions to delete category.
        ModeratorCantDeleteCategory,

        /// No permissions to update category.
        ModeratorCantUpdateCategory,

        // Error for limited size

        /// Maximum size of storage map exceeded
        MapSizeLimit,

        /// Category path len should be greater than zero
        PathLengthShouldBeGreaterThanZero,

        /// Maximum number of stickied threads per category exceeded
        MaxNumberOfStickiedThreadsExceeded,
    }
}

decl_storage! { generate_storage_info
    trait Store for Module<T: Config> as Forum_1_1 {
        /// Map category identifier to corresponding category.
        pub CategoryById get(fn category_by_id): map hasher(blake2_128_concat) T::CategoryId => CategoryOf<T>;

        /// Category identifier value to be used for the next Category created.
        pub NextCategoryId get(fn next_category_id) config(): T::CategoryId;

        /// Counter for all existing categories.
        pub CategoryCounter get(fn category_counter) config(): T::CategoryId;

        /// Map thread identifier to corresponding thread.
        pub ThreadById get(fn thread_by_id): double_map hasher(blake2_128_concat)
            T::CategoryId, hasher(blake2_128_concat) T::ThreadId => ThreadOf<T>;

        /// Thread identifier value to be used for next Thread in threadById.
        pub NextThreadId get(fn next_thread_id) config(): T::ThreadId;

        /// Post identifier value to be used for for next post created.
        pub NextPostId get(fn next_post_id) config(): T::PostId;

        /// Moderator set for each Category
        pub CategoryByModerator get(fn category_by_moderator): double_map
            hasher(blake2_128_concat) T::CategoryId, hasher(blake2_128_concat) ModeratorId<T> => ();

        /// Map post identifier to corresponding post.
        pub PostById get(fn post_by_id): double_map hasher(blake2_128_concat) T::ThreadId,
            hasher(blake2_128_concat) T::PostId => PostOf<T>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as Config>::CategoryId,
        ModeratorId = ModeratorId<T>,
        <T as Config>::ThreadId,
        <T as Config>::PostId,
        <T as frame_system::Config>::Hash,
        ForumUserId = ForumUserId<T>,
        PrivilegedActor = PrivilegedActor<T>,
        ExtendedPostId = ExtendedPostId<T>,
    {
        /// A category was introduced
        CategoryCreated(CategoryId, Option<CategoryId>, Vec<u8>, Vec<u8>),

        /// An arhical status of category with given id was updated.
        /// The second argument reflects the new archival status of the category.
        CategoryArchivalStatusUpdated(CategoryId, bool, PrivilegedActor),

        /// A title of category with given id was updated.
        /// The second argument reflects the new title hash of the category.
        CategoryTitleUpdated(CategoryId, Hash, PrivilegedActor),

        /// A discription of category with given id was updated.
        /// The second argument reflects the new description hash of the category.
        CategoryDescriptionUpdated(CategoryId, Hash, PrivilegedActor),

        /// A category was deleted
        CategoryDeleted(CategoryId, PrivilegedActor),

        /// A thread with given id was created.
        /// A third argument reflects the initial post id of the thread.
        ThreadCreated(CategoryId, ThreadId, PostId, ForumUserId, Vec<u8>, Vec<u8>),

        /// A thread with given id was moderated.
        ThreadModerated(ThreadId, Vec<u8>, PrivilegedActor, CategoryId),

        /// A thread with given id was updated.
        /// The second argument reflects the new archival status of the thread.
        ThreadUpdated(ThreadId, bool, PrivilegedActor, CategoryId),

        /// A thread metadata given id was updated.
        ThreadMetadataUpdated(ThreadId, ForumUserId, CategoryId, Vec<u8>),

        /// A thread was deleted.
        ThreadDeleted(ThreadId, ForumUserId, CategoryId, bool),

        /// A thread was moved to new category
        ThreadMoved(ThreadId, CategoryId, PrivilegedActor, CategoryId),

        /// Post with given id was created.
        PostAdded(PostId, ForumUserId, CategoryId, ThreadId, Vec<u8>, bool),

        /// Post with givne id was moderated.
        PostModerated(PostId, Vec<u8>, PrivilegedActor, CategoryId, ThreadId),

        /// Post with givne id was deleted.
        PostDeleted(Vec<u8>, ForumUserId, BTreeMap<ExtendedPostId, bool>),

        /// Post with given id had its text updated.
        /// The second argument reflects the number of total edits when the text update occurs.
        PostTextUpdated(PostId, ForumUserId, CategoryId, ThreadId, Vec<u8>),

        /// Sticky thread updated for category
        CategoryStickyThreadUpdate(CategoryId, BTreeSet<ThreadId>, PrivilegedActor),

        /// An moderator ability to moderate a category and its subcategories updated
        CategoryMembershipOfModeratorUpdated(ModeratorId, CategoryId, bool),
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        /// Predefined errors
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Exports const

        /// Deposit needed to create a post
        const PostDeposit: BalanceOf<T> = T::PostDeposit::get();

        /// Deposit needed to create a thread
        const ThreadDeposit: BalanceOf<T> = T::ThreadDeposit::get();

        /// MaxDirectSubcategoriesInCategory
        const MaxDirectSubcategoriesInCategory: u64 = <T::MapLimits as StorageLimits>::MaxDirectSubcategoriesInCategory::get();

        /// MaxTotalCategories
        const MaxTotalCategories: u64 = <T::MapLimits as StorageLimits>::MaxTotalCategories::get();

        /// Enable a moderator can moderate a category and its sub categories.
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (1)`
        /// - DB:
        ///    - O(1) doesn't depend on the state or parameters
        /// # </weight>
        #[weight = WeightInfoForum::<T>::update_category_membership_of_moderator_new()
            .max(WeightInfoForum::<T>::update_category_membership_of_moderator_old())]
        fn update_category_membership_of_moderator(origin, moderator_id: ModeratorId<T>, category_id: T::CategoryId, new_value: bool) -> DispatchResult {
            clear_prefix(b"Forum ForumUserById", None);

            let account_id = ensure_signed(origin)?;

            Self::ensure_can_update_category_membership_of_moderator(account_id, &category_id, &moderator_id, new_value)?;

            //
            // == MUTATION SAFE ==
            //

            if new_value {
                <CategoryByModerator<T>>::insert(category_id, moderator_id, ());

                <CategoryById<T>>::try_mutate(category_id, |category| {
                    category.num_direct_moderators = category.num_direct_moderators
                        .checked_add(1)
                        .ok_or(Error::<T>::ArithmeticError)?;
                    DispatchResult::Ok(())
                })?;
            } else {
                <CategoryByModerator<T>>::remove(category_id, moderator_id);

                <CategoryById<T>>::try_mutate(category_id, |category| {
                    category.num_direct_moderators = category.num_direct_moderators
                        .checked_sub(1)
                        .ok_or(Error::<T>::ArithmeticError)?;
                    DispatchResult::Ok(())
                })?;
            }

            // Generate event
            Self::deposit_event(RawEvent::CategoryMembershipOfModeratorUpdated(moderator_id, category_id, new_value));

            Ok(())
        }


        /// Add a new category.
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V + X)` where:
        /// - `W` is the category depth
        /// - `V` is the size of the category title in kilobytes.
        /// - `X` is the size of the category description in kilobytes.
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::create_category(
            T::MaxCategoryDepth::get() as u32,
            to_kb(title.len().saturated_into()),
            to_kb(description.len().saturated_into())
        )]
        fn create_category(origin, parent_category_id: Option<T::CategoryId>, title: Vec<u8>, description: Vec<u8>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            Self::ensure_can_create_category(account_id, &parent_category_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Get next category id
            let next_category_id = <NextCategoryId<T>>::get();

            // Create new category
            let new_category = Category {
                title_hash: T::calculate_hash(title.as_slice()),
                description_hash: T::calculate_hash(description.as_slice()),
                archived: false,
                num_direct_subcategories: 0,
                num_direct_threads: 0,
                num_direct_moderators: 0,
                parent_category_id,
                sticky_thread_ids: BoundedBTreeSet::default(),
            };

            // Insert category in map
            <CategoryById<T>>::mutate(next_category_id, |value| *value = new_category);

            // Update other next category id
            <NextCategoryId<T>>::mutate(|value| *value += One::one());

            // Update total category count
            <CategoryCounter<T>>::mutate(|value| *value += One::one());

            // If not root, increase parent's subcategories counter
            if let Some(tmp_parent_category_id) = parent_category_id {
                <CategoryById<T>>::try_mutate(tmp_parent_category_id, |c| {
                    c.num_direct_subcategories = c.num_direct_subcategories
                        .checked_add(1)
                        .ok_or(Error::<T>::ArithmeticError)?;
                    DispatchResult::Ok(())
                })?;
            }

            // Generate event
            Self::deposit_event(RawEvent::CategoryCreated(
                    next_category_id,
                    parent_category_id,
                    title,
                    description
                ));

            Ok(())
        }

        /// Update archival status
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W)` where:
        /// - `W` is the category depth
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::update_category_archival_status_lead(
            T::MaxCategoryDepth::get() as u32,
        ).max(WeightInfoForum::<T>::update_category_archival_status_moderator(
            T::MaxCategoryDepth::get() as u32,
        ))]
        fn update_category_archival_status(origin, actor: PrivilegedActor<T>, category_id: T::CategoryId, new_archival_status: bool) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Ensure actor can update category
            let category = Self::ensure_can_moderate_category(&account_id, &actor, &category_id)?;

            // No change, invalid transaction
            if new_archival_status == category.archived {
                return Err(Error::<T>::CategoryNotBeingUpdated.into())
            }

            //
            // == MUTATION SAFE ==
            //

            // Mutate category, and set possible new change parameters
            <CategoryById<T>>::mutate(category_id, |c| c.archived = new_archival_status);

            // Generate event
            Self::deposit_event(
                RawEvent::CategoryArchivalStatusUpdated(category_id, new_archival_status, actor)
            );

            Ok(())
        }

        /// Update category title
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V)` where:
        /// - `W` is the category depth
        /// - `V` is the size of the category title in kilobytes.
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::update_category_title_lead(
            T::MaxCategoryDepth::get() as u32,
            to_kb(title.len().saturated_into()),
        ).max(WeightInfoForum::<T>::update_category_title_moderator(
            T::MaxCategoryDepth::get() as u32,
            to_kb(title.len().saturated_into()),
        ))]
        fn update_category_title(origin, actor: PrivilegedActor<T>, category_id: T::CategoryId, title: Vec<u8>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Ensure actor can update category
            let category = Self::ensure_can_moderate_category(&account_id, &actor, &category_id)?;

            let title_hash = T::calculate_hash(title.as_slice());

            // No change, invalid transaction
            if title_hash == category.title_hash {
                return Err(Error::<T>::CategoryNotBeingUpdated.into())
            }

            //
            // == MUTATION SAFE ==
            //

            // Mutate category, and set possible new change parameters
            <CategoryById<T>>::mutate(category_id, |c| c.title_hash = title_hash);

            // Generate event
            Self::deposit_event(
                RawEvent::CategoryTitleUpdated(category_id, title_hash, actor)
            );

            Ok(())
        }

        /// Update category description
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W)` where:
        /// - `W` is the category depth
        /// - `V` is the size of the category description in kilobytes.
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::update_category_description_lead(
            T::MaxCategoryDepth::get() as u32,
            to_kb(description.len().saturated_into()),
        ).max(WeightInfoForum::<T>::update_category_description_moderator(
            T::MaxCategoryDepth::get() as u32,
            to_kb(description.len().saturated_into()),
        ))]
        fn update_category_description(origin, actor: PrivilegedActor<T>, category_id: T::CategoryId, description: Vec<u8>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Ensure actor can update category
            let category = Self::ensure_can_moderate_category(&account_id, &actor, &category_id)?;

            let description_hash = T::calculate_hash(description.as_slice());

            // No change, invalid transaction
            if description_hash == category.description_hash {
                return Err(Error::<T>::CategoryNotBeingUpdated.into())
            }

            //
            // == MUTATION SAFE ==
            //

            // Mutate category, and set possible new change parameters
            <CategoryById<T>>::mutate(category_id, |c| c.description_hash = description_hash);

            // Generate event
            Self::deposit_event(
                RawEvent::CategoryDescriptionUpdated(category_id, description_hash, actor)
            );

            Ok(())
        }

        /// Delete category
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W)` where:
        /// - `W` is the category depth
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::delete_category_lead(
            T::MaxCategoryDepth::get() as u32,
        ).max(WeightInfoForum::<T>::delete_category_moderator(
            T::MaxCategoryDepth::get() as u32,
        ))]
        fn delete_category(origin, actor: PrivilegedActor<T>, category_id: T::CategoryId) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            let category = Self::ensure_can_delete_category(account_id, &actor, &category_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Delete category
            <CategoryById<T>>::remove(category_id);
            if let Some(parent_category_id) = category.parent_category_id {
                <CategoryById<T>>::try_mutate(parent_category_id, |tmp_category| {
                    tmp_category.num_direct_subcategories = tmp_category.num_direct_subcategories
                        .checked_sub(1)
                        .ok_or(Error::<T>::ArithmeticError)?;
                    DispatchResult::Ok(())
                })?;
            }

            // Update total category count
            <CategoryCounter<T>>::mutate(|value| *value -= One::one());

            // Store the event
            Self::deposit_event(RawEvent::CategoryDeleted(category_id, actor));

            Ok(())
        }

        /// Create new thread in category
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V + X)` where:
        /// - `W` is the category depth
        /// - `V` is the size of the thread title in kilobytes.
        /// - `X` is the size of the thread text in kilobytes.
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::create_thread(
            T::MaxCategoryDepth::get() as u32,
            to_kb(metadata.len().saturated_into()),
            to_kb(text.len().saturated_into()),
        )]
        fn create_thread(
            origin,
            forum_user_id: ForumUserId<T>,
            category_id: T::CategoryId,
            metadata: Vec<u8>,
            text: Vec<u8>,
        ) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            Self::ensure_can_create_thread(&account_id, &forum_user_id, &category_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Create and add new thread
            let new_thread_id = <NextThreadId<T>>::get();

            // Pay the thread + post bloat bond into thread treasury account
            let (repaybale_thread_bloat_bond, repaybale_post_bloat_bond) =
                Self::pay_thread_and_post_deposit(new_thread_id, &account_id)?;

            // Build a new thread
            let new_thread = Thread {
                category_id,
                author_id: forum_user_id,
                cleanup_pay_off: repaybale_thread_bloat_bond,
                number_of_editable_posts: 0,
            };

            // Store thread
            <ThreadById<T>>::mutate(category_id, new_thread_id, |value| {
                *value = new_thread.clone()
            });

            // Add inital post to thread
            let initial_post_id = Self::add_new_post(
                new_thread_id,
                category_id,
                &text,
                forum_user_id,
                Some(repaybale_post_bloat_bond),
            );

            // Update next thread id
            <NextThreadId<T>>::mutate(|n| *n += One::one());

            // Update category's thread counter
            <CategoryById<T>>::try_mutate(category_id, |c| {
                c.num_direct_threads = c.num_direct_threads
                    .checked_add(1)
                    .ok_or(Error::<T>::ArithmeticError)?;
                DispatchResult::Ok(())
            })?;

            // Generate event
            Self::deposit_event(
                RawEvent::ThreadCreated(
                    category_id,
                    new_thread_id,
                    initial_post_id,
                    forum_user_id,
                    metadata,
                    text,
                )
            );

            Ok(())
        }

        /// Edit thread metadata
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V)` where:
        /// - `W` is the category depth
        /// - `V` is the size of the thread metadata in kilobytes.
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::edit_thread_metadata(
            T::MaxCategoryDepth::get() as u32,
            to_kb(new_metadata.len().saturated_into()),
        )]
        fn edit_thread_metadata(
            origin,
            forum_user_id: ForumUserId<T>,
            category_id: T::CategoryId,
            thread_id: T::ThreadId,
            new_metadata: Vec<u8>
        ) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            Self::ensure_can_edit_thread_metadata(account_id, &category_id, &thread_id, &forum_user_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Store the event
            Self::deposit_event(
                RawEvent::ThreadMetadataUpdated(
                    thread_id,
                    forum_user_id,
                    category_id,
                    new_metadata,
                )
            );

            Ok(())
        }

        /// Delete thread
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W)` where:
        /// - `W` is the category depth
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::delete_thread(T::MaxCategoryDepth::get() as u32)]
        fn delete_thread(
            origin,
            forum_user_id: ForumUserId<T>,
            category_id: T::CategoryId,
            thread_id: T::ThreadId,
            hide: bool,
        ) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            let thread = Self::ensure_can_delete_thread(
                &account_id,
                &forum_user_id,
                &category_id,
                &thread_id
            )?;

            //
            // == MUTATION SAFE ==
            //

            // Pay off the bloat bond
            let thread_account_id = Self::thread_account(thread_id);
            thread.cleanup_pay_off.repay::<T>(&thread_account_id, &account_id, true)?;

            // Delete thread
            Self::delete_thread_inner(thread.category_id, thread_id)?;

            // Store the event
            Self::deposit_event(RawEvent::ThreadDeleted(
                    thread_id,
                    forum_user_id,
                    category_id,
                    hide,
                ));

            Ok(())
        }

        /// Move thread to another category
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W)` where:
        /// - `W` is the category depth
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::move_thread_to_category_lead(
            T::MaxCategoryDepth::get() as u32,
        ).max(WeightInfoForum::<T>::move_thread_to_category_moderator(
            T::MaxCategoryDepth::get() as u32,
        ))]
        fn move_thread_to_category(
            origin,
            actor: PrivilegedActor<T>,
            category_id: T::CategoryId,
            thread_id: T::ThreadId,
            new_category_id: T::CategoryId
        ) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Make sure moderator move between selected categories
            let thread = Self::ensure_can_move_thread(account_id, &actor, &category_id, &thread_id, &new_category_id)?;

            //
            // == MUTATION SAFE ==
            //

            let updated_thread = Thread {
                category_id: new_category_id,
                ..thread
            };

            <ThreadById<T>>::remove(thread.category_id, thread_id);
            <ThreadById<T>>::insert(new_category_id, thread_id, updated_thread);
            <CategoryById<T>>::try_mutate(thread.category_id, |category| {
                category.num_direct_threads = category.num_direct_threads
                    .checked_sub(1)
                    .ok_or(Error::<T>::ArithmeticError)?;
                DispatchResult::Ok(())
            })?;
            <CategoryById<T>>::try_mutate(new_category_id, |category| {
                category.num_direct_threads = category.num_direct_threads
                    .checked_add(1)
                    .ok_or(Error::<T>::ArithmeticError)?;
                DispatchResult::Ok(())
            })?;

            // Store the event
            Self::deposit_event(
                RawEvent::ThreadMoved(thread_id, new_category_id, actor, category_id)
            );

            Ok(())
        }

        /// Moderate thread
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V + X)` where:
        /// - `W` is the category depth,
        /// - `V` is the number of thread posts,
        /// - `X` is the size of the rationale in kilobytes
        /// - DB:
        ///    - O(W + V)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::moderate_thread_lead(
            T::MaxCategoryDepth::get() as u32,
            to_kb(rationale.len().saturated_into()),
        ).max(
            WeightInfoForum::<T>::moderate_thread_moderator(
                T::MaxCategoryDepth::get() as u32,
                to_kb(rationale.len().saturated_into()),
            )
        )]
        fn moderate_thread(origin, actor: PrivilegedActor<T>, category_id: T::CategoryId, thread_id: T::ThreadId, rationale: Vec<u8>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Ensure actor is allowed to moderate thread
            let thread = Self::ensure_can_moderate_thread(&account_id, &actor, &category_id, &thread_id)?;

            //
            // == MUTATION SAFE ==
            //

            Self::slash_thread_account(thread_id, thread.cleanup_pay_off.amount)?;

            // Delete thread
            Self::delete_thread_inner(thread.category_id, thread_id)?;

            // Generate event
            Self::deposit_event(
                RawEvent::ThreadModerated(thread_id, rationale, actor, category_id)
            );

            Ok(())
        }

        /// Add post
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V)` where:
        /// - `W` is the category depth,
        /// - `V` is the size of the text in kilobytes
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::add_post(
            T::MaxCategoryDepth::get() as u32,
            to_kb(text.len().saturated_into()),
        )]
        fn add_post(
            origin,
            forum_user_id: ForumUserId<T>,
            category_id: T::CategoryId,
            thread_id: T::ThreadId,
            text: Vec<u8>,
            editable: bool,
        ) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Make sure thread exists and is mutable
            let _ = Self::ensure_can_add_post(&account_id, &forum_user_id, &category_id, &thread_id)?;

            if editable {
                let post_deposit = T::PostDeposit::get();
                ensure!(
                    has_sufficient_balance_for_fees::<T>(&account_id, post_deposit),
                    Error::<T>::InsufficientBalanceForPost
                );
            }

            //
            // == MUTATION SAFE ==
            //

            let repayable_bloat_bond = match editable {
                true => Some(Self::pay_post_deposit(thread_id, &account_id)?),
                false => None
            };

            // Add new post
            let post_id = Self::add_new_post(
                    thread_id,
                    category_id,
                    text.as_slice(),
                    forum_user_id,
                    repayable_bloat_bond,
                );

            // Generate event
            Self::deposit_event(
                RawEvent::PostAdded(post_id, forum_user_id, category_id, thread_id, text, editable)
            );

            Ok(())
        }

        /// Edit post text
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V)` where:
        /// - `W` is the category depth,
        /// - `V` is the size of the new text in kilobytes
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::edit_post_text(
            T::MaxCategoryDepth::get() as u32,
            to_kb(new_text.len().saturated_into()),
        )]
        fn edit_post_text(
            origin,
            forum_user_id: ForumUserId<T>,
            category_id: T::CategoryId,
            thread_id: T::ThreadId,
            post_id: T::PostId,
            new_text: Vec<u8>
        ) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Check that account is forum member
            Self::ensure_is_forum_user(&account_id, &forum_user_id)?;

            // Make sure there exists a mutable post with post id `post_id`
            let mut post = Self::ensure_post_is_mutable(&category_id, &thread_id, &post_id)?;

            // Signer does not match creator of post with identifier postId
            ensure!(post.author_id == forum_user_id, Error::<T>::AccountDoesNotMatchPostAuthor);

            //
            // == MUTATION SAFE ==
            //

            // Update post text
            let text_hash = T::calculate_hash(&new_text);
            post.text_hash = text_hash;
            post.last_edited = frame_system::Pallet::<T>::block_number();

            <PostById<T>>::insert(thread_id, post_id, post);

            // Generate event
            Self::deposit_event(RawEvent::PostTextUpdated(
                    post_id,
                    forum_user_id,
                    category_id,
                    thread_id,
                    new_text
                ));

            Ok(())
        }

        /// Moderate post
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V)` where:
        /// - `W` is the category depth,
        /// - `V` is the size of the rationale in kilobytes
        /// - DB:
        ///    - O(W)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::moderate_post_lead(
            T::MaxCategoryDepth::get() as u32,
            to_kb(rationale.len().saturated_into()),
        ).max(WeightInfoForum::<T>::moderate_post_moderator(
            T::MaxCategoryDepth::get() as u32,
            to_kb(rationale.len().saturated_into()),
        ))]
        fn moderate_post(origin, actor: PrivilegedActor<T>, category_id: T::CategoryId, thread_id: T::ThreadId, post_id: T::PostId, rationale: Vec<u8>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            // Ensure actor is allowed to moderate post and post is editable
            let post = Self::ensure_can_moderate_post(
                account_id,
                &actor,
                &category_id,
                &thread_id,
                &post_id
            )?;

            //
            // == MUTATION SAFE ==
            //

            Self::slash_thread_account(thread_id, post.cleanup_pay_off.amount)?;

            Self::delete_post_inner(category_id, thread_id, post_id);

            // Generate event
            Self::deposit_event(
                RawEvent::PostModerated(post_id, rationale, actor, category_id, thread_id)
            );

            Ok(())
        }

        /// Delete post from storage.
        /// You need to provide a vector of posts to delete in the form
        /// (T::CategoryId, T::ThreadId, T::PostId, bool)
        /// where the last bool is whether you want to hide it apart from deleting it
        ///
        /// ## Weight
        /// `O (W + V + P)` where:
        /// - `W` is the category depth,
        /// - `V` is the size of the rationale in kilobytes
        /// - `P` is the number of posts to delete
        /// - DB:
        ///    - O(W + P)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::delete_posts(
            T::MaxCategoryDepth::get() as u32,
            to_kb(rationale.len().saturated_into()),
            posts.len().saturated_into(),
        )]
        fn delete_posts(
            origin,
            forum_user_id: ForumUserId<T>,
            posts: BTreeMap<ExtendedPostId<T>, bool>,
            rationale: Vec<u8>,
        ) -> DispatchResult {

            let account_id = ensure_signed(origin)?;

            let mut deleting_posts = Vec::new();
            for (ExtendedPostIdObject {category_id, thread_id, post_id}, hide) in &posts {
                // Ensure actor is allowed to moderate post and post is editable
                let post = Self::ensure_can_delete_post(
                    &account_id,
                    &forum_user_id,
                    category_id,
                    thread_id,
                    post_id,
                    *hide,
                )?;

                deleting_posts.push((category_id, thread_id, post_id, post));
            }

            //
            // == MUTATION SAFE ==
            //

            for (category_id, thread_id, post_id, post) in deleting_posts {
                // Pay off the post bloat bond
                let thread_account_id = Self::thread_account(*thread_id);
                post.cleanup_pay_off.repay::<T>(&thread_account_id, &account_id, false)?;

                Self::delete_post_inner(*category_id, *thread_id, *post_id);
            }

            // Generate event
            Self::deposit_event(
                RawEvent::PostDeleted(rationale, forum_user_id, posts)
            );

            Ok(())
        }

        /// Set stickied threads for category
        ///
        /// <weight>
        ///
        /// ## Weight
        /// `O (W + V)` where:
        /// - `W` is the category depth,
        /// - `V` is the length of the stickied_ids
        /// - DB:
        ///    - O(W + V)
        /// # </weight>
        #[weight = WeightInfoForum::<T>::set_stickied_threads_lead(
            T::MaxCategoryDepth::get() as u32,
            stickied_ids.len().saturated_into(),
        ).max(
            WeightInfoForum::<T>::set_stickied_threads_moderator(
                T::MaxCategoryDepth::get() as u32,
                stickied_ids.len().saturated_into(),
            )
        )]
        fn set_stickied_threads(origin, actor: PrivilegedActor<T>, category_id: T::CategoryId, stickied_ids: BTreeSet<T::ThreadId>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            let stickied_ids_bounded: StickiedThreadIds<T> = stickied_ids
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::MaxNumberOfStickiedThreadsExceeded)?;
            Self::ensure_can_set_stickied_threads(account_id, &actor, &category_id, &stickied_ids)?;

            //
            // == MUTATION SAFE ==
            //

            // Update category
            <CategoryById<T>>::mutate(category_id, |category| category.sticky_thread_ids = stickied_ids_bounded);

            // Generate event
            Self::deposit_event(
                RawEvent::CategoryStickyThreadUpdate(category_id, stickied_ids, actor)
            );

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    fn thread_account(thread_id: T::ThreadId) -> T::AccountId {
        T::ModuleId::get().into_sub_account_truncating(thread_id)
    }

    fn slash_thread_account(thread_id: T::ThreadId, amount: BalanceOf<T>) -> DispatchResult {
        let thread_account_id = Self::thread_account(thread_id);
        burn_from_usable::<T>(&thread_account_id, amount).map(|_| ())
    }

    fn pay_post_deposit(
        thread_id: T::ThreadId,
        account_id: &T::AccountId,
    ) -> Result<RepayableBloatBondOf<T>, DispatchError> {
        let post_deposit = T::PostDeposit::get();
        let thread_account_id = Self::thread_account(thread_id);
        let locked_balance_used = pay_fee::<T>(account_id, Some(&thread_account_id), post_deposit)?;

        // construct RepayableBloatBond based on pay_fee result
        Ok(match locked_balance_used.is_zero() {
            true => RepayableBloatBond::new(post_deposit, None),
            false => RepayableBloatBond::new(post_deposit, Some(account_id.clone())),
        })
    }

    fn pay_thread_and_post_deposit(
        thread_id: T::ThreadId,
        account_id: &T::AccountId,
    ) -> Result<(RepayableBloatBondOf<T>, RepayableBloatBondOf<T>), DispatchError> {
        let thread_deposit = T::ThreadDeposit::get();
        let post_deposit = T::PostDeposit::get();
        let total_deposit = thread_deposit.saturating_add(post_deposit);
        let thread_account_id = Self::thread_account(thread_id);
        let locked_balance_used =
            pay_fee::<T>(account_id, Some(&thread_account_id), total_deposit)?;

        // construct RepayableBloatBond based on pay_fee result
        Ok(if locked_balance_used.is_zero() {
            (
                RepayableBloatBond::new(thread_deposit, None),
                RepayableBloatBond::new(post_deposit, None),
            )
        } else if locked_balance_used <= thread_deposit {
            (
                RepayableBloatBond::new(thread_deposit, Some(account_id.clone())),
                RepayableBloatBond::new(post_deposit, None),
            )
        } else {
            (
                RepayableBloatBond::new(thread_deposit, Some(account_id.clone())),
                RepayableBloatBond::new(post_deposit, Some(account_id.clone())),
            )
        })
    }

    /// Add new posts & increase thread counter
    pub fn add_new_post(
        thread_id: T::ThreadId,
        category_id: T::CategoryId,
        text: &[u8],
        author_id: ForumUserId<T>,
        repayable_bloat_bond: Option<RepayableBloatBondOf<T>>,
    ) -> T::PostId {
        // Make and add initial post
        let new_post_id = <NextPostId<T>>::get();

        // Update next post id
        <NextPostId<T>>::mutate(|n| *n += One::one());

        if let Some(cleanup_pay_off) = repayable_bloat_bond {
            // Post is editable.
            // Build a post
            let new_post = Post {
                text_hash: T::calculate_hash(text),
                thread_id,
                author_id,
                cleanup_pay_off,
                last_edited: frame_system::Pallet::<T>::block_number(),
            };

            <PostById<T>>::insert(thread_id, new_post_id, new_post);

            <ThreadById<T>>::mutate(category_id, thread_id, |thread| {
                // non editable post should leave the counter untouched
                thread.number_of_editable_posts = thread.number_of_editable_posts.saturating_add(1);
            });
        }

        new_post_id
    }

    fn delete_thread_inner(category_id: T::CategoryId, thread_id: T::ThreadId) -> DispatchResult {
        // Delete thread
        <ThreadById<T>>::remove(category_id, thread_id);

        // decrease category's thread counter
        <CategoryById<T>>::try_mutate(category_id, |category| {
            category.num_direct_threads = category
                .num_direct_threads
                .checked_sub(1)
                .ok_or(Error::<T>::ArithmeticError)?;
            DispatchResult::Ok(())
        })?;

        Ok(())
    }

    fn delete_post_inner(category_id: T::CategoryId, thread_id: T::ThreadId, post_id: T::PostId) {
        if <ThreadById<T>>::contains_key(category_id, thread_id) {
            let mut thread = <ThreadById<T>>::get(category_id, thread_id);
            thread.number_of_editable_posts = thread.number_of_editable_posts.saturating_sub(1);

            <ThreadById<T>>::mutate(category_id, thread_id, |value| *value = thread);
        }

        <PostById<T>>::remove(thread_id, post_id);
    }

    fn ensure_post_is_mutable(
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
        post_id: &T::PostId,
    ) -> Result<PostOf<T>, Error<T>> {
        // If the post is stored then it's mutable
        let post = Self::ensure_post_exists(category_id, thread_id, post_id)?;

        // and make sure thread is mutable
        Self::ensure_thread_is_mutable(category_id, thread_id)?;

        Ok(post)
    }

    // TODO: change this name, since it's no longer descriptive
    fn ensure_post_exists(
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
        post_id: &T::PostId,
    ) -> Result<PostOf<T>, Error<T>> {
        if !<ThreadById<T>>::contains_key(category_id, thread_id) {
            return Err(Error::<T>::PostDoesNotExist);
        }

        if !<PostById<T>>::contains_key(thread_id, post_id) {
            return Err(Error::<T>::PostDoesNotExist);
        }

        Ok(<PostById<T>>::get(thread_id, post_id))
    }

    fn ensure_can_moderate_post(
        account_id: T::AccountId,
        actor: &PrivilegedActor<T>,
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
        post_id: &T::PostId,
    ) -> Result<PostOf<T>, Error<T>> {
        // Ensure the moderator can moderate the category
        Self::ensure_can_moderate_category(&account_id, actor, category_id)?;

        // Make sure post exists and is mutable
        let post = Self::ensure_post_is_mutable(category_id, thread_id, post_id)?;

        Ok(post)
    }

    fn ensure_can_delete_post(
        account_id: &T::AccountId,
        forum_user_id: &ForumUserId<T>,
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
        post_id: &T::PostId,
        hide: bool,
    ) -> Result<PostOf<T>, Error<T>> {
        let post = Self::ensure_post_is_mutable(category_id, thread_id, post_id)?;

        // Check that account is forum member
        Self::ensure_is_forum_user(account_id, forum_user_id)?;

        // Signer does not match creator of post with identifier postId
        ensure!(
            post.author_id == *forum_user_id
                || Self::anyone_can_delete_post(&post, thread_id, category_id) && !hide,
            Error::<T>::AccountDoesNotMatchPostAuthor
        );

        Ok(post)
    }

    fn anyone_can_delete_post(
        post: &PostOf<T>,
        thread_id: &T::ThreadId,
        category_id: &T::CategoryId,
    ) -> bool {
        frame_system::Pallet::<T>::block_number() >= T::PostLifeTime::get() + post.last_edited
            && !Self::thread_exists(category_id, thread_id)
    }

    fn thread_exists(category_id: &T::CategoryId, thread_id: &T::ThreadId) -> bool {
        <ThreadById<T>>::contains_key(category_id, thread_id)
    }

    fn ensure_thread_is_mutable(
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
    ) -> Result<(CategoryOf<T>, ThreadOf<T>), Error<T>> {
        // Make sure thread exists
        let thread = Self::ensure_thread_exists(category_id, thread_id)?;

        // and corresponding category is mutable
        let category = Self::ensure_category_is_mutable(category_id)?;

        Ok((category, thread))
    }

    fn ensure_thread_exists(
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
    ) -> Result<ThreadOf<T>, Error<T>> {
        if !<ThreadById<T>>::contains_key(category_id, thread_id) {
            return Err(Error::<T>::ThreadDoesNotExist);
        }

        Ok(<ThreadById<T>>::get(category_id, thread_id))
    }

    fn ensure_can_edit_thread_metadata(
        account_id: T::AccountId,
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
        forum_user_id: &ForumUserId<T>,
    ) -> Result<ThreadOf<T>, Error<T>> {
        // Check that account is forum member
        Self::ensure_is_forum_user(&account_id, forum_user_id)?;

        // Ensure thread is mutable
        let (_, thread) = Self::ensure_thread_is_mutable(category_id, thread_id)?;

        // Ensure forum user is author of the thread
        Self::ensure_is_thread_author(&thread, forum_user_id)?;

        Ok(thread)
    }

    fn ensure_is_thread_author(
        thread: &ThreadOf<T>,
        forum_user_id: &ForumUserId<T>,
    ) -> Result<(), Error<T>> {
        ensure!(
            thread.author_id == *forum_user_id,
            Error::<T>::AccountDoesNotMatchThreadAuthor
        );

        Ok(())
    }

    fn ensure_actor_role(
        account_id: &T::AccountId,
        actor: &PrivilegedActor<T>,
    ) -> Result<(), Error<T>> {
        match actor {
            PrivilegedActor::Lead => {
                Self::ensure_is_forum_lead_account(account_id)?;
            }
            PrivilegedActor::Moderator(moderator_id) => {
                Self::ensure_is_moderator_account(account_id, moderator_id)?;
            }
        };
        Ok(())
    }

    // Ensure forum user is lead - check via account
    fn ensure_is_forum_lead_account(account_id: &T::AccountId) -> Result<(), Error<T>> {
        let is_lead = T::WorkingGroup::is_leader_account_id(account_id);

        ensure!(is_lead, Error::<T>::OriginNotForumLead);
        Ok(())
    }

    /// Ensure forum user id registered and its account id matched
    fn ensure_is_forum_user(
        account_id: &T::AccountId,
        forum_user_id: &ForumUserId<T>,
    ) -> Result<(), Error<T>> {
        let is_member =
            T::MemberOriginValidator::is_member_controller_account(forum_user_id, account_id);

        ensure!(is_member, Error::<T>::ForumUserIdNotMatchAccount);
        Ok(())
    }

    /// Ensure moderator id registered and its account id matched - check via account
    fn ensure_is_moderator_account(
        account_id: &T::AccountId,
        moderator_id: &ModeratorId<T>,
    ) -> Result<(), Error<T>> {
        let is_moderator = T::WorkingGroup::is_worker_account_id(account_id, moderator_id);

        ensure!(is_moderator, Error::<T>::ModeratorIdNotMatchAccount);

        Ok(())
    }

    // Ensure actor can moderate thread.
    fn ensure_can_moderate_thread(
        account_id: &T::AccountId,
        actor: &PrivilegedActor<T>,
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
    ) -> Result<ThreadOf<T>, DispatchError> {
        // Check that account is forum member
        Self::ensure_can_moderate_category(account_id, actor, category_id)?;

        let thread = Self::ensure_thread_exists(category_id, thread_id)?;

        // Ensure that the thread has no outstanding posts
        Self::ensure_empty_thread(&thread)?;

        Ok(thread)
    }

    // Ensure actor can manipulate thread.
    fn ensure_can_delete_thread(
        account_id: &T::AccountId,
        forum_user_id: &ForumUserId<T>,
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
    ) -> Result<ThreadOf<T>, DispatchError> {
        // Ensure thread exists and is mutable
        let (_, thread) = Self::ensure_thread_is_mutable(category_id, thread_id)?;

        // Check that account is forum member
        Self::ensure_is_forum_user(account_id, forum_user_id)?;

        // Ensure forum user is author of the thread
        Self::ensure_is_thread_author(&thread, forum_user_id)?;

        // Ensure the thread has no outstanding posts
        Self::ensure_empty_thread(&thread)?;

        Ok(thread)
    }

    fn ensure_can_move_thread(
        account_id: T::AccountId,
        actor: &PrivilegedActor<T>,
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
        new_category_id: &T::CategoryId,
    ) -> Result<ThreadOf<T>, DispatchError> {
        ensure!(
            category_id != new_category_id,
            Error::<T>::ThreadMoveInvalid,
        );

        ensure!(
            Self::ensure_can_moderate_category(&account_id, actor, category_id).is_ok(),
            Error::<T>::ModeratorModerateOriginCategory
        );

        let thread = Self::ensure_thread_exists(category_id, thread_id)?;

        Self::ensure_can_moderate_category_path(actor, new_category_id)
            .map_err(|_| Error::<T>::ModeratorModerateDestinationCategory)?;

        Ok(thread)
    }

    fn ensure_category_is_mutable(category_id: &T::CategoryId) -> Result<CategoryOf<T>, Error<T>> {
        let category = Self::ensure_category_exists(category_id)?;
        let category_tree_path = Self::build_category_tree_path(category_id);

        Self::ensure_can_mutate_in_path_leaf(&category_tree_path)?;

        Ok(category)
    }

    fn ensure_can_mutate_in_path_leaf(
        category_tree_path: &CategoryTreePathArg<T>,
    ) -> Result<(), Error<T>> {
        // Is parent category directly or indirectly deleted or archived category
        ensure!(
            !category_tree_path
                .iter()
                .any(|(_, c): &(_, CategoryOf<T>)| c.archived),
            Error::<T>::AncestorCategoryImmutable
        );

        Ok(())
    }

    fn ensure_can_add_subcategory_path_leaf(
        parent_category_id: &T::CategoryId,
    ) -> Result<(), Error<T>> {
        // Get the path from parent category to root
        let (_, category_tree_path) =
            Self::ensure_valid_category_and_build_category_tree_path(parent_category_id)?;

        let max_category_depth: u64 = T::MaxCategoryDepth::get();

        // Check if max depth reached
        if category_tree_path.len() as u64 >= max_category_depth {
            return Err(Error::<T>::MaxValidCategoryDepthExceeded);
        }

        Self::ensure_can_mutate_in_path_leaf(&category_tree_path)?;

        Ok(())
    }

    /// Build category tree path and validate them
    fn ensure_valid_category_and_build_category_tree_path(
        category_id: &T::CategoryId,
    ) -> Result<(CategoryOf<T>, CategoryTreePath<T>), Error<T>> {
        let category = Self::ensure_category_exists(category_id)?;

        // Get path from parent to root of category tree.
        let category_tree_path = Self::build_category_tree_path(category_id);

        if category_tree_path.is_empty() {
            debug_assert!(
                false,
                "Should not fail! {:?}",
                Error::<T>::PathLengthShouldBeGreaterThanZero
            );
            Err(Error::<T>::PathLengthShouldBeGreaterThanZero)
        } else {
            Ok((category, category_tree_path))
        }
    }

    /// Builds path and populates in `path`.
    /// Requires that `category_id` is valid
    fn build_category_tree_path(category_id: &T::CategoryId) -> CategoryTreePath<T> {
        // Get path from parent to root of category tree.
        let mut category_tree_path = vec![];

        Self::_build_category_tree_path(category_id, &mut category_tree_path);

        category_tree_path
    }

    /// Builds path and populates in `path`.
    /// Requires that `category_id` is valid
    fn _build_category_tree_path(category_id: &T::CategoryId, path: &mut CategoryTreePath<T>) {
        // Grab category
        let category = <CategoryById<T>>::get(*category_id);

        // Add category to path container
        path.push((*category_id, category.clone()));

        // Make recursive call on parent if we are not at root
        if let Some(parent_category_id) = category.parent_category_id {
            debug_assert!(<CategoryById<T>>::contains_key(parent_category_id));

            Self::_build_category_tree_path(&parent_category_id, path);
        }
    }

    fn ensure_can_delete_category(
        account_id: T::AccountId,
        actor: &PrivilegedActor<T>,
        category_id: &T::CategoryId,
    ) -> Result<CategoryOf<T>, Error<T>> {
        // Check actor's role
        Self::ensure_actor_role(&account_id, actor)?;

        // Ensure category exists
        let category = Self::ensure_category_exists(category_id)?;

        // Ensure category is empty
        ensure!(
            category.num_direct_threads == 0,
            Error::<T>::CategoryNotEmptyThreads,
        );
        ensure!(
            category.num_direct_subcategories == 0,
            Error::<T>::CategoryNotEmptyCategories,
        );

        // check moderator's privilege
        if let Some(parent_category_id) = category.parent_category_id {
            Self::ensure_can_moderate_category_path(actor, &parent_category_id)
                .map_err(|_| Error::<T>::ModeratorCantDeleteCategory)?;

            Ok(category)
        } else {
            // category is root - only lead can delete it
            match actor {
                PrivilegedActor::Lead => Ok(category),
                PrivilegedActor::Moderator(_) => Err(Error::<T>::ModeratorCantDeleteCategory),
            }
        }
    }

    /// check if an account can moderate a category.
    fn ensure_can_moderate_category(
        account_id: &T::AccountId,
        actor: &PrivilegedActor<T>,
        category_id: &T::CategoryId,
    ) -> Result<CategoryOf<T>, Error<T>> {
        // Ensure actor's role
        Self::ensure_actor_role(account_id, actor)?;

        Self::ensure_can_moderate_category_path(actor, category_id)
    }

    // check that moderator is allowed to manipulate category in hierarchy
    fn ensure_can_moderate_category_path(
        actor: &PrivilegedActor<T>,
        category_id: &T::CategoryId,
    ) -> Result<CategoryOf<T>, Error<T>> {
        fn check_moderator<T: Config>(
            category_tree_path: &CategoryTreePathArg<T>,
            moderator_id: &ModeratorId<T>,
        ) -> Result<(), Error<T>> {
            for item in category_tree_path {
                if <CategoryByModerator<T>>::contains_key(item.0, moderator_id) {
                    return Ok(());
                }
            }

            Err(Error::<T>::ModeratorCantUpdateCategory)
        }

        let (category, category_tree_path) =
            Self::ensure_valid_category_and_build_category_tree_path(category_id)?;

        match actor {
            PrivilegedActor::Lead => (),
            PrivilegedActor::Moderator(moderator_id) => {
                check_moderator::<T>(&category_tree_path, moderator_id)?
            }
        };

        Ok(category)
    }

    fn ensure_can_update_category_membership_of_moderator(
        account_id: T::AccountId,
        category_id: &T::CategoryId,
        moderator_id: &ModeratorId<T>,
        new_value: bool,
    ) -> Result<(), DispatchError> {
        // Not signed by forum LEAD
        Self::ensure_is_forum_lead_account(&account_id)?;

        // ensure category exists.
        let category = Self::ensure_category_exists(category_id)?;

        if new_value {
            // ensure worker by moderator_id exists
            T::WorkingGroup::ensure_worker_exists(moderator_id)?;
            Self::ensure_map_limits::<<<T>::MapLimits as StorageLimits>::MaxModeratorsForCategory>(
                category.num_direct_moderators as u64,
            )?;
        } else {
            ensure!(
                <CategoryByModerator<T>>::contains_key(category_id, moderator_id),
                Error::<T>::CategoryModeratorDoesNotExist
            );
        }

        Ok(())
    }

    fn ensure_category_exists(category_id: &T::CategoryId) -> Result<CategoryOf<T>, Error<T>> {
        ensure!(
            <CategoryById<T>>::contains_key(&category_id),
            Error::<T>::CategoryDoesNotExist
        );

        Ok(<CategoryById<T>>::get(category_id))
    }

    fn ensure_can_create_category(
        account_id: T::AccountId,
        parent_category_id: &Option<T::CategoryId>,
    ) -> Result<Option<CategoryOf<T>>, Error<T>> {
        // Not signed by forum LEAD
        Self::ensure_is_forum_lead_account(&account_id)?;

        Self::ensure_map_limits::<<<T>::MapLimits as StorageLimits>::MaxTotalCategories>(
            <CategoryCounter<T>>::get().into() as u64,
        )?;

        // If not root, then check that we can create in parent category
        if let Some(tmp_parent_category_id) = parent_category_id {
            // Can we mutate in this category?
            Self::ensure_can_add_subcategory_path_leaf(tmp_parent_category_id)?;

            let parent_category = <CategoryById<T>>::get(tmp_parent_category_id);

            Self::ensure_map_limits::<
                <<T>::MapLimits as StorageLimits>::MaxDirectSubcategoriesInCategory,
            >(parent_category.num_direct_subcategories as u64)?;

            return Ok(Some(parent_category));
        }

        Ok(None)
    }

    fn ensure_can_create_thread(
        account_id: &T::AccountId,
        forum_user_id: &ForumUserId<T>,
        category_id: &T::CategoryId,
    ) -> DispatchResult {
        // Check that account is forum member
        Self::ensure_is_forum_user(account_id, forum_user_id)?;

        Self::ensure_category_is_mutable(category_id)?;

        // Check if the costs associated with thread and post creation are coverable
        let fees = T::ThreadDeposit::get().saturating_add(T::PostDeposit::get());
        ensure!(
            has_sufficient_balance_for_fees::<T>(account_id, fees),
            Error::<T>::InsufficientBalanceForThreadCreation
        );

        Ok(())
    }

    fn ensure_can_add_post(
        account_id: &T::AccountId,
        forum_user_id: &ForumUserId<T>,
        category_id: &T::CategoryId,
        thread_id: &T::ThreadId,
    ) -> Result<(CategoryOf<T>, ThreadOf<T>), Error<T>> {
        // Check that account is forum member
        Self::ensure_is_forum_user(account_id, forum_user_id)?;

        let (category, thread) = Self::ensure_thread_is_mutable(category_id, thread_id)?;

        Ok((category, thread))
    }

    fn ensure_can_set_stickied_threads(
        account_id: T::AccountId,
        actor: &PrivilegedActor<T>,
        category_id: &T::CategoryId,
        stickied_ids: &BTreeSet<T::ThreadId>,
    ) -> Result<CategoryOf<T>, Error<T>> {
        // Ensure actor can moderate the category
        let category = Self::ensure_can_moderate_category(&account_id, actor, category_id)?;

        // Ensure all thread id valid and is under the category
        for thread_id in stickied_ids {
            Self::ensure_thread_exists(category_id, thread_id)?;
        }

        Ok(category)
    }

    // supposed to be called before mutations - checks if next entity can be added
    fn ensure_map_limits<U: Get<u64>>(current_amount: u64) -> Result<(), Error<T>> {
        fn check_limit<T: Config>(amount: u64, limit: u64) -> Result<(), Error<T>> {
            if amount >= limit {
                return Err(Error::<T>::MapSizeLimit);
            }

            Ok(())
        }

        check_limit(current_amount, U::get())
    }

    fn ensure_empty_thread(thread: &ThreadOf<T>) -> DispatchResult {
        ensure!(
            thread.number_of_editable_posts.is_zero(),
            Error::<T>::CannotDeleteThreadWithOutstandingPosts
        );
        Ok(())
    }
}
