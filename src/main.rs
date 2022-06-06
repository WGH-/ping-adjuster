use pnet::packet::ethernet::EtherType;

use clap::Parser;

use ping_adjuster::packet::modify_icmp_payload;
use ping_adjuster::timestamp::{
    modify_icmp_timestamp, BannerTimevalAdder, ConstantTimevalAdder, TimevalAdder,
};

const DEFAULT_QUEUE_NUM: u16 = 5256;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long, default_value_t = DEFAULT_QUEUE_NUM)]
    pub queue_num: u16,

    #[clap(long)]
    pub message: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let mut latency_calculator: Box<dyn TimevalAdder> = match cli.message {
        None => Box::new(ConstantTimevalAdder::new(133713371337)),
        Some(message) => Box::new(BannerTimevalAdder::new(&message)?),
    };

    let mut queue = nfq::Queue::open()?;
    queue.bind(cli.queue_num)?;
    log::info!("bound to queue {}, entering main loop", cli.queue_num);
    loop {
        let mut msg = queue.recv()?;
        let ethertype = EtherType::new(msg.get_hw_protocol());

        let _ = modify_icmp_payload(ethertype, msg.get_payload_mut(), |payload, seq| {
            modify_icmp_timestamp(payload, seq, &mut *latency_calculator)
        });
        queue.verdict(msg)?;
    }
}
