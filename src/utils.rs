use sqlite::ConnectionThreadSafe;
use twilight_http::Client;
use twilight_standby::Standby;

pub struct Context {
	pub client: Client,
	pub connection: ConnectionThreadSafe,
	pub standby: Standby,
}
