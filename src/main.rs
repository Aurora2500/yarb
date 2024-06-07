use dotenv::dotenv;
use std::env;
use twilight_gateway::{Intents, Shard, ShardId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv()?;
	let token = env::var("DISCORD_TOKEN")?;
	let intents = Intents::GUILD_MESSAGES;
	let mut shard = Shard::new(ShardId::ONE, token, intents);

	println!("Starting shard");
	while let Ok(event) = shard.next_event().await {
		println!("{:?}", event);
	}

	Ok(())
}
