use crate::intcode;
use crate::utils;
use std::collections::VecDeque;
use std::panic;
use std::thread::JoinHandle;
use std::time;
use std::{sync::mpsc, thread};

const NETWORK_SIZE: usize = 50;
const INVALID_NETWORK_ADDRESS: isize = -1;
const NAT_ADDRESS: isize = 255;

//Due to the multi-threaded nature of the solution, we need to tune when the computers consider themselves idle and how
//fast their polling loop is for new messages.
//If they report idle status too quickly, e.g. before messages have had a chance to arrive on their queue, Nat can assume network idle
//prematurely (both the speed of the polling loop and number of loops with no input/output determine this).
//But if status is reported too slowly or the poll interval is too long the solution takes a long time to run.
//Ironically, this would probably have been faster and significantly less flaky if I'd written it single-threadedly. Ah well.
const MAX_IDLE_COUNTER: usize = 3; //Number of worker loops with empty input/output before idle status decided
const STATUS_KICK_TIMER: u128 = 10; //milliseconds between status heartbeats
const POLL_INTERVAL: u64 = 1; //Poll for new messages ever millisecond.

// Represents a computer in the network.
#[derive(Debug)]
struct Computer {
    network_address: isize, //Network address of this computer
    nic: intcode::Program,  //The NIC program - the brain of this computer
    instruction_receiver: mpsc::Receiver<Instruction>, //Receiver for messages from the main thread (the "nat")
    packet_255_sender: mpsc::Sender<Packet>, //Used to send a packet to the nat, addressed by special address 255.
    packet_receiver: mpsc::Receiver<Packet>, //Receiver for messages from other Computers - this acts as an incoming message queue.
    packet_senders: Vec<mpsc::Sender<Packet>>, //Vec of send channels indexed by network address (array would be better but it's a ballache)
    status_sender: mpsc::Sender<StatusMessage>, //Send status - is this computer idle or not
    status: Status,
}

impl Computer {
    fn new(
        nic: intcode::Program,
        instruction_receiver: mpsc::Receiver<Instruction>,
        packet_255_sender: mpsc::Sender<Packet>,
        packet_receiver: mpsc::Receiver<Packet>,
        packet_senders: &[mpsc::Sender<Packet>], //Only pass a reference - the constructor takes care of cloning it.
        status_sender: mpsc::Sender<StatusMessage>,
    ) -> Computer {
        Computer {
            network_address: INVALID_NETWORK_ADDRESS,
            nic,
            instruction_receiver,
            packet_255_sender,
            packet_receiver,
            packet_senders: packet_senders.to_owned(), // This clones both the array and the underlying Senders.
            status_sender,
            status: Status::Active,
        }
    }

