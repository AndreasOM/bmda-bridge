
use std::sync::{
	mpsc::{
		self,
		channel
	},
};

use crate::atem_command::{
	AtemCommand,
	AtemResponse,
	Command,
	CommandId,
};

use tokio::net::UdpSocket;

const REMOTE_ADDR: &str = "192.168.186.101:9910";

const PACKET_BUFFER_SIZE: usize = 96;

#[derive(Debug, Default)]
struct Connection {
	sock:		Option< UdpSocket >,
	sessionId:	u16,
	localId:	u16,
	remoteId:	u16,
}

impl Connection {
	pub fn setSessionId(&mut self, sessionId: u16) {
		self.sessionId = sessionId;
	}
	pub fn sessionId( &self ) -> u16 {
		self.sessionId
	}
	pub fn localId( &self ) -> u16 {
		self.localId
	}
	pub fn incLocalId(&mut self) -> u16 {
		self.localId += 1;
		self.localId
	}
	pub fn remoteId( &self ) -> u16 {
		self.remoteId
	}

	fn createHeader( &mut self, cmd: AtemCommand, len: u16, remoteId: u16 ) -> [u8; PACKET_BUFFER_SIZE] {
		let cmd_code = cmd.id() as u8;
		let mut buf =[0; PACKET_BUFFER_SIZE];

		let hsb = ( len >> 8 ) as u8;
		let lsb = ( len & 0xff ) as u8;
		buf[ 0 ] = ( cmd_code << 3 ) | ( hsb & 0x07 );
		buf[ 1 ] = lsb;

		buf[ 2 ] = 0; // session ID / uID
		buf[ 3 ] = 0;

		buf[ 4 ] = 0; // remote ID / ack ID
		buf[ 5 ] = 0;

		buf[ 10 ] = 0; // package ID
		buf[ 11 ] = 0;

		if ![ AtemCommand::Hello ].contains( &cmd ) {

		}
		buf
	}

}

pub struct AtemMini {
	request_tx: Option< mpsc::Sender< AtemCommand > >,
	response_rx: Option< mpsc::Receiver< Command > >,
}

impl AtemMini {
	pub fn new() -> Self {
		Self {
			request_tx: None,
			response_rx: None,
		}
	}

	fn run_handler( &mut self ) -> anyhow::Result<()> {
		let (request_tx, request_rx) = channel();
		let (response_tx, response_rx) = channel();

		self.request_tx = Some( request_tx );
		self.response_rx = Some( response_rx );

		let handle: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move{
			let mut connection = Connection::default();

			let socket = UdpSocket::bind("0.0.0.0:55555").await?;
			let remote_addr = REMOTE_ADDR;
			match socket.connect(remote_addr).await {
				Ok( _ ) => {
					println!("Connected!");
				},
				Err( e ) => {
					println!("Error connecting: {:?}", &e );
				}
			};



			loop {

				// send outgoing requests
				let r = request_rx.try_recv();
				match r {
					Ok( cmd ) => {
						match cmd {
							AtemCommand::Hello => {
								println!("Sending Hello");
								let c = Command::create_hello();
								let buf = c.buffer();

//								let mut buf = connection.createHeader( AtemCommand::Hello, 20, 0 );
//								buf[  9 ] = 0x3a;
//								buf[ 12 ] = 0x01;
								println!("{:?}", &buf[..20]);

								let len = socket.send(&buf[..20]).await?;
								println!("{:?} bytes sent", len);
							},
							AtemCommand::Ack{ magic } => {
								println!("Sending Ack");
								let mut buf = connection.createHeader( cmd, 12, 0 );
								buf[  9 ] = 0x03;
								println!("{:?}", &buf[..12]);

								let len = socket.send(&buf[..12]).await?;
								println!("{:?} bytes sent", len);

							},
						}
					},
					Err( mpsc::TryRecvError::Empty ) => {
//						println!("Empty");
					},
					Err( e ) => {
						println!("{:?}", &e);
					},
				}

				// handle incomming responses
				let mut buf = [0;65535]; //[0;1024];
				match socket.try_recv(&mut buf) {
					Ok(n) => {
						if let Some( cmd ) = Command::from_buffer( &buf[..n] ) {
							println!("Response: {:?}", &cmd);
							response_tx.send( cmd );
						} else {
							println!("Unhandled {:?}", &buf[..n]);
							panic!("Unhandled Response");
						}
					}
					Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
						// no messages
//						println!("?");
					}
					Err(e) => {
						println!("{:?}", &e);
					}
				}
//				println!("!");
//				tokio::task::yield_now().await;
				std::thread::sleep(std::time::Duration::from_millis( 100 ) );

			}
			println!("???");
			Ok(())
		});

		Ok(())
	}
	pub fn connect( &mut self ) -> anyhow::Result<()> {
		self.run_handler();
		if let Some( tx ) = &mut self.request_tx {
			let cmd = AtemCommand::Hello;
			tx.send(cmd)?;
		}

		Ok(())
	}

	pub fn is_connected( &self ) -> bool {
		true
	}

	pub fn update( &mut self ) {
		let max_responses = 10;
		if let Some( response_rx ) = &self.response_rx {
			for _i in 0..max_responses {
				let r = response_rx.try_recv();
				match r {
					Ok( r ) => {
//						dbg!(&r);
						match r.id() {
							CommandId::AckRequest => {
								let cmd = AtemCommand::Ack{ magic: 0x09 };
								if let Some( request_tx ) = &mut self.request_tx {
									request_tx.send( cmd );
								}
							}
							CommandId::Hello => {
								let cmd = AtemCommand::Ack{ magic: 0x09 };
								if let Some( request_tx ) = &mut self.request_tx {
									request_tx.send( cmd );
								}
							},
							/*
							CommandId::Command6 => {
								println!("Ignored command {:?}", &r );
							},
							*/
							c => {
								println!("Unhandled command {:?}", &c );
								panic!("Unhandled command");
							},
						}
					},
					Err( mpsc::TryRecvError::Empty ) => {
						break;
					},
					Err( mpsc::TryRecvError::Disconnected ) => {
						panic!("Connection lost");
						break;
					},
					Err( e ) => {
						println!("{:?}", &e);
						break;
					},
				}
			}
		}
	}
}