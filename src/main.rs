use pnet::packet::ethernet::{EtherType, EtherTypes};
use pnet::packet::icmp::echo_reply::MutableEchoReplyPacket;
use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::icmpv6::{Icmpv6Types, MutableIcmpv6Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::MutablePacket;

use clap::Parser;

use ping_adjuster::{
    modify_icmp_payload, ConstantTimevalAdder, TimevalAdder,
    BannerTimevalAdder,
};

const DEFAULT_QUEUE_NUM: u16 = 5256;

fn handle_ipv4<T: TimevalAdder + ?Sized>(
    ipv4: &mut MutableIpv4Packet,
    f: &mut T,
) -> Result<(), ()> {
    if ipv4.get_next_level_protocol() != IpNextHeaderProtocols::Icmp {
        return Err(());
    }
    let destination = ipv4.get_destination();
    let icmp_packet = match MutableIcmpPacket::new(ipv4.payload_mut()) {
        Some(p) => p,
        None => return Err(()),
    };
    if icmp_packet.get_icmp_type() != IcmpTypes::EchoReply {
        return Err(());
    }
    let mut icmp_echo_reply_packet = match MutableEchoReplyPacket::new(ipv4.payload_mut()) {
        Some(p) => p,
        _ => return Err(()),
    };
    log::trace!("{} -> {:?}", destination, icmp_echo_reply_packet);
    let seq = icmp_echo_reply_packet.get_sequence_number();
    icmp_echo_reply_packet.set_sequence_number(1);
    match modify_icmp_payload(icmp_echo_reply_packet.payload_mut(), seq, f) {
        Ok(_) => {
            use pnet::packet::icmp::checksum;
            let mut icmp_packet =
                MutableIcmpPacket::new(icmp_echo_reply_packet.packet_mut()).unwrap();
            icmp_packet.set_checksum(checksum(&icmp_packet.to_immutable()));
            Ok(())
        }
        Err(_) => Err(()),
    }
}

fn handle_ipv6<T: TimevalAdder + ?Sized>(
    ipv6: &mut MutableIpv6Packet,
    f: &mut T,
) -> Result<(), ()> {
    if ipv6.get_next_header() != IpNextHeaderProtocols::Icmpv6 {
        return Err(());
    }
    let source = ipv6.get_source();
    let destination = ipv6.get_destination();
    let icmp_packet = match MutableIcmpv6Packet::new(ipv6.payload_mut()) {
        Some(p) => p,
        None => return Err(()),
    };
    if icmp_packet.get_icmpv6_type() != Icmpv6Types::EchoReply {
        return Err(());
    }
    let mut icmp_echo_reply_packet = match MutableEchoReplyPacket::new(ipv6.payload_mut()) {
        Some(p) => p,
        None => return Err(()),
    };
    log::trace!("{} -> {:?}", destination, icmp_echo_reply_packet);
    let seq = icmp_echo_reply_packet.get_sequence_number();
    icmp_echo_reply_packet.set_sequence_number(1);
    match modify_icmp_payload(icmp_echo_reply_packet.payload_mut(), seq, f) {
        Ok(_) => {
            use pnet::packet::icmpv6::checksum;
            let mut icmp_packet =
                MutableIcmpv6Packet::new(icmp_echo_reply_packet.packet_mut()).unwrap();
            icmp_packet.set_checksum(checksum(&icmp_packet.to_immutable(), &source, &destination));
            Ok(())
        }
        Err(_) => Err(()),
    }
}

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
        msg.set_verdict(nfq::Verdict::Accept);
        match EtherType::new(msg.get_hw_protocol()) {
            EtherTypes::Ipv4 => {
                if let Some(mut packet) = MutableIpv4Packet::new(msg.get_payload_mut()) {
                    let _ = handle_ipv4(&mut packet, &mut *latency_calculator);
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(mut packet) = MutableIpv6Packet::new(msg.get_payload_mut()) {
                    let _ = handle_ipv6(&mut packet, &mut *latency_calculator);
                }
            }
            _ => (),
        }
        queue.verdict(msg)?;
    }
}
