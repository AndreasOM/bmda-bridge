use crate::payload::Payload;

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

#[derive(Debug, Default, PartialEq)]
pub struct AtemCommandHeader {
    cmd: u8,
    len: u16,
    session_id: u16,
    ack_id: u16,
    resend_id: u16,
    package_id: u16,
    dirty: bool,
    buffer: [u8; SIZE_OF_HEADER],
}

impl AtemCommandHeader {
    pub fn from_buffer(buffer: &[u8; SIZE_OF_HEADER]) -> Option<AtemCommandHeader> {
        let mut h = AtemCommandHeader::default();
        h.buffer = *buffer;

        h.cmd = buffer[0] >> 3;
        h.len = (((buffer[0] & 0x03) as u16) << 8) | (buffer[1] as u16) - SIZE_OF_HEADER as u16;
        h.session_id = ((buffer[2] as u16) << 8) | (buffer[3] as u16);
        h.ack_id = ((buffer[4] as u16) << 8) | (buffer[5] as u16);
        h.resend_id = ((buffer[6] as u16) << 8) | (buffer[7] as u16);
        h.package_id = ((buffer[10] as u16) << 8) | (buffer[11] as u16);

        // :TODO: error checking
        Some(h)
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
    pub fn is_request_next(&self) -> bool {
        self.cmd & COMMAND_MASK_REQUEST_NEXT > 0
    }
    pub fn session_id(&self) -> u16 {
        self.session_id
    }
    pub fn package_id(&self) -> u16 {
        self.package_id
    }
    pub fn ack_id(&self) -> u16 {
        self.ack_id
    }
    pub fn resend_id(&self) -> u16 {
        self.resend_id
    }
    pub fn len(&self) -> u16 {
        self.len
    }

    pub fn set_command(&mut self, cmd: u8) {
        self.cmd = cmd;
        self.dirty = true;
    }

    pub fn set_len(&mut self, len: u16) {
        self.len = len; // len is without header+SIZE_OF_HEADER as u16;
        self.dirty = true;
    }

    pub fn set_session_id(&mut self, session_id: u16) {
        self.session_id = session_id;
        self.dirty = true;
    }

    pub fn set_package_id(&mut self, package_id: u16) {
        self.package_id = package_id;
        self.dirty = true;
    }

    pub fn set_ack_id(&mut self, ack_id: u16) {
        self.ack_id = ack_id;
        self.dirty = true;
    }

    pub fn set(&mut self, index: usize, value: u8) {
        self.buffer[index] = value;
        self.dirty = true
    }

    pub fn update_buffer(&mut self) {
        let len = self.len + SIZE_OF_HEADER as u16;
        let hsb = (len >> 8) as u8;
        let lsb = (len & 0xff) as u8;

        self.buffer[0] = (self.cmd << 3) | (hsb & 0x07);
        self.buffer[1] = lsb;

        self.buffer[2] = (self.session_id >> 8) as u8; // session ID / uID
        self.buffer[3] = (self.session_id & 0xff) as u8;
        //		println!("session {:#04x} -> {:#02x} {:#02x}", self.session_id, self.buffer[ 2 ], self.buffer[ 3 ]);

        self.buffer[4] = (self.ack_id >> 8) as u8; // ack ID
        self.buffer[5] = (self.ack_id & 0xff) as u8;
        //		println!("ack {:#04x} -> {:#02x} {:#02x}", self.ack_id, self.buffer[ 4 ], self.buffer[ 5 ]);

        self.buffer[10] = (self.package_id >> 8) as u8; // package ID
        self.buffer[11] = (self.package_id & 0xff) as u8;
        //		println!("package {:#04x} -> {:#02x} {:#02x}", self.package_id, self.buffer[ 10 ], self.buffer[ 11 ]);

        self.dirty = false;
    }

    pub fn buffer(&self) -> &[u8] {
        if self.dirty {
            panic!("Tried to use dirty buffer");
        }
        &self.buffer
    }
}

#[derive(Debug)]
pub struct AtemCommandPayload {
    payloads: Vec<Payload>, // read only for now
    dirty: bool,
    buffer: Vec<u8>,
}

impl Default for AtemCommandPayload {
    fn default() -> Self {
        Self {
            payloads: Vec::new(),
            dirty: false,
            buffer: Vec::new(),
        }
    }
}

const IGNORED_CHUNKS: &'static [&str] = &[
    "Time",
    "CCdP", // camera
    "FTDC", // finished transfer data
    "_MeC", // mix effects
    "_mpl", // media pool
    "_MvC", // multi view (count?)
    "_SSC",
    "_FAC",
    "_FEC",
    "_FMH",
    "_VMC", // video modes?
    "_DVE",
    "Powr", // power?
    "AiVM",
    "TcLK",
    "TCCc",
    "MvVM",
    "MvPr", // multi view "program"?
    "MvIn", // multi view input?
    "VuMC",
    "SaMw",
    "VuMo",
    "TrSS", // transition
    "TrPr",
    "TrPs",
    "TMxP",
    "TDpP",
    "TWpP",
    "TDvP",
    "TStP",
    "KeBP",
    /// keyer!
    "KBfT",
    "KeLm",
    "KACk",
    "KACC",
    "KePt",
    "KeDV",
    "KeFS",
    "KKFP",
    "DskB",
    "DskP",
    "FtbP",
    "FtbS",
    "MPfe", // media player
    "MPCE",
    "CapA",
    "RXMS",
    "RXCP",
    "RXSS",
    "RXCC",
    "SSrc",
    "SSBP",
    "FMPP",
    "FAIP",
    "FIEP",
    "AEBP",
    "AIXP",
    "AICP",
    "AILP",
    "FASP",
    "FMTl",
    "FAMP",
    "AMBP",
    "MOCP",
    "AMLP",
    "FMHP",
    "FAMS",
    "TlSr", // tally state? !
    "TlFc",
    //	"MRPr",	// macro run?
    "MRcS", // macro recording?
    "CCst",
    "RMSu",
    "RTMS",
    "RTMR",
    "RMRD",
    "SRSU",
    "STAB",
    "StRS",
    "SRST",
    "SRSD",
    "SRRS",
    "SAth",
    "SLow",
    "NIfT",
    "LKST",
    "SRSS",
];

