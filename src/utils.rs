use sqlite::ConnectionThreadSafe;
use twilight_http::Client;
use twilight_model::id::{marker::ChannelMarker, Id};
use twilight_standby::Standby;

use crate::result::Result;

pub struct Context {
	pub client: Client,
	pub connection: ConnectionThreadSafe,
	pub standby: Standby,
}

impl Context {
	pub async fn message(&self, channel: Id<ChannelMarker>, content: impl AsRef<str>) -> Result {
		self.client
			.create_message(channel)
			.content(content.as_ref())?
			.await?;

		Ok(())
	}
}
