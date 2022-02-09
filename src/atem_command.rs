
// hello
// [16, 20, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]
// response
// [16, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 110, 0, 0, 0, 0]
const SIZE_OF_HEADER: usize = 0x0c;

const COMMAND_MASK_ACK_REQUEST: u8 = 0x01;
const COMMAND_MASK_HELLO: u8 = 0x02;
const COMMAND_MASK_RESEND: u8 = 0x04;
const COMMAND_MASK_REQUEST_NEXT: u8 = 0x08;
const COMMAND_MASK_ACK: u8 = 0x10;

#[derive(Debug,Default,PartialEq)]
pub struct AtemCommandHeader {
	cmd:	u8,
	len:	u16,
	session_id:	u16,
	ack_id:	u16,
	package_id: u16,
	dirty:	bool,
	buffer: [u8;SIZE_OF_HEADER],
}

impl AtemCommandHeader {
	pub fn from_buffer( buffer: &[u8;SIZE_OF_HEADER] ) -> Option< AtemCommandHeader > {
		let mut h = AtemCommandHeader::default();
		h.buffer = *buffer;

		h.cmd = buffer[ 0 ] >> 3;
		h.len = ( ( ( buffer[ 0 ] & 0x03 ) as u16 ) << 8 ) | ( buffer[1] as u16 ) - SIZE_OF_HEADER as u16;
		h.session_id = ( ( buffer[2] as u16 ) << 8 ) | ( buffer[3] as u16 );
		h.ack_id = ( ( buffer[4] as u16 ) << 8 ) | ( buffer[5] as u16 );
		h.package_id = ( ( buffer[10] as u16 ) << 8 ) | ( buffer[11] as u16 );

		// :TODO: error checking
		Some( h )
	}

	pub fn cmd(&self) -> u8 {
		self.cmd
	}
	pub fn is_ack(&self) -> bool {
		self.cmd & COMMAND_MASK_ACK > 0
	}
	pub fn is_hello(&self) -> bool {
		self.cmd & COMMAND_MASK_HELLO > 0
	}
	pub fn is_resend(&self) -> bool {
		self.cmd & COMMAND_MASK_RESEND > 0
	}
	pub fn is_ack_request(&self) -> bool {
		self.cmd & COMMAND_MASK_ACK_REQUEST > 0
	}
	pub fn session_id(&self) -> u16 {
		self.session_id
	}
	pub fn package_id(&self) -> u16 {
		self.package_id
	}
	pub fn len(&self) -> u16 {
		self.len
	}

	pub fn set_command( &mut self, cmd: u8 ) {
		self.cmd = cmd;
		self.dirty = true;
	}

	pub fn set_len( &mut self, len: u16 ) {
		self.len = len;// len is without header+SIZE_OF_HEADER as u16;
		self.dirty = true;
	}

	pub fn set_session_id( &mut self, session_id: u16 ) {
		self.session_id = session_id;
		self.dirty = true;
	}

	pub fn set_package_id( &mut self, package_id: u16 ) {
		self.package_id = package_id;
		self.dirty = true;
	}

	pub fn set_ack_id( &mut self, ack_id: u16 ) {
		self.ack_id = ack_id;
		self.dirty = true;
	}

	pub fn set( &mut self, index: usize, value: u8 ) {
		self.buffer[ index ] = value;
		self.dirty = true
	}