fn word_at(buffer: &[u8], index: usize) -> u16 {
    if index + 1 >= buffer.len() {
        println!("Tried to read past end of buffer");
        0
    } else {
        ((buffer[index + 0] as u16) << 8) | (buffer[index + 1] as u16)
    }
}

fn string_at(buffer: &[u8], index: usize, len: Option<usize>) -> String {
    let b = if let Some(len) = len {
        &buffer[index..index + len]
    } else {
        &buffer[index..]
    };

    String::from_utf8_lossy(b).to_string()
}

impl AtemCommandPayload {
    pub fn from_buffer(buffer: &[u8]) -> Option<AtemCommandPayload> {
        let mut p = AtemCommandPayload::default();
        p.buffer = buffer.into();

        let mut o = 0;
        while (o + 2 < buffer.len()) {
            let l = if o + 10 > buffer.len() {
                buffer.len()
            } else {
                o + 10
            };

            //			println!("{:?}", &buffer[ o+0 .. l ] );
            //		    let size = ( ( buffer[ o+0 ] as u16 ) << 8 ) | ( buffer[ o+1 ] as u16 );
            let size = word_at(&buffer, o);
            if size == 0 {
                break;
            }
            //		    println!("Chunk Size: {:#04x} from {:#02x} {:#02x} {} {}", size, buffer[ o+1 ], buffer[ o+0 ], buffer[ o+1 ], buffer[ o+0 ]);

            let s = o + 2;
            let e = s + (size as usize) - 2;
            let chunk = &buffer[o + 2..e];
            //		    println!("Chunk: {:?}", &chunk );
            let mut name = [0; 4];
            name[0] = chunk[0 + 2];
            name[1] = chunk[1 + 2];
            name[2] = chunk[2 + 2];
            name[3] = chunk[3 + 2];
            //		    println!("{:?}", &name);

            let name = String::from_utf8_lossy(&name);
            //		    println!("{:?}", &name);

            match name.as_ref() {
                "InCm" => {
                    println!("InCm: {:?}", &chunk);
                }
                "_ver" => {
                    let maj = word_at(&buffer, 6);
                    let min = word_at(&buffer, 8);
                    println!("Got version {}.{}", maj, min);
                }
                "_pin" => {
                    let pin = string_at(&chunk, 6, None);
                    println!("Got pin >{}<", pin);
                }
                "_top" => {
                    let me_count = chunk[6];
                    let source_count = chunk[7];
                    let colgen_count = chunk[8];
                    let auxbus_count = chunk[9];
                    // 10?
                    let dsk_count = chunk[11];
                    // 12?
                    let usk_count = chunk[13];
                    let stinger_count = chunk[14];
                    let dve_count = chunk[15];
                    let ss_count = chunk[16];
                    let sd = chunk[17];
                    println!("Got Topology");
                }
                "_TlC" => {
                    let c = word_at(&chunk, 6);
                    println!("Tally Channel Count: {}", c);
                }
                "AuxS" => {
                    println!("Got Auxiliary Source");
                    let i = chunk[6];
                    let v = word_at(&buffer, 8);
                    println!("{} -> {}", i, v);
                }
                "DskS" => {
                    println!("Got Downstream Keyer");
                    let i = chunk[6];
                    let on = chunk[7];
                    let trans = chunk[8];
                    let auto_trans = chunk[9];
                    let frame = chunk[9];
                    let v = word_at(&buffer, 8);
                    println!("{} -> {} ({}/{}/{})", i, on, trans, auto_trans, v);
                }
                "TlIn" => {
                    println!("Got Tally Info");
                    let count = (((chunk[6] as u16) << 8) | (chunk[7] as u16)) as usize;
                    println!("Count: {}", count);
                    for i in 0..count {
                        let t = chunk[8 + i];
                        println!("{} -> {}", i, t);
                    }
                }
                "InPr" => {
                    let i = word_at(&chunk, 6);
                    let lt = string_at(&chunk, 8, Some(20));
                    let st = string_at(&chunk, 28, Some(4));
                    let et = chunk[37];
                    let it = chunk[38];
                    let avail = chunk[40];
                    let mea = chunk[41];

                    println!("Input: {:>8} {:<4} | {:<20}", i, st, lt);
                }
                "PrgI" => {
                    let me = chunk[6];
                    let input = word_at(&chunk, 8);
                    println!("Program Input: {} -> {}", me, input);
                }
                "PrvI" => {
                    let me = chunk[6];
                    let input = word_at(&chunk, 8);
                    println!("Preview Input: {} -> {}", me, input);
                }
                "KeOn" => {
                    let w = chunk[6];
                    let i = chunk[7];
                    let s = chunk[8];

                    println!("KeOn {} {} {}", w, i, s);
                    p.payloads.push(Payload::KeOn {
                        who: w,
                        index: i,
                        state: s,
                    });
                }
                "_MAC" => {
                    let c = chunk[6];
                    println!("Got Macro Count: {}", c);
                }
                "MPrp" => {
                    // macro?
                    let i = chunk[7];
                    let u = chunk[8];
                    let name_len = (((chunk[10] as u16) << 8) | (chunk[11] as u16)) as usize;
                    let body_len = (((chunk[12] as u16) << 8) | (chunk[13] as u16)) as usize;

                    let name = if name_len > 0 {
                        let n = &chunk[14..14 + name_len];
                        String::from_utf8_lossy(&n)
                    } else {
                        String::from_utf8_lossy(&[])
                    };
                    let body = if body_len > 0 {
                        let b = &chunk[14 + name_len..14 + name_len + body_len];
                        String::from_utf8_lossy(&b)
                    } else {
                        String::from_utf8_lossy(&[])
                    };
                    if u > 0 {
                        println!("Macro: {:>4} {}\n{}", i, name, body);
                    }
                }
                "VidM" => {
                    let m = chunk[6];
                    let n = match m {
                        27 => "1080p60".to_string(),
                        o => format!("unknown {}", o),
                    };
                    println!("Video Mode: {} -> {}", m, n);
                }
                "ColV" => {
                    let i = chunk[6];
                    let h = word_at(&chunk, 8);
                    let s = word_at(&chunk, 10);
                    let l = word_at(&chunk, 12);

                    let h = (h as f32) / 10.0;
                    let s = (s as f32) / 1000.0;
                    let l = (l as f32) / 1000.0;

                    println!("Col Gen: {} -> {}/{}/{}", i, h, s, l);
                }
                "MRPr" => {
                    let s = chunk[6];
                    let r = chunk[7];
                    let m = chunk[9];
                    println!("Macro Running: {} -> {}/{}", m, s, r);
                }
                o => {
                    if IGNORED_CHUNKS.contains(&o) {
                        //		    			print!("{} ", &o);
                    } else {
                        println!("Unhandled chunk type: {:?}", o);
                    }
                }
            }
            o += size as usize;
        }

        Some(p)
    }

