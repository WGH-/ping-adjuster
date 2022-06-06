//! Parsing Ethernet frames to handle ICMP payloads.
use pnet::packet::{
    ethernet::{EtherType, EtherTypes},
    icmp::{echo_reply::MutableEchoReplyPacket, IcmpTypes, MutableIcmpPacket},
    icmpv6::{Icmpv6Types, MutableIcmpv6Packet},
    ip::IpNextHeaderProtocols,
    ipv4::MutableIpv4Packet,
    ipv6::MutableIpv6Packet,
    MutablePacket,
};

fn handle_ipv4<F, E>(ipv4: &mut MutableIpv4Packet, f: F) -> Result<(), E>
where
    F: FnOnce(&mut [u8], u16) -> Result<(), E>,
{
    if ipv4.get_next_level_protocol() != IpNextHeaderProtocols::Icmp {
        return Ok(());
    }
    let destination = ipv4.get_destination();
    let icmp_packet = match MutableIcmpPacket::new(ipv4.payload_mut()) {
        Some(p) => p,
        None => return Ok(()),
    };
    if icmp_packet.get_icmp_type() != IcmpTypes::EchoReply {
        return Ok(());
    }
    let mut icmp_echo_reply_packet = match MutableEchoReplyPacket::new(ipv4.payload_mut()) {
        Some(p) => p,
        _ => return Ok(()),
    };
    log::trace!("{} -> {:?}", destination, icmp_echo_reply_packet);
    let seq = icmp_echo_reply_packet.get_sequence_number();
    icmp_echo_reply_packet.set_sequence_number(1);

    match f(icmp_echo_reply_packet.payload_mut(), seq) {
        Err(e) => Err(e),
        Ok(_) => {
            use pnet::packet::icmp::checksum;
            let mut icmp_packet =
                MutableIcmpPacket::new(icmp_echo_reply_packet.packet_mut()).unwrap();
            icmp_packet.set_checksum(checksum(&icmp_packet.to_immutable()));
            Ok(())
        }
    }
}

fn handle_ipv6<F, E>(ipv6: &mut MutableIpv6Packet, f: F) -> Result<(), E>
where
    F: FnOnce(&mut [u8], u16) -> Result<(), E>,
{
    if ipv6.get_next_header() != IpNextHeaderProtocols::Icmpv6 {
        return Ok(());
    }
    let source = ipv6.get_source();
    let destination = ipv6.get_destination();
    let icmp_packet = match MutableIcmpv6Packet::new(ipv6.payload_mut()) {
        Some(p) => p,
        None => return Ok(()),
    };
    if icmp_packet.get_icmpv6_type() != Icmpv6Types::EchoReply {
        return Ok(());
    }
    let mut icmp_echo_reply_packet = match MutableEchoReplyPacket::new(ipv6.payload_mut()) {
        Some(p) => p,
        None => return Ok(()),
    };
    log::trace!("{} -> {:?}", destination, icmp_echo_reply_packet);
    let seq = icmp_echo_reply_packet.get_sequence_number();
    icmp_echo_reply_packet.set_sequence_number(1);
    match f(icmp_echo_reply_packet.payload_mut(), seq) {
        Ok(_) => {
            use pnet::packet::icmpv6::checksum;
            let mut icmp_packet =
                MutableIcmpv6Packet::new(icmp_echo_reply_packet.packet_mut()).unwrap();
            icmp_packet.set_checksum(checksum(&icmp_packet.to_immutable(), &source, &destination));
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Parses an Ethernet frame with specified `ethertype`, and calls `f` on ICMP
/// Echo Reply payload if it's in there.
///
/// ICMP checksum is automatically recalculated.
///
/// Both IPv4 + ICMP and IPV6 + ICMPv6 are supported.
pub fn modify_icmp_payload<F, E>(ethertype: EtherType, payload: &mut [u8], f: F) -> Result<(), E>
where
    F: FnOnce(&mut [u8], u16) -> Result<(), E>,
{
    match ethertype {
        EtherTypes::Ipv4 => {
            if let Some(mut packet) = MutableIpv4Packet::new(payload) {
                return handle_ipv4(&mut packet, f);
            }
        }
        EtherTypes::Ipv6 => {
            if let Some(mut packet) = MutableIpv6Packet::new(payload) {
                return handle_ipv6(&mut packet, f);
            }
        }
        _ => (),
    }
    Ok(())
}
