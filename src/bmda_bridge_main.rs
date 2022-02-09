use bmda_bridge::AtemMini;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let mut am = AtemMini::new();

	am.connect()?;

	while am.is_connected() {
		std::thread::sleep(std::time::Duration::from_millis( 1000 ) );
//		println!(".");
		am.update();	// :HACK:
	};

	Ok(())
}
