use dotenv::dotenv;
use std::{env, error::Error, sync::Arc};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv()?;
	let token = env::var("DISCORD_TOKEN")?;
	let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
	let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

	let http = Arc::new(Client::new(token));

	println!("Starting shard");
	while let Ok(event) = shard.next_event().await {
		tokio::spawn(handle_event(event, Arc::clone(&http)));
	}

	Ok(())
}

async fn handle_event(event: Event, http: Arc<Client>) -> Result<(), Box<dyn Error + Send + Sync>> {
	match event {
		Event::MessageCreate(msg) if msg.content == "!ping" => {
			http.create_message(msg.channel_id)
				.content("Pong!")?
				.await?;
		}
		_ => {}
	}
	Ok(())
}
