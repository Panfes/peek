#![allow(unused)]
use pnet_packet::ip::IpNextHeaderProtocols;
use pnet_packet::ipv4::{MutableIpv4Packet, Ipv4Packet};
use pnet_packet::tcp::{MutableTcpPacket, TcpPacket, TcpFlags};
use pnet_packet::util::checksum;
use std::net::{Ipv4Addr, SocketAddrV4};

const IP_HEADER_LENGTH: usize = 20;
const TCP_HEADER_LENGTH: usize = 20;
const TOTAL_LENGTH: usize = 40;

fn fill_ipv4_header(header: &mut [u8], source: Ipv4Addr, destination: Ipv4Addr) {
    {
        let Some(mut ip_header) = MutableIpv4Packet::new(header) else {
            eprintln!("Не удалось создать MutableIpv4Packet для IP заголовка в пакете");
            return;
        };

        ip_header.set_version(4);
        ip_header.set_header_length(5);
        ip_header.set_total_length(TOTAL_LENGTH as u16);
        ip_header.set_ttl(64);
        ip_header.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
        ip_header.set_source(source);
        ip_header.set_destination(destination);
        ip_header.set_checksum(0);
    }

    let ip_checksum = checksum(header, 0);

    {
        let Some(mut ip_header) = MutableIpv4Packet::new(header) else {
            eprintln!("Не удалось снова создать MutableIpv4Packet");
            return;
        };

        ip_header.set_checksum(ip_checksum);
    }
}

fn fill_tcp_header(
        header: &mut [u8],
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        source_port: u16,
        destination_port: u16,
        sequence_number: u32
    ) {

    {
        let Some(mut tcp_header) = MutableTcpPacket::new(header) else {
            eprintln!("Не удалось создать MutableTcpPacket, для TCP заголовка");
            return;
        };

        tcp_header.set_source(source_port);
        tcp_header.set_destination(destination_port);
        tcp_header.set_sequence(sequence_number);
        tcp_header.set_acknowledgement(0);
        tcp_header.set_flags(TcpFlags::SYN);
        tcp_header.set_data_offset(5);
        tcp_header.set_urgent_ptr(0);
        tcp_header.set_window(64240);
        tcp_header.set_checksum(0);
    }

    let checksum = pnet_packet::util::ipv4_checksum(
            header,
            8,
            &[],
            &source_ip,
            &destination_ip,
            IpNextHeaderProtocols::Tcp,
        );

    {
        let Some(mut tcp_header) = MutableTcpPacket::new(header) else {
            eprintln!("Не удалось создать MutableTcpPacket, при установке финальной checksum");
            return;
        };

        tcp_header.set_checksum(checksum);
    }
}

pub fn create_syn_packet(
    buffer: &mut [u8],
    source: SocketAddrV4,
    destination: SocketAddrV4,
    sequence_number: u32,
) {
    fill_ipv4_header(
        &mut buffer[0..IP_HEADER_LENGTH],
        *source.ip(),
        *destination.ip()
    );

    fill_tcp_header(
        &mut buffer[IP_HEADER_LENGTH..TOTAL_LENGTH],
        *source.ip(),
        *destination.ip(),
        source.port(),
        destination.port(),
        sequence_number,
    );
}

pub fn parse_packet<'a>(buf: &'a [u8]) -> Option<(Ipv4Packet<'a>, TcpPacket<'a>)> {
    let ip = Ipv4Packet::new(buf)?;

    let header_len = (ip.get_header_length() * 4) as usize;

    let tcp = TcpPacket::new(&buf[header_len..])?;

    Some((ip, tcp))
}