    pub fn set_len(&mut self, len: u16) {
        //		if self.buffer.capacity() < len {
        self.buffer.resize(len as usize, 0);
        //		}
    }
    pub fn set(&mut self, index: usize, value: u8) {
        self.buffer[index] = value;
        self.dirty = true
    }

    pub fn update_buffer(&mut self) {
        self.dirty = false;
    }

    pub fn buffer(&self) -> &[u8] {
        if self.dirty {
            panic!("Tried to use dirty buffer");
        }
        &self.buffer
    }
}

#[derive(Debug)]
pub struct AtemCommand {
    header: AtemCommandHeader,
    payload: AtemCommandPayload,
    cmd: Option<[u8; 4]>,
    buffer: Vec<u8>,
    dirty: bool,
    fill_hack: bool,
}

impl Default for AtemCommand {
    fn default() -> Self {
        Self {
            header: AtemCommandHeader::default(),
            payload: AtemCommandPayload::default(),
            cmd: None,
            buffer: Vec::new(),
            dirty: false,
            fill_hack: false,
        }
    }
}

impl AtemCommand {
    pub fn from_buffer(buffer: &[u8]) -> Option<AtemCommand> {
        if buffer.len() < 12 {
            None
        } else {
            let (bh, bp) = buffer.split_at(SIZE_OF_HEADER);
            let h = AtemCommandHeader::from_buffer(&bh.try_into().ok()?);

            let is_hello = if let Some(h) = &h { h.is_hello() } else { true };
            let p = if is_hello {
                None
            } else {
                AtemCommandPayload::from_buffer(&bp)
            };

            if let Some(h) = h {
                let plen = h.len() as usize + SIZE_OF_HEADER;
                if false && buffer.len() != plen {
                    println!(
                        "Expected length: {}/{:#04x} != {}/{:#04x} :Actual length",
                        plen,
                        plen,
                        buffer.len(),
                        buffer.len()
                    );
                    dbg!(&h);
                    //					dbg!(&p);
                    todo!("wrong length");
                    None
                } else {
                    if h.is_hello() {
                        println!("Got HELLO. Length {}", plen);
                        let mut ac = AtemCommand::default();
                        ac.header = h;
                        if let Some(p) = p {
                            ac.payload = p;
                        }
                        Some(ac)
                    } else if h.is_ack_request() {
                        //						println!("Got ACK_REQUEST. Length {}", plen);
                        let mut ac = AtemCommand::default();
                        ac.header = h;
                        if let Some(p) = p {
                            ac.payload = p;
                        }
                        Some(ac)
                    } else if h.is_ack() {
                        println!("Got ACK. Length {}", plen);
                        let mut ac = AtemCommand::default();
                        ac.header = h;
                        if let Some(p) = p {
                            ac.payload = p;
                        }
                        Some(ac)
                    } else if h.is_request_next() {
                        println!("Got REQUEST_NEXT. Length {}", plen);
                        let mut ac = AtemCommand::default();
                        ac.header = h;
                        if let Some(p) = p {
                            ac.payload = p;
                        }
                        Some(ac)
                    } else {
                        println!("Unhandled command {:#02x}", h.cmd());
                        None
                    }
                }
            } else {
                panic!("Error parsing header");
            }
        }
    }

