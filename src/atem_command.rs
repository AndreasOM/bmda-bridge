
const SIZE_OF_HEADER: usize = 0x0c;

#[derive(Debug,PartialEq)]
pub enum AtemCommandId {
	Hello = 0x02,
	Ack = 0x10,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum AtemCommand {
	Hello,
	Ack {
		magic: u8,
	},
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum AtemResponse {
	Hello{
		session_id: u16,
		remote_id:	u16,
	}
}



impl AtemCommand {
	pub fn id( &self ) -> AtemCommandId {
		match self {
			AtemCommand::Hello => AtemCommandId::Hello,
			AtemCommand::Ack{ magic }   => AtemCommandId::Ack,
		}
	}
}

impl AtemResponse {
	pub fn from_buffer( buffer: &[u8] ) -> Option< AtemResponse > {

		if buffer.len() < 12 {
			None
		} else {
			let h = buffer[ 0 ] >> 3;
			let plen = ( ( ( ( buffer[ 0 ] & 0x03 ) as u16 ) << 8 ) | ( buffer[1] as u16 ) ) as usize;
			let session_id = ( ( buffer[2] as u16 ) << 8 ) | ( buffer[3] as u16 );
			let ack_id = ( ( buffer[4] as u16 ) << 8 ) | ( buffer[5] as u16 );
			let remote_id = ( ( buffer[10] as u16 ) << 8 ) | ( buffer[11] as u16 );
			if false && buffer.len() != plen {
				println!("Expected length: {}/{:#04x} != {}/{:#04x} :Actual length", plen, plen, buffer.len(), buffer.len() );
				None
			} else {
				if h & AtemCommandId::Hello as u8 != 0 {
					println!("Got HELLO");
					Some( AtemResponse::Hello {
						session_id,
						remote_id,
					} )
				} else {
					println!("Unhandled command {:#02x}", h );
					None
				}
			}
		}
	}	
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum CommandId {
	AckRequest	= 0x01,	// Note: this is actually a bit mask, and may contain multiple in the header :(
	Hello		= 0x02,
	Resend		= 0x04,
	Unknown		= 0x08,
	Ack			= 0x10,
/*
	Command6	= 6,
	Command13	= 13,
	Command17	= 17,
*/
	Invalid		= 0xff,
}

impl Default for CommandId {
	fn default() -> Self {
		CommandId::Invalid
	}
}

impl From<u8> for CommandId {
	fn from( v: u8 ) -> CommandId {
		match v {
			0x01 => CommandId::AckRequest,
			0x02 => CommandId::Hello,
			0x10 => CommandId::Ack,
/*
			6 => CommandId::Command6,
			13 => CommandId::Command13,
			17 => CommandId::Command17,
*/			
			_ => CommandId::Invalid,
		}
	}
}

#[derive()]
pub struct Command {
	id:		CommandId,
	buffer: [u8;1024],
	len:	usize,
}

impl Default for Command {
	fn default() -> Self {
		Self {
			id:		CommandId::default(),
			buffer: [0;1024],
			len:	0,
		}
	}
}

impl core::fmt::Debug for Command {
	fn fmt( &self, f: &mut core::fmt::Formatter ) -> Result<(), std::fmt::Error > {
		f.debug_struct("Command")
            .field("id", &self.id)
            .field("buffer", &format!( "{:?}", &self.buffer[..self.len]))
            .field("len", &self.len)
//            .field("addr", &format_args!("{}", self.addr))
            .finish()

	}
}

impl Command {
	pub fn from_buffer( buffer: &[u8] ) -> Option< Command > {
		if buffer.len() < 12 {
			None
		} else {
			let h = buffer[ 0 ] >> 3;
			let plen = ( ( ( ( buffer[ 0 ] & 0x03 ) as u16 ) << 8 ) | ( buffer[1] as u16 ) ) as usize;
			let session_id = ( ( buffer[2] as u16 ) << 8 ) | ( buffer[3] as u16 );
			let ack_id = ( ( buffer[4] as u16 ) << 8 ) | ( buffer[5] as u16 );
			let remote_id = ( ( buffer[10] as u16 ) << 8 ) | ( buffer[11] as u16 );

			let is_ack_request	= h & 0x01;
			let is_hello		= h & 0x02;
			let is_resend		= h & 0x04;
			let is_ack			= h & 0x10;

			if is_hello {
				
			}
			None
			/*
			if false && buffer.len() != plen {
				println!("Expected length: {} != {} :Actual length", plen, buffer.len() );
				None
			} else {
				match h.into() {
					CommandId::AckRequest => {
						println!("Got ACK_REQUEST");
						Some( Self {
							id:		h.into(),
							buffer: [0;1024],
							len:	20,
	//						session_id,
	//						remote_id,
						} )
					},
					CommandId::Hello => {
						println!("Got HELLO");
						Some( Self {
							id:			h.into(),
							buffer: [0;1024],
							len:	20,
	//						session_id,
	//						remote_id,
						} )						
					},
					CommandId::Command6
					| CommandId::Command13
					| CommandId::Command17 => {
						println!("Got unhandled, but known {}", h );
						Some( Self {
							id:			h.into(),
							buffer: 	[0;1024],
							len:		20,
						} )
					}
					_ => {
						println!("Got unhandled {}", h );
						None
					}
				}
			}
			*/
		}
	}

	pub fn id( &self ) -> CommandId {
		self.id
	}
	pub fn buffer( &self ) -> &[u8] {
		&self.buffer[..self.len]
	}

	fn update_header( &mut self ) {
		let cmd_code = self.id as u8;

		let hsb = ( self.len >> 8 ) as u8;
		let lsb = ( self.len & 0xff ) as u8;
		self.buffer[ 0 ] = ( cmd_code << 3 ) | ( hsb & 0x07 );
		self.buffer[ 1 ] = lsb;

		self.buffer[ 2 ] = 0; // session ID / uID
		self.buffer[ 3 ] = 0;

		self.buffer[ 4 ] = 0; // remote ID / ack ID
		self.buffer[ 5 ] = 0;

		self.buffer[ 10 ] = 0; // package ID
		self.buffer[ 11 ] = 0;

		if ![ CommandId::Hello as u8 ].contains( &cmd_code ) {

		}
	}

	pub fn create_hello( ) -> Command {
		let mut s = Self {
			id: CommandId::Hello,
			buffer: [0;1024],
			len:	20,
		};
		s.buffer[  9 ] = 0x3a;
		s.buffer[ 12 ] = 0x01;
		s.update_header();

		s
	}
}
