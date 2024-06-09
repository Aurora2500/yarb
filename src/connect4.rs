use std::error::Error;
use std::sync::Arc;

use regex::Regex;
use twilight_model::{
	gateway::payload::incoming::MessageCreate,
	id::{
		marker::{ChannelMarker, UserMarker},
		Id,
	},
};

use crate::utils::Context;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tile {
	Player1,
	Player2,
	Empty,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Turn {
	Player1,
	Player2,
}

impl Turn {
	fn swap(&mut self) {
		*self = match self {
			Turn::Player1 => Turn::Player2,
			Turn::Player2 => Turn::Player1,
		}
	}
}

struct Board {
	tiles: [[Tile; 6]; 7],
	turn: Turn,
	players: (Id<UserMarker>, Id<UserMarker>),
}

impl Board {
	fn new(initiator: Id<UserMarker>, pinged: Id<UserMarker>) -> Board {
		Board {
			tiles: [[Tile::Empty; 6]; 7],
			turn: Turn::Player1,
			players: (initiator, pinged),
		}
	}
}

pub async fn connect_4(
	ids: (Id<UserMarker>, Id<UserMarker>),
	chanel: Id<ChannelMarker>,
	ctx: Arc<Context>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	let message = ctx
		.standby
		.wait_for_message(chanel, move |message: &MessageCreate| {
			message.author.id == ids.1 && message.content.to_lowercase().contains("accept")
		})
		.await;
	if let Err(_error) = message {
		ctx.client
			.create_message(chanel)
			.content("Game declined!!!")?
			.await?;
		return Ok(());
	}

	let mut board = Board::new(ids.0, ids.1);
	ctx.client
		.create_message(chanel)
		.content(&print_board(&board))?
		.await?;
	loop {
		let cur_player = match board.turn {
			Turn::Player1 => board.players.0,
			Turn::Player2 => board.players.1,
		};
		if let Ok(message) = ctx
			.standby
			.wait_for_message(chanel, move |message: &MessageCreate| {
				message.author.id == cur_player
			})
			.await
		{
			ctx.client
				.create_message(chanel)
				.content(&connect_4_turn(&message.content, &mut board))?
				.await?;
			if connect_4_check(&board) {
				ctx.client
					.create_message(chanel)
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
					match turn {
						Turn::Player1 => *tile = Tile::Player1,
						Turn::Player2 => *tile = Tile::Player2,
					}
					board.turn.swap();
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

	// diagonals
	// positive slope

	for colum in 0..4 {
		for row in 0..3 {
			if board.tiles[colum][row] == board.tiles[colum + 1][row + 1]
				&& board.tiles[colum][row] == board.tiles[colum + 2][row + 2]
				&& board.tiles[colum][row] == board.tiles[colum + 3][row + 3]
				&& board.tiles[colum][row] != Tile::Empty
			{
				return true;
			}
		}
	}

	// negative slope
	for colum in 0..4 {
		for row in 3..6 {
			if board.tiles[colum][row] == board.tiles[colum + 1][row - 1]
				&& board.tiles[colum][row] == board.tiles[colum + 2][row - 2]
				&& board.tiles[colum][row] == board.tiles[colum + 3][row - 3]
				&& board.tiles[colum][row] != Tile::Empty
			{
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
