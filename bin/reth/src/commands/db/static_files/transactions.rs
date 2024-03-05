use super::{
    bench::{bench, BenchKind},
    Command, Compression, PerfectHashingFunction,
};
use rand::{seq::SliceRandom, Rng};
use reth_db::{static_file::TransactionMask, DatabaseEnv};

use reth_primitives::{
    static_file::{Filters, InclusionFilter},
    StaticFileSegment, TransactionSignedNoHash,
};
use reth_provider::{
    providers::StaticFileProvider, BlockNumReader, ProviderError, ProviderFactory,
    TransactionsProvider, TransactionsProviderExt,
};
use std::{path::PathBuf, sync::Arc};

impl Command {
    pub(crate) fn bench_transactions_static_file(
        &self,
        provider_factory: Arc<ProviderFactory<DatabaseEnv>>,
        compression: Compression,
        inclusion_filter: InclusionFilter,
        phf: Option<PerfectHashingFunction>,
    ) -> eyre::Result<()> {
        let provider = provider_factory.provider()?;
        let tip = provider.last_block_number()?;
        let block_range = *self.block_ranges(tip).first().expect("has been generated before");

        let filters = if let Some(phf) = self.with_filters.then_some(phf).flatten() {
            Filters::WithFilters(inclusion_filter, phf)
        } else {
            Filters::WithoutFilters
        };

        let mut rng = rand::thread_rng();

        let tx_range = provider.transaction_range_by_block_range(block_range.into())?;

        let mut row_indexes = tx_range.clone().collect::<Vec<_>>();

        let path: PathBuf = StaticFileSegment::Transactions
            .filename_with_configuration(filters, compression, &block_range)
            .into();
        let provider = StaticFileProvider::new(PathBuf::default())?;
        let jar_provider = provider.get_segment_provider_from_block(
            StaticFileSegment::Transactions,
            self.from,
            Some(&path),
        )?;
        let mut cursor = jar_provider.cursor()?;

        for bench_kind in [BenchKind::Walk, BenchKind::RandomAll] {
            bench(
                bench_kind,
                provider_factory.clone(),
                StaticFileSegment::Transactions,
                filters,
                compression,
                || {
                    for num in row_indexes.iter() {
                        cursor
                            .get_one::<TransactionMask<TransactionSignedNoHash>>((*num).into())?
                            .ok_or(ProviderError::TransactionNotFound((*num).into()))?
                            .with_hash();
                    }
                    Ok(())
                },
                |provider| {
                    for num in row_indexes.iter() {
                        provider
                            .transaction_by_id(*num)?
                            .ok_or(ProviderError::TransactionNotFound((*num).into()))?;
                    }
                    Ok(())
                },
            )?;

            // For random walk
            row_indexes.shuffle(&mut rng);
        }

        // BENCHMARK QUERYING A RANDOM TRANSACTION BY NUMBER
        {
            let num = row_indexes[rng.gen_range(0..row_indexes.len())];
            bench(
                BenchKind::RandomOne,
                provider_factory.clone(),
                StaticFileSegment::Transactions,
                filters,
                compression,
                || {
                    Ok(cursor
                        .get_one::<TransactionMask<TransactionSignedNoHash>>(num.into())?
                        .ok_or(ProviderError::TransactionNotFound(num.into()))?
                        .with_hash())
                },
                |provider| {
                    Ok(provider
                        .transaction_by_id(num as u64)?
                        .ok_or(ProviderError::TransactionNotFound((num as u64).into()))?)
                },
            )?;
        }

        // BENCHMARK QUERYING A RANDOM TRANSACTION BY HASH
        {
            let num = row_indexes[rng.gen_range(0..row_indexes.len())] as u64;
            let transaction_hash = provider_factory
                .transaction_by_id(num)?
                .ok_or(ProviderError::TransactionNotFound(num.into()))?
                .hash();

            bench(
                BenchKind::RandomHash,
                provider_factory,
                StaticFileSegment::Transactions,
                filters,
                compression,
                || {
                    Ok(cursor
                        .get_one::<TransactionMask<TransactionSignedNoHash>>(
                            (&transaction_hash).into(),
                        )?
                        .ok_or(ProviderError::TransactionNotFound(transaction_hash.into()))?
                        .with_hash())
                },
                |provider| {
                    Ok(provider
                        .transaction_by_hash(transaction_hash)?
                        .ok_or(ProviderError::TransactionNotFound(transaction_hash.into()))?)
                },
            )?;
        }
        Ok(())
    }
}
