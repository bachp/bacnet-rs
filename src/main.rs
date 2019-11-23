use bacnet::application::*;
use bacnet::network::*;
use bacnet::transport::bacnetip::*;
use bacnet::{Decode, Encode};

use async_std::io;
use async_std::net::UdpSocket;
use async_std::task;

use serde::{Deserialize, Serialize};
use serde_asn1_der::{from_bytes, to_vec};
use serde_json::Value;

use bytes::{BufMut, BytesMut};

use tracing::trace;

fn main() {
    tracing_subscriber::fmt::init();

    task::block_on(async {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))
            .await
            .unwrap();
        socket.set_broadcast(true);
        let mut buf = vec![0u8; 1024];

        println!("Listening on {}", socket.local_addr().unwrap());

        let addr = format!("192.168.69.255:{}", 0xBAC0);
        let data_ref = hex::decode("810b000c0120ffff00ff1008").unwrap(); // Who-is
        let apdu = APDU::new(0x01, 0x08, vec![]);
        trace!("APDU Len: {}", apdu.len());
        let dest = Dest::new(0xffff, 0);
        let npdu = NPDU::new(apdu, Some(dest), None, NPDUPriority::Normal);
        let bvlc = BVLC::new(BVLCFunction::OriginalBroadcastNPDU(npdu));
        let mut w = BytesMut::new().writer();
        bvlc.encode(&mut w);
        let data = w.into_inner();
        println!("Who-Is: {:?}", bvlc);
        println!("Send: {:02x?}", data.to_vec());
        println!("Ref : {:02x?}", data_ref);
        let sent = socket.send_to(&data, &addr).await.unwrap();
        println!("Sent {} bytes to {}", sent, addr);

        loop {
            let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
            // === Data Structure ===
            let mut data = std::io::Cursor::new(&buf[..n]);
            trace!("Data: {:02x?}", data);

            let b = BVLC::decode(&mut data).unwrap();
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

            /*
            // === Slice ===
            let data = &buf[..n];
            trace!("Slice Data: {:02x?}", data);

            let b = BVLCSlice::from_slice(&data).unwrap();
            trace!("Slice BVLC: {:02x?}", b);
            trace!("Slice Function: {:02x?}", b.function());
            trace!("Slice Length: {:?}", b.length());
            */
        }

        /*loop {
            let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
            //println!("Buffer: {:02x?}", buf);
            //let sent = socket.send_to(&buf[..n], &peer).await.unwrap();
            //println!("Sent {} out of {} bytes to {}", sent, n, peer);

            //let data = hex::decode("810b000c0120ffff00ff1008000000000000").unwrap(); // Who-is
            //let data = hex::decode("810a001401001000c4020002572204009100210f").unwrap(); // I-am
            let data = &buf[..n];
            trace!("Data: {:02x?}", data);

            //let b = BVLCSlice::from_slice(&data).unwrap();
            let b = BVLC::decode(data.).unwrap();

            println!("Function: {:02x?}", b.function());
            println!("Length: {:?}", b.length());

            let n = b.function().unwrap();
            match n {
                BVLCSliceFunction::OriginalBroadcastNPDU(n)
                | BVLCSliceFunction::OriginalUnicastNPDU(n) => {
                    println!("Version: {:?}", n.version());
                    println!("Priority: {:?}", n.priority());
                    println!("Content: {:02x?}", n.apdu().unwrap().content());
                    match n.apdu().unwrap().content().unwrap() {
                        BACnetPDUSlice::UnconfirmedRequest(c) => {
                            println!("{:02x?}", c.service());
                            match c.service().unwrap() {
                                UnconfirmedService::WhoIs() => {
                                    println!("Who is received");
                                    let mut data = BytesMut::new().writer();
                                    BVLC::new(BVLCFunction::OriginalUnicastNPDU(NPDU::new(
                                        NPDUContent::APDU(APDU::new()),
                                        None,
                                        None,
                                        NPDUPriority::Normal,
                                    )))
                                    .encode(&mut data)
                                    .expect("Unable to write I Am");

                                    println!("Data1: {:02x?}", data);
                                    let data =
                                        hex::decode("810a001401001000c4020002572204009100210f")
                                            .unwrap(); // I-am
                                    println!("Data: {:02x?}", data);
                                    let sent = socket.send_to(&data[..], &peer).await.unwrap();
                                    println!("Sent {} bytes to {}", sent, peer);
                                }
                                UnconfirmedService::IAm(iam) => {
                                    println!("IAm: {:02x?}", c);
                                }
                                _ => unimplemented!(),
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            }
        }*/
    });
}
