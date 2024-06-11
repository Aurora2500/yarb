use std::{env, sync::Arc};

use chrono::Utc;
use dotenv::dotenv;
use regex::Regex;
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
	let query =
		"CREATE TABLE IF NOT EXISTS scores (id INTEGER PRIMARY KEY,score INTEGER,option INTEGER,last_time INTEGER)";
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
			score(msg.channel_id, msg.author.id, ctx).await?;
		}
		Event::MessageCreate(msg) if msg.content.to_lowercase().contains("hamis gamble") => {
			let re = Regex::new(r"(\d+)").unwrap();
			if let Some(num_string) = re.find(&msg.content) {
				let num = num_string.as_str().parse::<i64>().unwrap();
				if num < 0 || num > 9 {
					ctx.message(msg.channel_id, "You need a number from 0 ton9 retard")
						.await?;
				} else {
					gamble(msg.channel_id, msg.author.id, ctx, num).await?;
				}
			} else {
				ctx.message(msg.channel_id, "You need a number from 0 to 9 retard")
					.await?;
			}
		}
		_ => {}
	}
	Ok(())
}

async fn score(channel: Id<ChannelMarker>, id: Id<UserMarker>, ctx: Arc<Context>) -> Result {
	let mut score = 0;
	let mut option = 0;
	let current_time = Utc::now().timestamp();
	let mut last_time = current_time;
	{
		let query = "SELECT * FROM scores WHERE id = ?";
		let mut statement = ctx.connection.prepare(query)?;
		statement.bind((1, id.into_nonzero().get() as i64))?;
		if let Ok(State::Row) = statement.next() {
			score = statement.read::<i64, _>("score")?;
			option = statement.read::<i64, _>("option")?;
			score -= score / 4;
			last_time = statement.read::<i64, _>("last_time")?;
		}
		let seed = last_time % 1000;
		let first = seed / 100;
		let second = (seed / 10) % 10;
		let third = seed % 10;
		let mut amount = 1;
		if first == option {
			amount *= 3;
		}
		if second == option {
			amount *= 3;
		}
		if third == option {
			amount *= 3;
		}

		let time_elapsed = current_time - last_time;
		score += time_elapsed * amount;
		let insert_query = "INSERT OR REPLACE INTO scores VALUES (?,?,?,?)";
		let mut statement = ctx.connection.prepare(insert_query)?;
		statement.bind((1, id.into_nonzero().get() as i64))?;
		statement.bind((2, score))?;
		statement.bind((3, option))?;
		statement.bind((4, current_time))?;
		statement.next()?;
	}
	ctx.message(channel, format!("Your score is: {}", score))
		.await?;

	Ok(())
}

async fn gamble(
	channel: Id<ChannelMarker>,
	id: Id<UserMarker>,
	ctx: Arc<Context>,
	option: i64,
) -> Result {
	let mut last_time = Utc::now().timestamp();
	let mut score = 0;
	{
		let query = "SELECT * FROM scores WHERE id = ?";
		let mut statement = ctx.connection.prepare(query)?;
		if let Ok(State::Row) = statement.next() {
			last_time = statement.read::<i64, _>("last_time")?;
			score = statement.read::<i64, _>("score")?;
		}
		let insert_query = "INSERT OR REPLACE INTO scores VALUES (?,?,?,?)";
		let mut statement = ctx.connection.prepare(insert_query)?;
		statement.bind((1, id.into_nonzero().get() as i64))?;
		statement.bind((2, score))?;
		statement.bind((3, option))?;
		statement.bind((4, last_time))?;
		statement.next()?;
	}
	ctx.message(channel, format!("Gambling on {}", option))
		.await?;
	Ok(())
}