    fn run(&mut self) {
        //called from dedicated thread for this computer.
        let mut idle_counter = 0;
        let current_time = time::Instant::now();
        'outer: loop {
            let mut empty_input_queue = false;
            let mut no_outputs = false;
            //Initialize if not already up and going.
            if self.network_address == INVALID_NETWORK_ADDRESS {
                let result = self.nic.run();
                assert_eq!(intcode::ProgramResult::AwaitingInput, result);
                //Block waiting on the instruction to start running
                match self
                    .instruction_receiver
                    .recv()
                    .unwrap_or_else(|err| panic!("Failed to receive instruction!: {err}"))
                {
                    Instruction::Init(network_address) => {
                        self.network_address = network_address;
                        self.nic.add_input(network_address)
                    }
                    Instruction::Shutdown => break 'outer,
                }
            }

            //Prioritize anything coming from the nat - e.g. being asked to shut down
            match self.instruction_receiver.try_recv() {
                Ok(Instruction::Init(_)) => {
                    eprintln!("Unexpected call to init when already initted!")
                }
                Ok(Instruction::Shutdown) => break 'outer,
                Err(_) => (), //no-op
            }

            //See if there are any inbound packets on the queue
            let mut packets = self
                .packet_receiver
                .try_iter()
                .collect::<VecDeque<Packet>>();

            //Do work until we run out of inputs
            if packets.is_empty() {
                //Nothing on queue - pass in -1
                self.nic.add_input(-1);
                empty_input_queue = true;
            } else {
                while !packets.is_empty() {
                    let packet = packets
                        .pop_front()
                        .expect("No packets left on queue - unexpected!");
                    self.nic.add_input(packet.x);
                    self.nic.add_input(packet.y);
                }
            }

            match self.nic.run() {
                intcode::ProgramResult::AwaitingInput => {
                    let mut outputs: VecDeque<isize> = VecDeque::new();

                    if self.nic.outputs().is_empty() {
                        no_outputs = true;
                    } else {
                        while !self.nic.outputs().is_empty() {
                            //nic outputs are a stack, hence the pop front to turn it into a queue.
                            outputs.push_front(
                                self.nic
                                    .remove_last_output()
                                    .expect("Nic outputs unexpectedly empty!"),
                            );
                        }
                    }

                    while !outputs.is_empty() {
                        let address = outputs
                            .pop_front()
                            .expect("Address output unexpectedly absent");
                        let packet = Packet::new(
                            address,
                            outputs.pop_front().expect("X output unexpectedly absent"),
                            outputs.pop_front().expect("Y output unexpectedly absent"),
                        );

                        if packet.address == NAT_ADDRESS {
                            //Packet 255!
                            self.packet_255_sender.send(packet).unwrap_or_else(|err| {
                                eprintln!("Failed to send packet to nat. Packet: {:?}, Channel: {:?}, Error: {}", packet, self.packet_255_sender, err)
                            });
                        } else {
                            self.packet_senders[packet.address as usize]
                                .send(packet)
                                .unwrap_or_else(|err| {
                                    eprintln!("Failed to send packet to computer. Packet: {:?}, Channel: {:?}, Error: {}", packet, self.packet_255_sender, err)
                            });
                        }
                    }
                }
                intcode::ProgramResult::Halted => panic!("Program unexpectedly halted"), //I don't think this should ever happen - always expecting the program to be awaiting input.
            }

            if empty_input_queue && no_outputs {
                //Send idle status if this condition has persisted for a while
                idle_counter += 1;

                if idle_counter > MAX_IDLE_COUNTER && self.status == Status::Active {
                    // Status change - notify
                    self.status = Status::Idle;
                    self.status_sender
                        .send(StatusMessage::new(self.network_address, Status::Idle))
                        .unwrap_or_else(|err| {
                            panic!("Unable to send idle status message: {}", err)
                        });
                }
            } else {
                idle_counter = 0;

                if self.status == Status::Idle {
                    self.status = Status::Active;
                    self.status_sender
                        .send(StatusMessage::new(self.network_address, Status::Active))
                        .unwrap_or_else(|err| {
                            panic!("Unable to send active status message: {}", err)
                        });
                }
            }

            //Every 10ms kick with current status in case stuff gets stuck
            if current_time.elapsed().as_millis() % STATUS_KICK_TIMER == 0 {
                self.status_sender
                    .send(StatusMessage::new(self.network_address, self.status))
                    .unwrap_or_else(|err| panic!("Unable to send current status message: {}", err));
            }

            //Slow things down just a little.
            thread::sleep(time::Duration::from_millis(POLL_INTERVAL));
        }
    }
}

#[derive(Debug)]
struct Nat {
    instruction_senders: Vec<mpsc::Sender<Instruction>>,
    packet_255_receiver: mpsc::Receiver<Packet>,
    status_receiver: mpsc::Receiver<StatusMessage>,
    idle_packet_sender: mpsc::Sender<Packet>,
    network_status: [Status; NETWORK_SIZE],
    last_packet: Option<Packet>,
    last_y_sent: Option<isize>,
}

impl Nat {
    fn new(
        instruction_senders: Vec<mpsc::Sender<Instruction>>,
        packet_255_receiver: mpsc::Receiver<Packet>,
        status_receiver: mpsc::Receiver<StatusMessage>,
        idle_packet_sender: mpsc::Sender<Packet>,
    ) -> Nat {
        Nat {
            instruction_senders,
            packet_255_receiver,
            status_receiver,
            idle_packet_sender,
            network_status: [Status::Active; 50], //Assume active at first
            last_packet: None,
            last_y_sent: None,
        }
    }

    fn start_network(&self) {
        for (network_address, sender) in self.instruction_senders.iter().enumerate() {
            sender
                .send(Instruction::Init(network_address as isize))
                .unwrap_or_else(|err| panic!("Failed to start computer {network_address}: {err}"));
        }
    }

    fn shutdown_network(&self) {
        for sender in &self.instruction_senders {
            sender
                .send(Instruction::Shutdown)
                .unwrap_or_else(|err| panic!("Failed to send instruction: {err}"));
        }
    }

    //Tests to see if any packets received on the queue and stores if so
    fn receive_packet_255(&mut self) -> Option<isize> {
        let mut opt_y = None;
        for packet in self.packet_255_receiver.try_iter() {
            self.last_packet = Some(packet); //overwrites the stored packet
            opt_y = Some(packet.y)
        }
        opt_y
    }

