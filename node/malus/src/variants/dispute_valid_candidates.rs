// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! A malicious node that replaces approvals with invalid disputes
//! against valid candidates.
//!
//! Attention: For usage with `zombienet` only!

#![allow(missing_docs)]

use polkadot_cli::{
	prepared_overseer_builder,
	service::{
		AuthorityDiscoveryApi, AuxStore, BabeApi, Block, Error, HeaderBackend, Overseer,
		OverseerConnector, OverseerGen, OverseerGenArgs, OverseerHandle, ParachainHost,
		ProvideRuntimeApi, SpawnNamed,
	},
};

// Filter wrapping related types.
use crate::{interceptor::*, shared::MALUS, FakeCandidateValidation};
use polkadot_node_primitives::{InvalidCandidate, ValidationResult};

// Import extra types relevant to the particular
// subsystem.
use polkadot_node_core_backing::CandidateBackingSubsystem;
use polkadot_node_core_candidate_validation::CandidateValidationSubsystem;

use polkadot_node_subsystem::messages::{
	ApprovalDistributionMessage, CandidateBackingMessage, CandidateValidationMessage,
	DisputeCoordinatorMessage,
};
use sp_keystore::SyncCryptoStorePtr;

use std::sync::Arc;

/// Replace outgoing approval messages with disputes.
#[derive(Clone, Debug)]
struct ReplaceApprovalsWithDisputes;

#[derive(Clone, Debug)]
struct ReplaceValidationResult(FakeCandidateValidation);

impl<Sender> MessageInterceptor<Sender> for ReplaceApprovalsWithDisputes
where
	Sender: overseer::SubsystemSender<CandidateBackingMessage> + Clone + Send + 'static,
{
	type Message = CandidateBackingMessage;

	fn intercept_incoming(
		&self,
		_sender: &mut Sender,
		msg: FromOverseer<Self::Message>,
	) -> Option<FromOverseer<Self::Message>> {
		Some(msg)
	}

	fn intercept_outgoing(&self, msg: AllMessages) -> Option<AllMessages> {
		match msg {
			AllMessages::ApprovalDistribution(ApprovalDistributionMessage::DistributeApproval(
				_,
			)) => {
				// drop the message on the floor
				None
			},
			AllMessages::DisputeCoordinator(DisputeCoordinatorMessage::ImportStatements {
				candidate_hash,
				candidate_receipt,
				session,
				..
			}) => {
				tracing::info!(
					target = MALUS,
					para_id = ?candidate_receipt.descriptor.para_id,
					?candidate_hash,
					"Disputing candidate",
				);
				// this would also dispute candidates we were not assigned to approve
				Some(AllMessages::DisputeCoordinator(
					DisputeCoordinatorMessage::IssueLocalStatement(
						session,
						candidate_hash,
						candidate_receipt,
						false,
					),
				))
			},
			msg => Some(msg),
		}
	}
}

impl<Sender> MessageInterceptor<Sender> for ReplaceValidationResult
where
	Sender: overseer::SubsystemSender<CandidateValidationMessage> + Clone + Send + 'static,
{
	type Message = CandidateValidationMessage;

	// Capture all candidate validation requests and fail them.
	// MaybeTODO: add option to configure the failure reason.
	fn intercept_incoming(
		&self,
		_sender: &mut Sender,
		msg: FromOverseer<Self::Message>,
	) -> Option<FromOverseer<Self::Message>> {
		if self.0 == FakeCandidateValidation::None {
			return Some(msg)
		}

		match msg {
			FromOverseer::Communication {
				msg:
					CandidateValidationMessage::ValidateFromExhaustive(_, _, descriptor, _, _, sender),
			} => {
				let validation_result = ValidationResult::Invalid(InvalidCandidate::InvalidOutputs);

				tracing::info!(
					target = MALUS,
					para_id = ?descriptor.para_id,
					candidate_hash = ?descriptor.para_head,
					"ValidateFromExhaustive result: {:?}",
					&validation_result
				);

				// We're not even checking the candidate, this makes us appear faster than honest validators.
				sender.send(Ok(validation_result)).unwrap();
				None
			},
			FromOverseer::Communication {
				msg: CandidateValidationMessage::ValidateFromChainState(descriptor, _, _, sender),
			} => {
				let validation_result = ValidationResult::Invalid(InvalidCandidate::InvalidOutputs);

				tracing::info!(
					target = MALUS,
					para_id = ?descriptor.para_id,
					candidate_hash = ?descriptor.para_head,
					"ValidateFromChainState result: {:?}",
					&validation_result
				);

				// We're not even checking the candidate, this makes us appear faster than honest validators.
				sender.send(Ok(validation_result)).unwrap();
				None
			},
			msg => Some(msg),
		}
	}

	fn intercept_outgoing(&self, msg: AllMessages) -> Option<AllMessages> {
		Some(msg)
	}
}

/// Generates an overseer that disputes instead of approving valid candidates.
pub(crate) struct DisputeValidCandidates {
	fake_candidate_validation: FakeCandidateValidation,
}

impl Default for DisputeValidCandidates {
	fn default() -> Self {
		Self { fake_candidate_validation: FakeCandidateValidation::None }
	}
}

impl DisputeValidCandidates {
	pub fn new(fake_candidate_validation: FakeCandidateValidation) -> Self {
		Self { fake_candidate_validation }
	}
}

impl OverseerGen for DisputeValidCandidates {
	fn generate<'a, Spawner, RuntimeClient>(
		&self,
		connector: OverseerConnector,
		args: OverseerGenArgs<'a, Spawner, RuntimeClient>,
	) -> Result<(Overseer<Spawner, Arc<RuntimeClient>>, OverseerHandle), Error>
	where
		RuntimeClient: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block> + AuxStore,
		RuntimeClient::Api: ParachainHost<Block> + BabeApi<Block> + AuthorityDiscoveryApi<Block>,
		Spawner: 'static + SpawnNamed + Clone + Unpin,
	{
		let spawner = args.spawner.clone();
		let crypto_store_ptr = args.keystore.clone() as SyncCryptoStorePtr;
		let backing_filter = ReplaceApprovalsWithDisputes;
		let validation_filter = ReplaceValidationResult(self.fake_candidate_validation.clone());
		let candidate_validation_config = args.candidate_validation_config.clone();

		prepared_overseer_builder(args)?
			.replace_candidate_backing(move |cb_subsystem| {
				InterceptedSubsystem::new(
					CandidateBackingSubsystem::new(
						spawner,
						crypto_store_ptr,
						cb_subsystem.params.metrics,
					),
					backing_filter,
				)
			})
			.replace_candidate_validation(move |cv_subsystem| {
				InterceptedSubsystem::new(
					CandidateValidationSubsystem::with_config(
						candidate_validation_config,
						cv_subsystem.metrics,
						cv_subsystem.pvf_metrics,
					),
					validation_filter,
				)
			})
			.build_with_connector(connector)
			.map_err(|e| e.into())
	}
}
