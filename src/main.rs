use bacnet::application::*;
use bacnet::network::*;
use bacnet::transport::bacnetip::*;
use bacnet::{Decode, Encode};

use async_std::net::UdpSocket;
use async_std::task;

use tracing::trace;

fn main() {
    tracing_subscriber::fmt::init();

    task::block_on(async {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))
            .await
            .unwrap();
        socket.set_broadcast(true).unwrap();
        let mut buf = vec![0u8; 1024];

        println!("Listening on {}", socket.local_addr().unwrap());

        let addr = format!("192.168.69.255:{}", 0xBAC0);
        let data_ref = hex::decode("810b000c0120ffff00ff1008").unwrap(); // Who-is
        let apdu = APDU::new(0x01, 0x08, vec![]);
        trace!("APDU Len: {}", apdu.len());
        let dest = NPDUDest::new(0xffff, 0);
        let npdu = NPDU::new(apdu, Some(dest), None, NPDUPriority::Normal);
        let bvlc = BVLC::new(BVLCFunction::OriginalBroadcastNPDU(npdu));
        let data = bvlc.encode_vec().unwrap();
        println!("Who-Is: {:?}", bvlc);
        println!("Send: {:02x?}", data.to_vec());
        println!("Ref : {:02x?}", data_ref);
        let sent = socket.send_to(&data, &addr).await.unwrap();
        println!("Sent {} bytes to {}", sent, addr);

        loop {
            let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
            // === Data Structure ===
            trace!("Data: {:02x?}", data);

            let b = BVLC::decode_slice(&data).unwrap();
            trace!("BVLC: {:02x?}", b);
            trace!("Function: {:02x?}", b.function);
            trace!("Length: {:?}", b.len());

            match b.function {
                BVLCFunction::OriginalBroadcastNPDU(n) | BVLCFunction::OriginalUnicastNPDU(n) => {
                    trace!("NPDU: {:02x?}", n);
                    trace!("Version: {}", n.version);
                    trace!("Priority: {:?}", n.priority);
                    match n.content {
                        NPDUContent::APDU(apdu) => {
                            trace!("APDU: {:02x?}", apdu);
                            match apdu.service_choice {
                                08 => {
                                    trace!("Who-Is received!");
                                    //let apdu = APDU::new();
                                    //let sent = socket.send_to().await.unwrap();
                                }
                                00 => {
                                    trace!("I-Am received!");
                                }
                                _ => unimplemented!(),
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            }
        }
    });
}
