use bmda_bridge::AtemMini;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let mut am = AtemMini::new();

	am.connect()?;

	let mut wait = 0;
	while am.is_connected() {
		std::thread::sleep(std::time::Duration::from_millis( 1 ) );
//		println!(".");
		am.update();	// :HACK:
		wait += 1;

		if wait == 5000 {
			println!("Run Macro Test 0");
			am.run_macro( 0 );
		} else if wait == 7000 {
			println!("Run Macro Test 1");
			am.run_macro( 1 );
		} else if wait == 10000 {
			println!("Run Macro Test 0");
			am.run_macro( 0 );
		}
	};

	Ok(())
}
