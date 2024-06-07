use dotenv::dotenv;
use regex::Regex;
use std::{env, error::Error, sync::Arc};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client;
use twilight_model::{
	gateway::payload::incoming::MessageCreate,
	id::{
		marker::{ChannelMarker, UserMarker},
		Id,
	},
};
use twilight_standby::Standby;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv()?;
	let token = env::var("DISCORD_TOKEN")?;
	let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
	let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

	let http = Arc::new(Client::new(token));

	let standby = Arc::new(Standby::new());

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
		standby.process(&event);
		tokio::spawn(handle_event(event, Arc::clone(&http), Arc::clone(&standby)));
	}

	Ok(())
}

async fn handle_event(
	event: Event,
	http: Arc<Client>,
	standby: Arc<Standby>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	match event {
		Event::MessageCreate(msg) if msg.content.to_lowercase().contains("hamis start") => {
			if msg.mentions.is_empty() {
				http.create_message(msg.channel_id)
					.content("You can't play alone dumbass")?
					.await?;
				return Ok(());
			}
			let pinged = &msg.mentions[0];
			tokio::spawn(connect_4(
				(msg.author.id, pinged.id),
				standby,
				msg.channel_id,
				http,
			));
		}
		_ => {}
	}
	Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tile {
	Player1,
	Player2,
	Empty,
}

struct Board {
	tiles: [[Tile; 6]; 7],
	turn: bool,
	players: (Id<UserMarker>, Id<UserMarker>),
}
impl Board {
	fn new(initiator: Id<UserMarker>, pinged: Id<UserMarker>) -> Board {
		Board {
			tiles: [[Tile::Empty; 6]; 7],
			turn: false,
			players: (initiator, pinged),
		}
	}
}

async fn connect_4(
	ids: (Id<UserMarker>, Id<UserMarker>),
	standby: Arc<Standby>,
	chanel: Id<ChannelMarker>,
	http: Arc<Client>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	let message = standby
		.wait_for_message(chanel, move |message: &MessageCreate| {
			message.author.id == ids.1 && message.content.to_lowercase().contains("accept")
		})
		.await;
	if let Err(_error) = message {
		http.create_message(chanel)
			.content("Game declined!!!")?
			.await?;
		return Ok(());
	}

	let mut board = Board::new(ids.0, ids.1);
	http.create_message(chanel)
		.content(&print_board(&board))?
		.await?;
	loop {
		let cur_player = if board.turn {
			board.players.1
		} else {
			board.players.0
		};
		if let Ok(message) = standby
			.wait_for_message(chanel, move |message: &MessageCreate| {
				message.author.id == cur_player
			})
			.await
		{
			http.create_message(chanel)
				.content(&connect_4_turn(&message.content, &mut board))?
				.await?;
			if connect_4_check(&board) {
				http.create_message(chanel)
					.content(&format!("decisive victory for <@{}>", cur_player))?
					.await?;
				break;
			}
		}
	}

	Ok(())
}

fn connect_4_turn(string: &String, board: &mut Board) -> String {
	let re = Regex::new(r"(\d)").unwrap();
	if let Some(num_string) = re.find(&string) {
		let num = num_string.as_str().parse::<u32>().unwrap();
		if num <= 7 && num > 0 {
			let turn = board.turn;
			let mut sucess = false;
			for tile in board.tiles[(num - 1) as usize].iter_mut() {
				if *tile == Tile::Empty {
					if turn {
						*tile = Tile::Player2;
					} else {
						*tile = Tile::Player1;
					}
					board.turn = !board.turn;
					sucess = true;
					break;
				}
			}
			if sucess {
				return print_board(&board);
			} else {
				return "Column full".to_string();
			}
		} else {
			return "Out of board limits!".to_string();
		}
	}
	"Error".to_string()
}

fn connect_4_check(board: &Board) -> bool {
	let mut num_colum = 0 as u32;
	let mut last_tile_colum: Tile = Tile::Empty;

	for row in 0..6 as usize {
		for colum in 0..7 as usize {
			if last_tile_colum == board.tiles[colum][row] && last_tile_colum != Tile::Empty {
				num_colum += 1;
			} else {
				num_colum = 1;
			}
			last_tile_colum = board.tiles[colum][row];
			if num_colum == 4 {
				return true;
			}
		}
	}

	let mut num_row = 0 as u32;
	let mut last_tile_row: Tile = Tile::Empty;
	for colum in 0..7 {
		for row in 0..6 as usize as usize {
			if last_tile_row == board.tiles[colum][row] && last_tile_row != Tile::Empty {
				num_row += 1;
			} else {
				num_row = 1;
			}
			last_tile_row = board.tiles[colum][row];
			if num_row == 4 {
				return true;
			}
		}
	}

	false
}

fn print_board(board: &Board) -> String {
	let mut string = "".to_string();
	for row in 0..6 as usize {
		for colum in 0..7 as usize {
			match board.tiles[colum][5 - row] {
				Tile::Empty => string.push_str(":black_circle:"),
				Tile::Player1 => string.push_str(":red_circle:"),
				Tile::Player2 => string.push_str(":yellow_circle:"),
			}
		}
		string.push_str("\n");
	}

	string
}
