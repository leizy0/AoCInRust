use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    env,
    fmt::Display,
    io,
    rc::Rc,
};

use crate::int_code::{
    com::SeqIntCodeComputer,
    io::{InputPort, OutputPort, SeqIODevice},
};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    WrongNumberOfArgs(usize, usize),
    ExecutionError(crate::Error),
    SendDataBeforeWorking(i64),
    InvalidSendAddr(i64),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::WrongNumberOfArgs(real_n, expect_n) => write!(
                f,
                "Given wrong number({}) of arguemnts, expect {}",
                real_n, expect_n
            ),
            Error::ExecutionError(ee) => {
                write!(f, "Get error({}) in execution of given intcode program", ee)
            }
            Error::SendDataBeforeWorking(v) => write!(
                f,
                "Try to send data({}) before network interface card is working",
                v
            ),
            Error::InvalidSendAddr(addr) => {
                write!(f, "Try to send packet to invalid address({})", addr)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    from_addr: usize,
    to_addr: usize,
    x: i64,
    y: i64,
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(from: {}, to: {}, x: {}, y: {})",
            self.from(),
            self.to(),
            self.x(),
            self.y()
        )
    }
}

impl Packet {
    pub fn from(&self) -> usize {
        self.from_addr
    }

    pub fn to(&self) -> usize {
        self.to_addr
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }
}

#[derive(Debug)]
pub struct NetworkNAT {
    addr: usize,
    send_addr: usize,
    recv_pac: Option<Packet>,
    sent_pacs: Vec<Packet>,
}

impl NetworkNAT {
    pub fn new(addr: usize, send_addr: usize) -> Self {
        Self {
            addr,
            send_addr,
            recv_pac: None,
            sent_pacs: Vec::new(),
        }
    }

    pub fn sent_pacs(&self) -> &[Packet] {
        &self.sent_pacs
    }

    pub fn addr(&self) -> usize {
        self.addr
    }

    pub fn recv(&mut self, mut pac: Packet) {
        println!("NAT receive {}", pac);
        pac.from_addr = self.addr;
        pac.to_addr = self.send_addr;
        self.recv_pac = Some(pac);
    }

