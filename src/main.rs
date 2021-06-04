#[macro_use]
extern crate clap;

use std::str::FromStr;

use pnet::packet::ethernet::{EtherType, EtherTypes};
use pnet::packet::icmp::echo_reply::MutableEchoReplyPacket;
use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::icmpv6::{Icmpv6Types, MutableIcmpv6Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::MutablePacket;

use ping_fuckuper::fuckup_icmp_payload_buffer;

const DEFAULT_QUEUE_NUM: u16 = 5256;

fn handle_ipv4(ipv4: &mut MutableIpv4Packet) -> Result<(), ()> {
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
    match fuckup_icmp_payload_buffer(icmp_echo_reply_packet.payload_mut()) {
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

fn handle_ipv6(ipv6: &mut MutableIpv6Packet) -> Result<(), ()> {
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
    match fuckup_icmp_payload_buffer(icmp_echo_reply_packet.payload_mut()) {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = clap_app!(ping_fuckuper =>
        (@arg queue_num: --queue-num +takes_value)
    ).get_matches();

    let queue_num = match matches.value_of("queue_num") {
        Some(val) => {
            u16::from_str(val)?
        },
        None => {
            DEFAULT_QUEUE_NUM
        },
    };

    let mut queue = nfq::Queue::open()?;
    queue.bind(queue_num)?;
    log::info!("bound to queue {}, entering main loop", queue_num);
    loop {
        let mut msg = queue.recv()?;
        msg.set_verdict(nfq::Verdict::Accept);
        match EtherType::new(msg.get_hw_protocol()) {
            EtherTypes::Ipv4 => {
                if let Some(mut packet) = MutableIpv4Packet::new(msg.get_payload_mut()) {
                    let _ = handle_ipv4(&mut packet);
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(mut packet) = MutableIpv6Packet::new(msg.get_payload_mut()) {
                    let _ = handle_ipv6(&mut packet);
                }
            }
            _ => (),
        }
        queue.verdict(msg)?;
    }

    Ok(())
}
