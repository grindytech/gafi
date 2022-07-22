use std::{marker::PhantomData, sync::Arc};

use jsonrpsee::{core::{RpcResult as Result, async_trait}, proc_macros::rpc};
use pallet_player_rpc_runtime_api::PlayerRuntimeRPCApi;
use sp_api::{Core, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use devnet as runtime;

use runtime::AccountId;

#[rpc(server, namespace = "player")]
pub trait PlayerApi<BlockHash> {
	#[method(name = "getTotalTimeJoinedUpfront")]
    fn get_total_time_joined_upfront(&self, at: Option<BlockHash>, player: AccountId) -> Result<u128>;
}

pub struct Player<C, P> {
	client: Arc<C>,
	_marker: PhantomData<P>,
}

impl<C, P> Player<C, P> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

#[async_trait]
impl<C, Block> PlayerApiServer<<Block as BlockT>::Hash> for Player<C, Block>
where
	Block: BlockT,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: PlayerRuntimeRPCApi<Block, AccountId>,
{
	fn get_total_time_joined_upfront(&self, at:	Option<Block::Hash>, player: AccountId) -> Result<u128> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.get_total_time_joined_upfront(&at, player);
    	Ok(runtime_api_result.unwrap())
    }
}
