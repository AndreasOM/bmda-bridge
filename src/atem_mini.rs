use rand::Rng;

use std::sync::mpsc::{self, channel};

use crate::atem_command::{
    AtemCommand,
    //	AtemResponse,
    Command,
    //	CommandId,
};

use tokio::net::UdpSocket;

const REMOTE_ADDR: &str = "192.168.186.101:9910";

const PACKET_BUFFER_SIZE: usize = 96;

#[derive(Debug, Default)]
struct Connection {
    sock: Option<UdpSocket>,
    session_id: u16,
    localId: u16,
    remoteId: u16,
    package_id: u16,
}

impl Connection {
    pub fn set_session_id(&mut self, session_id: u16) {
        self.session_id = session_id;
    }
    pub fn session_id(&self) -> u16 {
        self.session_id
    }
    pub fn localId(&self) -> u16 {
        self.localId
    }
    pub fn incLocalId(&mut self) -> u16 {
        self.localId += 1;
        self.localId
    }
    pub fn remoteId(&self) -> u16 {
        self.remoteId
    }
}

pub struct AtemMini {
    request_tx: Option<mpsc::Sender<Command>>,
    response_rx: Option<mpsc::Receiver<AtemCommand>>,
    initial_payload_received: bool,
}

impl AtemMini {
    pub fn new() -> Self {
        Self {
            request_tx: None,
            response_rx: None,
            initial_payload_received: false,
        }
    }

    fn run_handler(&mut self) -> anyhow::Result<()> {
        let (request_tx, request_rx) = channel();
        let (response_tx, response_rx) = channel();

        self.request_tx = Some(request_tx);
        self.response_rx = Some(response_rx);

        let handle: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            let mut connection = Connection::default();

            let local_addr = {
                let mut rng = rand::thread_rng();
                let r = rng.gen_range(0..100);
                format!("0.0.0.0:{}", 55555 + r)
            };

            let socket = UdpSocket::bind(&local_addr).await?;
            let remote_addr = REMOTE_ADDR;
            match socket.connect(remote_addr).await {
                Ok(_) => {
                    println!("Connected!");
                }
                Err(e) => {
                    println!("Error connecting: {:?}", &e);
                }
            };

            loop {
                // send outgoing requests
                let r = request_rx.try_recv();
                match r {
                    Ok(cmd) => {
                        match cmd {
                            Command::Hello => {
                                println!("Sending Hello");
                                let c = AtemCommand::create_hello();
                                let buf = c.buffer();
                                println!("{:?}", &buf);
                                let len = socket.send(&buf[..20]).await?;
                                //								println!("{:?} bytes sent", len);
                            }
                            Command::Ack(session_id, remote_id) => {
                                /*
                                let package_id = connection.package_id;
                                connection.package_id += 1;
                                */
                                let package_id = 0;
                                // :HACK
                                connection.session_id = session_id;
                                //								println!("Sending Ack for session {}, remote {}", session_id, remote_id);
                                let c = AtemCommand::create_ack(package_id, session_id, remote_id);
                                let buf = c.buffer();
                                //								println!("{:?}", &buf[..12]);
                                let len = socket.send(&buf[..12]).await?;
                                //								println!("{:?} bytes sent", len);
                            }
                            Command::Shutdown => {
                                return Ok(());
                            }
                            Command::AtemCommand(ac) => {
                                println!("Sending AtemCommand");
                            }
                            Command::RunMacro(index) => {
                                println!(
                                    "Running Macro {} - {} / {}",
                                    index, connection.session_id, connection.package_id
                                );
                                connection.package_id += 1;
                                let package_id = connection.package_id;
                                let session_id = connection.session_id;
                                let mut c =
                                    AtemCommand::create_command(package_id, session_id, b"MAct", 4);
                                c.payload().set(1, index);
                                c.update_buffer();
                                println!("{:?}", &c.buffer());
                                let len = socket.send(&c.buffer()).await?;
                                /*
                                    QByteArray cmd("MAct");
                                    QByteArray payload(4, 0x0);

                                    payload[1] = static_cast<char>(macroIndex);

                                    sendCommand(cmd, payload);
                                */
                            }
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        //						println!("Empty");
                    }
                    Err(e) => {
                        println!("{:?}", &e);
                    }
                }

                // handle incomming responses
                let mut buf = [0; 65535]; //[0;1024];
                match socket.try_recv(&mut buf) {
                    Ok(n) => {
                        if let Some(cmd) = AtemCommand::from_buffer(&buf[..n]) {
                            //							println!("Response: {:?}", &cmd);
                            response_tx.send(cmd);
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
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            println!("???");
            Ok(())
        });

        Ok(())
    }
    pub fn connect(&mut self) -> anyhow::Result<()> {
        self.run_handler();
        if let Some(tx) = &mut self.request_tx {
            let cmd = Command::Hello;
            tx.send(cmd)?;
        }

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        true
    }

    pub fn run_macro(&mut self, index: u8) {
        if let Some(tx) = &mut self.request_tx {
            let cmd = Command::RunMacro(index);
            match tx.send(cmd) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
    pub fn update(&mut self) {
        let max_responses = 10;
        if let Some(response_rx) = &self.response_rx {
            for _i in 0..max_responses {
                let r = response_rx.try_recv();
                match r {
                    Ok(c) => {
                        if !self.initial_payload_received {
                            if c.header().len() == 0 {
                                println!("Initial payload transfered");
                                self.initial_payload_received = true;
                            }
                        }
                        if c.header().is_hello() {
                            println!("Got HELLO ... {}", c.header().session_id());
                            if c.header().is_ack_request() {
                                //								println!("ACK REQUEST!");
                            }
                            if let Some(tx) = &mut self.request_tx {
                                let cmd = Command::Ack(c.header().session_id(), 0);
                                match tx.send(cmd) {
                                    Ok(_) => {}
                                    Err(_) => {}
                                }
                            }
                        } else if c.header().is_ack_request() {
                            //							println!("ACK REQUEST! for {}", c.header().package_id());
                            // :TODO: handle payload
                            if let Some(tx) = &mut self.request_tx {
                                let cmd =
                                    Command::Ack(c.header().session_id(), c.header().package_id());
                                match tx.send(cmd) {
                                    Ok(_) => {}
                                    Err(_) => {}
                                }
                            }
                        } else if c.header().is_ack() {
                            println!("ACK! for {}", c.header().ack_id());
                        } else if c.header().is_request_next() {
                            println!("REQUEST NEXT! for {}", c.header().resend_id());
                        } else {
                            if let Some(tx) = &mut self.request_tx {
                                let cmd = Command::Shutdown;
                                match tx.send(cmd) {
                                    Ok(_) => {}
                                    Err(_) => {}
                                }
                            }
                            dbg!(&c);
                            dbg!(&c.header());
                            todo!("...");
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("Connection lost");
                        break;
                    }
                    Err(e) => {
                        println!("{:?}", &e);
                        break;
                    }
                }
            }
        }
    }
}
