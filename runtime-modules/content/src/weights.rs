// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for content
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./scripts/../target/release/joystream-node
// benchmark
// pallet
// --pallet=content
// --extrinsic=*
// --chain=dev
// --steps=50
// --repeat=20
// --execution=native
// --template=./scripts/../devops/joystream-pallet-weight-template.hbs
// --output=.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for content.
pub trait WeightInfo {
	fn create_channel(_a: u32, _b: u32, _c: u32, _d: u32, _e: u32, ) -> Weight;
	fn channel_update_with_assets(_a: u32, _b: u32, _c: u32, _d: u32, _e: u32, ) -> Weight;
	fn channel_update_without_assets(_a: u32, _b: u32, ) -> Weight;
	fn delete_channel(_a: u32, _b: u32, _c: u32, ) -> Weight;
	fn create_curator_group(_a: u32, ) -> Weight;
	fn update_curator_group_permissions(_a: u32, ) -> Weight;
	fn set_curator_group_status() -> Weight;
	fn add_curator_to_group() -> Weight;
	fn remove_curator_from_group() -> Weight;
	fn initialize_channel_transfer(_a: u32, ) -> Weight;
	fn cancel_channel_transfer() -> Weight;
	fn accept_channel_transfer(_a: u32, ) -> Weight;
	fn update_channel_payouts() -> Weight;
	fn withdraw_from_channel_balance() -> Weight;
	fn claim_channel_reward(_h: u32, ) -> Weight;
	fn claim_and_withdraw_channel_reward(_h: u32, ) -> Weight;
	fn issue_nft() -> Weight;
	fn destroy_nft() -> Weight;
	fn sling_nft_back() -> Weight;
	fn offer_nft() -> Weight;
	fn cancel_offer() -> Weight;
	fn accept_incoming_offer() -> Weight;
}

