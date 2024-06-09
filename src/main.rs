use std::{env, sync::Arc};

use dotenv::dotenv;
use sqlite::State;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client;
use twilight_model::id::{
	marker::{ChannelMarker, UserMarker},
	Id,
};
use twilight_standby::Standby;

use result::Result;
use utils::Context;

mod connect4;
mod result;
mod utils;

#[tokio::main]
async fn main() -> Result {
	dotenv()?;
	let token = env::var("DISCORD_TOKEN")?;
	let sql_path = env::var("DATA")?;

	let connection = sqlite::Connection::open_thread_safe(sql_path)?;
	let query = "CREATE TABLE IF NOT EXISTS scores (id INTEGER,score INTEGER)";
	connection.execute(query)?;
	let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
	let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

	let client = Client::new(token);

	let standby = Standby::new();

	let ctx = Arc::new(Context {
		client,
		standby,
		connection,
	});

	println!("Starting shard");
	loop {
		let event = match shard.next_event().await {
			Ok(event) => event,
			Err(source) => {
				println!("{}", source);
				if source.is_fatal() {
					break;
				}
				continue;
			}
		};
		ctx.standby.process(&event);
		tokio::spawn(handle_event(event, Arc::clone(&ctx)));
	}

	Ok(())
}

async fn handle_event(event: Event, ctx: Arc<Context>) -> Result {
	match event {
		Event::MessageCreate(msg) if msg.content.to_lowercase().contains("hamis start") => {
			if msg.mentions.is_empty() {
				ctx.message(msg.channel_id, "You can't play alone dumbass")
					.await?;
				return Ok(());
			}
			let pinged = &msg.mentions[0];
			connect4::connect_4((msg.author.id, pinged.id), msg.channel_id, ctx).await?;
		}
		Event::MessageCreate(msg) if msg.content.to_lowercase().contains("hamis score") => {
			score(msg.channel_id, msg.author.id, Arc::clone(&ctx)).await?;
		}

		_ => {}
	}
	Ok(())
}

async fn score(channel: Id<ChannelMarker>, id: Id<UserMarker>, ctx: Arc<Context>) -> Result {
	let mut score = 0;

	{
		let query = "SELECT * FROM scores WHERE id = ?";
		let mut statement = ctx.connection.prepare(query)?;
		statement.bind((1, id.into_nonzero().get() as i64))?;
		if let Ok(State::Row) = statement.next() {
			score = statement.read::<i64, _>("score")?;
		} else {
			let insert_query = "INSERT INTO scores VALUES (?,?)";
			let mut statement = ctx.connection.prepare(insert_query)?;
			statement.bind((1, id.into_nonzero().get() as i64))?;
			statement.bind((2, 0))?;
			statement.next()?;
		}
	}

	ctx.message(channel, format!("Your score is: {}", score))
		.await?;

	Ok(())
}