    pub fn header(&self) -> &AtemCommandHeader {
        &self.header
    }

    pub fn payload(&mut self) -> &mut AtemCommandPayload {
        &mut self.payload
    }

    fn set_payload_len(&mut self, len: u16) {
        self.header.set_len(len);
        self.payload.set_len(len);
    }
    pub fn create_hello() -> AtemCommand {
        let mut s = Self::default();
        s.set_payload_len(8);
        //		s.header.set_len( 8 );	// payload size
        s.header.set_command(COMMAND_MASK_HELLO);
        s.header.set(9, 0x3a);
        s.payload.set(0 /*12*/, 0x01);

        s.update_buffer();
        s
    }
    pub fn create_ack(package_id: u16, session_id: u16, ack_id: u16) -> AtemCommand {
        let mut s = Self::default();
        s.set_payload_len(0);
        //		s.header.set_len( 8 );	// payload size
        s.header.set_command(COMMAND_MASK_ACK);
        s.header.set_package_id(package_id);
        s.header.set_session_id(session_id);
        s.header.set_ack_id(ack_id);
        s.header.set(9, 0x03);
        //		s.header.set(  9, 0x3a );
        //		s.payload.set( 0 /*12*/, 0x01 );

        s.update_buffer();
        s
    }

    pub fn create_command(
        package_id: u16,
        session_id: u16,
        cmd: &[u8; 4],
        len: u16,
    ) -> AtemCommand {
        let mut s = Self::default();
        //		s.set_payload_len( len + cmd.len() as u16 + 4 );
        s.set_payload_len(len + 4 + 4);
        s.header.set_command(COMMAND_MASK_ACK_REQUEST);
        s.header.set_package_id(package_id);
        s.header.set_session_id(session_id);
        s.cmd = Some(*cmd);
        s.fill_hack = false;
        s
    }
    // [16, 20, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]
    // [16, 20, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 8, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]
    // [16, 20, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]