    //Receives status updates from computers (Active if the last status message
    //from any computer is Active; Idle if all computers are idle). Sends to address 0 if idle
    fn receive_status_updates(&mut self) -> Option<isize> {
        for status_message in self.status_receiver.try_iter() {
            self.network_status[status_message.address as usize] = status_message.status;
        }

        if !self
            .network_status
            .iter()
            .any(|status| *status == Status::Active)
        {
            //No computers are active. Overall network status is Idle
            let last_packet = self
                .last_packet
                .expect("Should always have last packet to send when network idle");
            if let Some(last_y) = self.last_y_sent {
                if last_y == last_packet.y {
                    //Sending same y again to computer 0
                    return Some(last_y);
                }
            }
            self.last_y_sent = Some(last_packet.y);
            self.idle_packet_sender
                .send(last_packet)
                .unwrap_or_else(|err| panic!("Failed to send idle packet: {}", err));
            //Reset idle status
            self.network_status = [Status::Active; 50];
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Init(isize), //Used to send the network address for the Nic
    Shutdown,    //Requests that the computer shut down.
}

#[derive(Clone, Copy, Debug)]
struct Packet {
    address: isize,
    x: isize,
    y: isize,
}

impl Packet {
    fn new(address: isize, x: isize, y: isize) -> Packet {
        Packet { address, x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Idle,
    Active,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StatusMessage {
    address: isize,
    status: Status,
}

impl StatusMessage {
    fn new(address: isize, status: Status) -> StatusMessage {
        StatusMessage { address, status }
    }
}

pub fn day23() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day23.txt", ',');
    let program = intcode::Program::new(&initial_state);
    let (computers, mut nat) = construct_network(&program);
    let computer_handles = boot_network(computers);
    nat.start_network();
    let mut part1_option = None;
    let mut part2_option: Option<isize>;

    loop {
        if let Some(y) = nat.receive_packet_255() {
            if part1_option.is_none() {
                part1_option = Some(y);
            }
        }

        part2_option = nat.receive_status_updates();
        if part2_option.is_some() {
            break;
        }

        thread::sleep(time::Duration::from_millis(1));
    }

    nat.shutdown_network();

    //Threads should all shut down.
    for handle in computer_handles {
        handle
            .join()
            .unwrap_or_else(|err| panic::resume_unwind(err));
    }

    (
        part1_option.unwrap() as usize,
        part2_option.unwrap() as usize,
    )
}

//Constructs Computer objects required by network giving each the resources it needs to talk to the others. Also constructs a "nat"
//to be used for sending and receiving messages to/from each of the computers, monitoring queues, etc.
fn construct_network(nic_program: &intcode::Program) -> (Vec<Computer>, Nat) {
    //First construct the nat. We need 2 mpsc channels for sending Instructions and receiving the 255 packet.
    let mut instruction_senders: Vec<mpsc::Sender<Instruction>> = vec![];
    let mut instruction_receivers: VecDeque<mpsc::Receiver<Instruction>> = VecDeque::new();
    let mut packet_senders: Vec<mpsc::Sender<Packet>> = vec![];
    let mut packet_receivers: VecDeque<mpsc::Receiver<Packet>> = VecDeque::new();

    //Set up resources
    for _ in 0..NETWORK_SIZE {
        let (instruction_sender, instruction_receiver) = mpsc::channel::<Instruction>();
        let (packet_sender, packet_receiver) = mpsc::channel::<Packet>();
        instruction_senders.push(instruction_sender);
        instruction_receivers.push_back(instruction_receiver);
        packet_senders.push(packet_sender);
        packet_receivers.push_back(packet_receiver);
    }
    let (packet_255_sender, packet_255_receiver) = mpsc::channel::<Packet>();
    let (status_sender, status_receiver) = mpsc::channel::<StatusMessage>();

    //Create the nat
    let nat = Nat::new(
        instruction_senders,
        packet_255_receiver,
        status_receiver,
        packet_senders[0].clone(),
    );

    //Create the computers
    let mut computers: Vec<Computer> = vec![];

    for _ in 0..NETWORK_SIZE {
        computers.push(Computer::new(
            nic_program.clone(),
            instruction_receivers
                .pop_front()
                .unwrap_or_else(|| panic!("Failed to get instruction receiver from queue")),
            packet_255_sender.clone(),
            packet_receivers
                .pop_front()
                .unwrap_or_else(|| panic!("Failed to get packet receiver from queue")),
            &packet_senders,
            status_sender.clone(),
        ));
    }

    (computers, nat)
}

//Starts the computers running by spawning a thread for each, calling its run method.
//Takes ownership of computers because the mpsc receivers need to be confined to a single
//thread.
fn boot_network(computers: Vec<Computer>) -> Vec<JoinHandle<()>> {
    let mut computer_join_handles = vec![];
    for mut computer in computers {
        computer_join_handles.push(thread::spawn(move || computer.run()));
    }

    computer_join_handles
}