    pub fn send(&mut self) -> Option<Packet> {
        if let Some(pac) = self.recv_pac.take() {
            println!("NAT send {}.", pac);
            self.sent_pacs.push(pac.clone());
            Some(pac)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct NetworkHub {
    ports: HashMap<usize, Port>,
    card_addrs: HashSet<usize>,
    nat_op: Option<NetworkNAT>,
}

impl NetworkHub {
    pub fn new() -> Self {
        Self {
            ports: HashMap::new(),
            card_addrs: HashSet::new(),
            nat_op: None,
        }
    }

    pub fn with_nat(nat_addr: usize, nat_send_addr: usize) -> Self {
        let mut hub = Self::new();
        hub.nat_op = Some(NetworkNAT::new(nat_addr, nat_send_addr));

        hub
    }

    pub fn nat(&self) -> Option<&NetworkNAT> {
        self.nat_op.as_ref()
    }

    pub fn recv_pac_log(&self, to_addr: usize, ind: usize) -> Option<Packet> {
        self.ports
            .get(&to_addr)
            .and_then(|pb| pb.packets.get(ind).cloned())
    }

    pub fn is_idle(&self) -> bool {
        self.card_addrs
            .iter()
            .all(|addr| self.ports.get(addr).unwrap().is_waiting())
    }

    pub fn is_empty(&self) -> bool {
        self.ports.is_empty()
    }

    pub fn connect(&mut self, card: &NICard) {
        let addr = card.addr();
        if !self.card_addrs.insert(addr) {
            // Connect to a card twice.
            panic!("Connect to card with address {} again.", addr);
        } else if self.nat_op.as_ref().is_some_and(|nat| nat.addr() == addr) {
            panic!("Connect a card to a port(with address {}) for NAT.", addr);
        }

        self.ports.entry(addr).or_insert_with(|| Port::new());
    }

    pub fn send(&mut self, packet: Packet) {
        println!("Send: {}", packet);
        if let Some(nat) = self.nat_op.as_mut() {
            if packet.to() == nat.addr() {
                nat.recv(packet);
                return;
            }
        }

        self.ports
            .entry(packet.to())
            .or_insert_with(|| Port::new())
            .send(packet)
    }

    pub fn recv(&mut self, addr: usize) -> Option<Packet> {
        if !self.is_empty() && self.is_idle() {
            if let Some(pac) = self.nat_op.as_mut().and_then(|nat| nat.send()) {
                self.send(pac);
            }
        }

        self.ports
            .get_mut(&addr)
            .and_then(|pb| pb.recv())
            .inspect(|p| println!("Receive: {}", p))
    }
}

#[derive(Debug)]
struct Port {
    unread_ind: usize,
    packets: Vec<Packet>,
    is_waiting: bool,
}

impl Port {
    pub fn new() -> Self {
        Self {
            unread_ind: 0,
            packets: Vec::new(),
            is_waiting: false,
        }
    }

    pub fn send(&mut self, packet: Packet) {
        self.packets.push(packet)
    }

    pub fn recv(&mut self) -> Option<Packet> {
        if let Some(pac) = self.packets.get(self.unread_ind).cloned() {
            self.unread_ind += 1;
            self.is_waiting = false;
            Some(pac)
        } else {
            self.is_waiting = true;
            None
        }
    }

    pub fn is_waiting(&self) -> bool {
        self.is_waiting && self.packets.get(self.unread_ind).is_none()
    }
}

type NetworkHubRef = Rc<RefCell<NetworkHub>>;

enum NICardState {
    InitAddr,
    Working,
}

enum NICardRecvState {
    Idle,
    RecvY(i64),
}

enum PacketAsmState {
    Idle,
    WaitX,
    WaitY,
}

struct PacketAssembler {
    state: PacketAsmState,
    imm_packet: Packet,
}

impl PacketAssembler {
    pub fn new(addr: usize) -> Self {
        Self {
            state: PacketAsmState::Idle,
            imm_packet: Packet {
                from_addr: addr,
                to_addr: 0,
                x: 0,
                y: 0,
            },
        }
    }

    pub fn assemble(&mut self, value: i64) -> Result<Option<Packet>, Error> {
        match self.state {
            PacketAsmState::Idle => {
                let to_addr = usize::try_from(value).map_err(|_| Error::InvalidSendAddr(value))?;
                self.imm_packet.to_addr = to_addr;
                self.state = PacketAsmState::WaitX;
                Ok(None)
            }
            PacketAsmState::WaitX => {
                self.imm_packet.x = value;
                self.state = PacketAsmState::WaitY;
                Ok(None)
            }
            PacketAsmState::WaitY => {
                self.imm_packet.y = value;
                self.state = PacketAsmState::Idle;
                Ok(Some(self.imm_packet.clone()))
            }
        }
    }
}

pub struct NICard {
    addr: usize,
    hub: NetworkHubRef,
    pac_asm: PacketAssembler,
    state: NICardState,
    recv_state: NICardRecvState,
}

impl NICard {
    pub fn new(addr: usize, hub: NetworkHubRef) -> Self {
        Self {
            addr,
            hub,
            pac_asm: PacketAssembler::new(addr),
            state: NICardState::InitAddr,
            recv_state: NICardRecvState::Idle,
        }
    }

    pub fn addr(&self) -> usize {
        self.addr
    }
}

impl InputPort for NICard {
    fn get(&mut self) -> Option<i64> {
        match self.state {
            NICardState::InitAddr => {
                self.state = NICardState::Working;
                Some(i64::try_from(self.addr).unwrap())
            }
            NICardState::Working => match self.recv_state {
                NICardRecvState::Idle => {
                    if let Some(packet) = self.hub.borrow_mut().recv(self.addr) {
                        self.recv_state = NICardRecvState::RecvY(packet.y());
                        Some(packet.x())
                    } else {
                        // println!("Host @ {} is waiting for packets.", self.addr);
                        Some(-1)
                    }
                }
                NICardRecvState::RecvY(y) => {
                    self.recv_state = NICardRecvState::Idle;
                    Some(y)
                }
            },
        }
    }

    fn reg_proc(&mut self, _proc_id: usize) {}
}

impl OutputPort for NICard {
    fn put(&mut self, value: i64) -> Result<(), crate::Error> {
        match self.state {
            NICardState::InitAddr => Err(crate::Error::IOProcessError(
                Error::SendDataBeforeWorking(value).to_string(),
            )),
            NICardState::Working => {
                if let Some(packet) = self
                    .pac_asm
                    .assemble(value)
                    .map_err(|e| crate::Error::IOProcessError(e.to_string()))?
                {
                    self.hub.borrow_mut().send(packet)
                }

                Ok(())
            }
        }
    }

    fn wait_proc_id(&self) -> Option<usize> {
        None
    }
}

pub fn check_args() -> Result<String, Error> {
    let args = env::args();
    let args_n = args.len();
    if args_n != 2 {
        Err(Error::WrongNumberOfArgs(args_n, 2))
    } else {
        Ok(args.skip(1).next().unwrap().to_string())
    }
}

pub fn run_network(host_n: usize, intcode: &Vec<i64>) -> Result<NetworkHub, Error> {
    let mut computer = SeqIntCodeComputer::new(false);
    let mut proc_ids = Vec::new();
    let hub = Rc::new(RefCell::new(NetworkHub::new()));
    for i in 0..host_n {
        let card = NICard::new(i, hub.clone());
        hub.borrow_mut().connect(&card);
        let io_dev = SeqIODevice::new(card);
        let cur_proc_id =
            computer.new_proc(&intcode, io_dev.input_device(), io_dev.output_device());
        proc_ids.push(cur_proc_id);
    }

    computer
        .exe_procs_pmp_cond(&proc_ids, proc_ids[0], 100, || {
            !hub.borrow().is_empty() && hub.borrow().is_idle()
        })
        .map(|_| Rc::try_unwrap(hub).unwrap().into_inner())
        .map_err(Error::ExecutionError)
}

pub fn run_network_nat(
    host_n: usize,
    intcode: &Vec<i64>,
    nat_addr: usize,
    nat_send_addr: usize,
) -> Result<NetworkHub, Error> {
    let mut computer = SeqIntCodeComputer::new(false);
    let mut proc_ids = Vec::new();
    let hub = Rc::new(RefCell::new(NetworkHub::with_nat(nat_addr, nat_send_addr)));
    for i in 0..host_n {
        let card = NICard::new(i, hub.clone());
        hub.borrow_mut().connect(&card);
        let io_dev = SeqIODevice::new(card);
        let cur_proc_id =
            computer.new_proc(&intcode, io_dev.input_device(), io_dev.output_device());
        proc_ids.push(cur_proc_id);
    }

    computer
        .exe_procs_pmp_cond(&proc_ids, proc_ids[0], 100, || {
            let hub = hub.borrow();
            let sent_pacs = hub.nat().unwrap().sent_pacs();
            let mut rev_pac_iter = sent_pacs.iter().rev();
            rev_pac_iter
                .next()
                .is_some_and(|p0| rev_pac_iter.next().is_some_and(|p1| p0.y() == p1.y()))
        })
        .map(|_| Rc::try_unwrap(hub).unwrap().into_inner())
        .map_err(Error::ExecutionError)
}
