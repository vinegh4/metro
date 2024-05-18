#![cfg_attr(not(test), no_std)]

struct MetroPacket {
    port: u8,
    data: [u8; 256],
    data_len: u8,
}

impl Clone for MetroPacket {
    fn clone(&self) -> MetroPacket {
        MetroPacket {
            port: self.port,
            data: self.data.clone(),
            data_len: self.data_len,
        }
    }
}

trait MetroApp {
    fn rx_callback(&self, packet: MetroPacket);
    fn send(&self, packe: MetroPacket);
}

struct Metro<'a, T: MetroTcvr> {
    apps: [Option<&'a dyn MetroApp>; 256],
    tcvr: &'a T,
}

impl<'a, T: MetroTcvr> Metro<'a, T> {
    fn new(tcvr: &'a T) -> Metro<'a, T> {
        let apps = [None; 256];
        Metro { apps, tcvr }
    }

    fn process(&self) {
        if let Some(packet) = self.tcvr.recv() {
            if let Some(app) = self.apps[packet.port as usize] {
                app.rx_callback(packet);
            }
        }
    }

    fn send(&self, packet: MetroPacket) {
        self.tcvr.send(packet);
    }
}

trait MetroTcvr {
    fn send(&self, packet: MetroPacket);
    fn recv(&self) -> Option<MetroPacket>;
}

//trait MetroApp  {}
//
//struct Tube<T : MetroTcvr> {
//    tcvr: T,
//    apps: Vec<Box<dyn MetroApp>>,
//}

#[cfg(test)]
mod tests {

    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct TestMetroTcvr {
        packet_fifo_tx: RefCell<VecDeque<MetroPacket>>,
        packet_fifo_rx: RefCell<VecDeque<MetroPacket>>,
    }

    impl TestMetroTcvr {
        fn new(
            pf_tx: RefCell<VecDeque<MetroPacket>>,
            pf_rx: RefCell<VecDeque<MetroPacket>>,
        ) -> TestMetroTcvr {
            TestMetroTcvr {
                packet_fifo_tx: pf_tx,
                packet_fifo_rx: pf_rx,
            }
        }
    }

    impl MetroTcvr for TestMetroTcvr {
        fn send(&self, packet: MetroPacket) {
            self.packet_fifo_tx.borrow_mut().push_back(packet);
        }

        fn recv(&self) -> Option<MetroPacket> {
            self.packet_fifo_rx.borrow_mut().pop_front()
        }
    }

    struct TestMetroApp<'a> {
        packet_rx_count: usize,
        packet_tx_count: usize,
        last_tx_packet: Option<MetroPacket>,
        last_rx_packet: Option<MetroPacket>,
        metro_inst: &'a Metro<'a, TestMetroTcvr>,
    }

    impl<'a> TestMetroApp<'a> {
        fn new(metro: &'a Metro<'a, TestMetroTcvr>) -> TestMetroApp<'a> {
            TestMetroApp {
                packet_rx_count: 0,
                packet_tx_count: 0,
                last_tx_packet: None,
                last_rx_packet: None,
                metro_inst: metro,
            }
        }
    }

    impl<'a> MetroApp for TestMetroApp<'a> {
        fn rx_callback(&self, packet: MetroPacket) {
            self.packet_rx_count += 1;
            self.last_rx_packet = Some(packet.clone());
        }

        fn send(&self, packet: MetroPacket) {
            self.metro_inst.send(packet);
        }
    }

    fn setup() {
        let packet_fifo_mosi: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());
        let packet_fifo_miso: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());

        let tcvr_master = TestMetroTcvr::new(packet_fifo_mosi, packet_fifo_miso);
        let tcvr_slave = TestMetroTcvr::new(packet_fifo_miso, packet_fifo_mosi);
    }

    #[test]
    fn test_send() {
        let packet_fifo_mosi: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());
        let packet_fifo_miso: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());

        let tcvr_master = TestMetroTcvr::new(packet_fifo_mosi, packet_fifo_miso);
    }
}