	pub fn update_buffer( &mut self ) {
		let len = self.len + SIZE_OF_HEADER as u16;
		let hsb = ( len >> 8 ) as u8;
		let lsb = ( len & 0xff ) as u8;

		self.buffer[ 0 ] = ( self.cmd << 3 ) | ( hsb & 0x07 );
		self.buffer[ 1 ] = lsb;

		self.buffer[ 2 ] = ( self.session_id >> 8 ) as u8;; // session ID / uID
		self.buffer[ 3 ] = ( self.session_id & 0xff ) as u8;
		println!("session {:#04x} -> {:#02x} {:#02x}", self.session_id, self.buffer[ 2 ], self.buffer[ 3 ]);

		self.buffer[ 4 ] = ( self.ack_id >> 8 ) as u8;; // ack ID
		self.buffer[ 5 ] = ( self.ack_id & 0xff ) as u8;
		println!("ack {:#04x} -> {:#02x} {:#02x}", self.ack_id, self.buffer[ 4 ], self.buffer[ 5 ]);

		self.buffer[ 10 ] = ( self.package_id >> 8 ) as u8;; // package ID 
		self.buffer[ 11 ] = ( self.package_id & 0xff ) as u8;
		println!("package {:#04x} -> {:#02x} {:#02x}", self.package_id, self.buffer[ 10 ], self.buffer[ 11 ]);

		self.dirty = false;
	}

	pub fn buffer( &self ) -> &[u8] {
		if self.dirty {
			panic!("Tried to use dirty buffer");
		}
		&self.buffer
	}
}

#[derive(Debug,PartialEq)]
pub struct AtemCommandPayload {
	buffer: Vec::<u8>,
	dirty:	bool,
}

impl Default for AtemCommandPayload {
	fn default() -> Self {
		Self {
			buffer: Vec::new(),
			dirty: 	false,
		}
	}
}

impl AtemCommandPayload {
	pub fn from_buffer( buffer: &[u8] ) -> Option< AtemCommandPayload > {
		let mut p = AtemCommandPayload::default();
		p.buffer = buffer.into();

		let mut o = 0;
		while( o+2 < buffer.len() ) {
			let l = if o+10 > buffer.len() {
				buffer.len()
			} else {
				o+10
			};

			println!("{:?}", &buffer[ o+0 .. l ] );
		    let size = ( ( buffer[ o+0 ] as u16 ) << 8 ) | ( buffer[ o+1 ] as u16 );
		    if size == 0 {
		    	break;
		    }
		    println!("Chunk Size: {:#04x} from {:#02x} {:#02x} {} {}", size, buffer[ o+1 ], buffer[ o+0 ], buffer[ o+1 ], buffer[ o+0 ]);

		    let s = o+2;
		    let e = s+( size as usize )-2;
		    let chunk = &buffer[ o+2..e ];
		    println!("Chunk: {:?}", &chunk );
		    let mut name = [0;4];
		    name[ 0 ] = chunk[ 0 + 2 ];
		    name[ 1 ] = chunk[ 1 + 2 ];
		    name[ 2 ] = chunk[ 2 + 2 ];
		    name[ 3 ] = chunk[ 3 + 2 ];
		    println!("{:?}", &name);

		    let name = String::from_utf8_lossy( &name );
		    println!("{:?}", &name);

		    match name.as_ref() {
		    	"TlIn" => {
		    		println!("Got Tally Info");
		    		let count = ( ( ( chunk[ 6 ] as u16 ) <<8 ) | ( chunk[ 7 ] as u16 ) ) as usize;
		    		println!("Count: {}", count);
		    		for i in 0..count {
		    			let t = chunk[ 8 + i ];
		    			println!("{} -> {}", i, t );
		    		}
		    	},
		    	"KeOn" => {
		    		let w = chunk[ 6 ];
		    		let i = chunk[ 7 ];
		    		let o = chunk[ 8 ];

		    		println!("KeOn {} {} {}", w, i, o );
		    	}
		    	o => {
		    		println!("Unhandled chunk type: {:?}", o );
		    	}
		    }
			o+=size as usize;
		}

		Some( p )
	}

	pub fn set_len( &mut self, len: u16 ) {
//		if self.buffer.capacity() < len {
			self.buffer.resize(len as usize, 0);
//		}
	}
	pub fn set( &mut self, index: usize, value: u8 ) {
		self.buffer[ index ] = value;
		self.dirty = true
	}

	pub fn update_buffer( &mut self ) {
		self.dirty = false;
	}