    // [8, 24, 129, 45, 0, 0, 0, 0, 0, 0, 0, 49,/*<h p>*/ 0, 12, 0, 0, 77, 65, 99, 116, 0, 0, 0, 0]
    // [8, 24, 129, 55, 0, 0, 0, 0, 0, 0, 0, 53,          0, 12, 0, 0, 77, 65, 99, 116, 0, 1, 0, 0]
    pub fn update_buffer(&mut self) {
        self.header.update_buffer();
        self.payload.update_buffer();
        let s = self.header.len();
        /*
        let m = if self.fill_hack {
            [ ( s>>8 ) as u8, ( s & 0xff ) as u8, 0, 0 ].to_vec()
        } else {
            [].to_vec()
        };
        */
        let mut l = self.payload().buffer().len();

        let cmd = if let Some(cmd) = &self.cmd {
            l -= 8;
            [[(s >> 8) as u8, (s & 0xff) as u8, 0, 0], *cmd]
                .concat()
                .to_vec()
        } else {
            [].to_vec()
        };
        let b = [
            self.header.buffer(),
            /*&m,*/ &cmd,
            &self.payload.buffer()[..l],
        ]
        .concat();
        self.buffer = b;
        self.dirty = false;
    }

    pub fn buffer(&self) -> &[u8] {
        if self.dirty {
            panic!("Tried to use dirty buffer");
        }
        &self.buffer
    }
}

#[derive(Debug)]
pub enum Command {
    Hello,
    Ack(u16, u16),
    AtemCommand(AtemCommand),
    RunMacro(u8),
    Shutdown,
}