/// Weights for content using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87aa6eccf0cc6941ba2e31cdb5870e3229] (r:1 w:0)
	// Storage: unknown [0x2ce461329fdf4be12bce01afc0af09bc13020dc69e85870ac7b4c755bb8753c2] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b872d56750ffbaedbf3dd8dd3900c756381] (r:1 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad97a953f295d54035e7cdf8d29308e498] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad25e0482900c98a1856a1e4878ed6eac6] (r:1 w:0)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:2 w:2)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad3323e092df90358439e7c6649f66d93f] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adbaa937139b20b131cc2fcb0072c015f8] (r:20 w:20)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad33807bc23ee2cb31454339d2b2c6b0a8] (r:101 w:101)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad047d54b603e8604dc1c09c8e0fdc9dc8] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adb7612c99e31defd01cd5a28e9967e208] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6add6fd5c94c285d60cbe96c66929f01c31] (r:1 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad235899829563c4064c97520e04fce94e] (r:1 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adade883233841e9338c8e73f6b9f74890] (r:0 w:10)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:0 w:1)
	fn create_channel(a: u32, b: u32, c: u32, d: u32, e: u32, ) -> Weight {
		(106_978_000 as Weight)
			// Standard Error: 171_000
			.saturating_add((3_220_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 86_000
			.saturating_add((5_072_000 as Weight).saturating_mul(b as Weight))
			// Standard Error: 9_000
			.saturating_add((6_009_000 as Weight).saturating_mul(c as Weight))
			// Standard Error: 171_000
			.saturating_add((2_679_000 as Weight).saturating_mul(d as Weight))
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(e as Weight))
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	// Storage: unknown [0x2ce461329fdf4be12bce01afc0af09bc13020dc69e85870ac7b4c755bb8753c2] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6add6fd5c94c285d60cbe96c66929f01c31] (r:1 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad25e0482900c98a1856a1e4878ed6eac6] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad97a953f295d54035e7cdf8d29308e498] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad047d54b603e8604dc1c09c8e0fdc9dc8] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adb7612c99e31defd01cd5a28e9967e208] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adade883233841e9338c8e73f6b9f74890] (r:10 w:20)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adbaa937139b20b131cc2fcb0072c015f8] (r:20 w:20)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:2 w:2)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad235899829563c4064c97520e04fce94e] (r:1 w:1)
	fn channel_update_with_assets(a: u32, b: u32, c: u32, d: u32, e: u32, ) -> Weight {
		(181_316_000 as Weight)
			// Standard Error: 151_000
			.saturating_add((4_211_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 151_000
			.saturating_add((1_698_000 as Weight).saturating_mul(b as Weight))
			// Standard Error: 151_000
			.saturating_add((4_311_000 as Weight).saturating_mul(c as Weight))
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(d as Weight))
			// Standard Error: 77_000
			.saturating_add((5_239_000 as Weight).saturating_mul(e as Weight))
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(e as Weight)))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(e as Weight)))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	// Storage: unknown [0x2ce461329fdf4be12bce01afc0af09bc13020dc69e85870ac7b4c755bb8753c2] (r:1 w:0)
	fn channel_update_without_assets(a: u32, b: u32, ) -> Weight {
		(119_053_000 as Weight)
			// Standard Error: 127_000
			.saturating_add((3_907_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6add6fd5c94c285d60cbe96c66929f01c31] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adade883233841e9338c8e73f6b9f74890] (r:2 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adbaa937139b20b131cc2fcb0072c015f8] (r:20 w:20)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad33807bc23ee2cb31454339d2b2c6b0a8] (r:101 w:101)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:2 w:2)
	fn delete_channel(a: u32, b: u32, c: u32, ) -> Weight {
		(188_610_000 as Weight)
			// Standard Error: 193_000
			.saturating_add((4_885_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 100_000
			.saturating_add((3_924_000 as Weight).saturating_mul(b as Weight))
			// Standard Error: 11_000
			.saturating_add((4_722_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b870c0ce290812b08a3418d76f63fc3b322] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:0 w:1)
	fn create_curator_group(a: u32, ) -> Weight {
		(15_532_000 as Weight)
			// Standard Error: 11_000
			.saturating_add((906_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:1)
	fn update_curator_group_permissions(a: u32, ) -> Weight {
		(64_837_000 as Weight)
			// Standard Error: 27_000
			.saturating_add((799_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:1)
	fn set_curator_group_status() -> Weight {
		(61_804_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:2 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:1)
	fn add_curator_to_group() -> Weight {
		(98_543_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:1)
	fn remove_curator_from_group() -> Weight {
		(93_192_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b876c94feae87c592d6b11319fb0e516386] (r:1 w:1)
	// Storage: unknown [0x2ce461329fdf4be12bce01afc0af09bc13020dc69e85870ac7b4c755bb8753c2] (r:1 w:0)
	fn initialize_channel_transfer(a: u32, ) -> Weight {
		(121_188_000 as Weight)
			// Standard Error: 271_000
			.saturating_add((3_677_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	fn cancel_channel_transfer() -> Weight {
		(120_472_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457f3928fc443e8d9cca27b4e39e5c29cac] (r:1 w:1)
	fn accept_channel_transfer(a: u32, ) -> Weight {
		(43_954_000 as Weight)
			// Standard Error: 154_000
			.saturating_add((2_394_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad235899829563c4064c97520e04fce94e] (r:1 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad25e0482900c98a1856a1e4878ed6eac6] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad97a953f295d54035e7cdf8d29308e498] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6add6fd5c94c285d60cbe96c66929f01c31] (r:1 w:1)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6ad047d54b603e8604dc1c09c8e0fdc9dc8] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adb7612c99e31defd01cd5a28e9967e208] (r:1 w:0)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:1 w:0)
	// Storage: unknown [0x95875cb80ebaf9f918457db6a86ac6adade883233841e9338c8e73f6b9f74890] (r:0 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b877964dbed9704df430d1b26981274f604] (r:0 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b879668bbcc9610301ef2e64e43972cde4a] (r:0 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87b009e9a04ffa0cbce05e5827c4d4a6f8] (r:0 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87c881f7e86560cfd10e668fc0b706f7c7] (r:0 w:1)
	fn update_channel_payouts() -> Weight {
		(25_863_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:0)
	// Storage: unknown [0x2ce461329fdf4be12bce01afc0af09bc13020dc69e85870ac7b4c755bb8753c2] (r:1 w:0)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:2 w:2)
	fn withdraw_from_channel_balance() -> Weight {
		(48_057_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b877964dbed9704df430d1b26981274f604] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b879668bbcc9610301ef2e64e43972cde4a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87b009e9a04ffa0cbce05e5827c4d4a6f8] (r:1 w:0)
	// Storage: unknown [0xaebd463ed9925c488c112434d61debc0f3928fc443e8d9cca27b4e39e5c29cac] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87c881f7e86560cfd10e668fc0b706f7c7] (r:1 w:0)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:1 w:1)
	fn claim_channel_reward(h: u32, ) -> Weight {
		(63_303_000 as Weight)
			// Standard Error: 139_000
			.saturating_add((932_000 as Weight).saturating_mul(h as Weight))
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec4573091994c5737d8f16ba1c53919a94bf2] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b877964dbed9704df430d1b26981274f604] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b879668bbcc9610301ef2e64e43972cde4a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87b009e9a04ffa0cbce05e5827c4d4a6f8] (r:1 w:0)
	// Storage: unknown [0xaebd463ed9925c488c112434d61debc0f3928fc443e8d9cca27b4e39e5c29cac] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87c881f7e86560cfd10e668fc0b706f7c7] (r:1 w:0)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:1 w:1)
	fn claim_and_withdraw_channel_reward(h: u32, ) -> Weight {
		(71_002_000 as Weight)
			// Standard Error: 131_000
			.saturating_add((778_000 as Weight).saturating_mul(h as Weight))
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b870af9e4882a7d4dfc4aa025e76973af62] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:1)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b878fcac5fb69cd7149f5d142817326cd4f] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87f9ad4eaa35a4c52d9289acbc42eba9d9] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8728ddfed5d1473440d52323ba831817ae] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b877eeddc9ade82616dd2b2522920104f47] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87a7a293d9925f4ae46443ea58e41d0904] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b871ce624c36fa09833f33e5287f370d756] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8799806df27cdcf1eb83a25d651bf93c2d] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87cbb19eafcf52ef3196a3966a6214aa9d] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b872b3448b5048347b84cf9031e0e5dd85d] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b871b10931eafb6faa5fa01f0cf89f95940] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8701f92f535ddd83122720f4e9929b95b2] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87f65b6d352abb4d7727263feb7398e759] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87d2c14024f1b303fdc87019c4c1facfde] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8713013e1b58f6706b9bc1d1f2461e2668] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8780475d76dbd965b5ffe4c9edf3b044a5] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87163a6537c0073cca32731acb69cf63e2] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87fbf3e09a262eab22b5614cc059547717] (r:1 w:1)
	fn issue_nft() -> Weight {
		(162_814_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(21 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b870af9e4882a7d4dfc4aa025e76973af62] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	fn destroy_nft() -> Weight {
		(160_711_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b870af9e4882a7d4dfc4aa025e76973af62] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	fn sling_nft_back() -> Weight {
		(159_621_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b870af9e4882a7d4dfc4aa025e76973af62] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	// Storage: unknown [0x2ce461329fdf4be12bce01afc0af09bc13020dc69e85870ac7b4c755bb8753c2] (r:1 w:0)
	fn offer_nft() -> Weight {
		(165_390_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b870af9e4882a7d4dfc4aa025e76973af62] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:0)
	// Storage: unknown [0x43c6aad67b8d5d8180583e494c8ec457b88c49b6e6ccae735eb57de6295caf6a] (r:1 w:0)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87af96ab8605611c2e562953c9f5fafe05] (r:1 w:0)
	fn cancel_offer() -> Weight {
		(161_364_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b870af9e4882a7d4dfc4aa025e76973af62] (r:1 w:1)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b8727cd538b4597adba02e6593ace94e804] (r:1 w:0)
	// Storage: unknown [0x2ce461329fdf4be12bce01afc0af09bc13020dc69e85870ac7b4c755bb8753c2] (r:1 w:0)
	// Storage: unknown [0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9] (r:2 w:2)
	// Storage: unknown [0xb5a494c92fa4747cc071573e93b32b87dd035684a32b6434aa4ecfa552644c79] (r:1 w:0)
	fn accept_incoming_offer() -> Weight {
		(80_576_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}

// Default implementation for tests
impl WeightInfo for () {
	fn create_channel(a: u32, b: u32, c: u32, d: u32, e: u32, ) -> Weight {
		0
	}
	fn channel_update_with_assets(a: u32, b: u32, c: u32, d: u32, e: u32, ) -> Weight {
		0
	}
	fn channel_update_without_assets(a: u32, b: u32, ) -> Weight {
		0
	}
	fn delete_channel(a: u32, b: u32, c: u32, ) -> Weight {
		0
	}
	fn create_curator_group(a: u32, ) -> Weight {
		0
	}
	fn update_curator_group_permissions(a: u32, ) -> Weight {
		0
	}
	fn set_curator_group_status() -> Weight {
		0
	}
	fn add_curator_to_group() -> Weight {
		0
	}
	fn remove_curator_from_group() -> Weight {
		0
	}
	fn initialize_channel_transfer(a: u32, ) -> Weight {
		0
	}
	fn cancel_channel_transfer() -> Weight {
		0
	}
	fn accept_channel_transfer(a: u32, ) -> Weight {
		0
	}
	fn update_channel_payouts() -> Weight {
		0
	}
	fn withdraw_from_channel_balance() -> Weight {
		0
	}
	fn claim_channel_reward(h: u32, ) -> Weight {
		0
	}
	fn claim_and_withdraw_channel_reward(h: u32, ) -> Weight {
		0
	}
	fn issue_nft() -> Weight {
		0
	}
	fn destroy_nft() -> Weight {
		0
	}
	fn sling_nft_back() -> Weight {
		0
	}
	fn offer_nft() -> Weight {
		0
	}
	fn cancel_offer() -> Weight {
		0
	}
	fn accept_incoming_offer() -> Weight {
		0
	}
}