	pub fn buffer( &self ) -> &[u8] {
		if self.dirty {
			panic!("Tried to use dirty buffer");
		}
		&self.buffer
	}
}

#[derive(Debug,PartialEq)]
pub struct AtemCommand {
	header:		AtemCommandHeader,
	payload:	AtemCommandPayload,
	buffer:		Vec::< u8 >,
	dirty:		bool,
}

impl Default for AtemCommand {
	fn default() -> Self {
		Self {
			header:		AtemCommandHeader::default(),
			payload: 	AtemCommandPayload::default(),
			buffer:		Vec::new(),
			dirty:		false,
		}
	}
}

impl AtemCommand {
	pub fn from_buffer(buffer: &[u8]) -> Option< AtemCommand > {
		if buffer.len() < 12 {
			None
		} else {
			let ( bh, bp ) = buffer.split_at( SIZE_OF_HEADER );
			let h = AtemCommandHeader::from_buffer( &bh.try_into().ok()? );

			let is_hello = if let Some( h ) = &h {
				h.is_hello()
			} else {
				true
			};
			let p = if is_hello {
				None
			} else {
				AtemCommandPayload::from_buffer( &bp )
			};

			if let Some( h ) = h {
				let plen = h.len() as usize + SIZE_OF_HEADER;
				if false && buffer.len() != plen {
					println!("Expected length: {}/{:#04x} != {}/{:#04x} :Actual length", plen, plen, buffer.len(), buffer.len() );
					dbg!(&h);
//					dbg!(&p);
					todo!("wrong length");
					None
				} else {
					if h.is_hello() {
						println!("Got HELLO. Length {}", plen);
						let mut ac = AtemCommand::default();
						ac.header = h;
						if let Some( p ) = p {
							ac.payload = p;
						}
						Some( ac )
					} else if h.is_ack_request() {
						println!("Got ACK_REQUEST. Length {}", plen);
						let mut ac = AtemCommand::default();
						ac.header = h;
						if let Some( p ) = p {
							ac.payload = p;
						}
						Some( ac )
					} else {
						println!("Unhandled command {:#02x}", h.cmd() );
						None
					}
				}
			} else {
				panic!("Error parsing header");
			}

		}
	}

	pub fn header( &self ) -> &AtemCommandHeader {
		&self.header
	}

	fn set_payload_len( &mut self, len: u16 ) {
		self.header.set_len( len );
		self.payload.set_len( len );
	}
	pub fn create_hello() -> AtemCommand {
		let mut s = Self::default();
		s.set_payload_len( 8 );
//		s.header.set_len( 8 );	// payload size
		s.header.set_command( COMMAND_MASK_HELLO );
		s.header.set(  9, 0x3a );
		s.payload.set( 0 /*12*/, 0x01 );

		s.update_buffer();
		s
	}
	pub fn create_ack( package_id: u16, session_id: u16, ack_id: u16 ) -> AtemCommand {
		let mut s = Self::default();
		s.set_payload_len( 0 );
//		s.header.set_len( 8 );	// payload size
		s.header.set_command( COMMAND_MASK_ACK );
		s.header.set_package_id( package_id );
		s.header.set_session_id( session_id );
		s.header.set_ack_id( ack_id );
		s.header.set( 9, 0x03 );
//		s.header.set(  9, 0x3a );
//		s.payload.set( 0 /*12*/, 0x01 );

		s.update_buffer();
		s
	}
	pub fn update_buffer( &mut self ) {
		self.header.update_buffer();
		self.payload.update_buffer();
		let b = [self.header.buffer(), self.payload.buffer()].concat();
		self.buffer = b;
		self.dirty = false;
	}

	pub fn buffer( &self ) -> &[u8] {
		if self.dirty {
			panic!("Tried to use dirty buffer");
		}
		&self.buffer
	}

}

#[derive(Debug,PartialEq)]
pub enum Command {
	Hello,
	Ack( u16, u16 ),
	AtemCommand( AtemCommand ),
	Shutdown,
}

/*
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

*/

/*
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
*/
