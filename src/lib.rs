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
    fn rx_callback(&mut self, packet: MetroPacket);
    fn send(&self, packet: MetroPacket);
}

struct Metro<'a, T: MetroTcvr> {
    apps: [Option<&'a mut dyn MetroApp>; 256],
    tcvr: T,
}

impl<'a, T: MetroTcvr> Metro<'a, T> {
    fn new(tcvr: T) -> Metro<'a, T> {
        let apps = core::array::from_fn(|_| None);
        Metro { apps, tcvr }
    }

    fn bind(&mut self, port: usize, app: &'a mut dyn MetroApp) {
        self.apps[port] = Some(app);
    }

    fn process(&mut self) {
        if let Some(packet) = self.tcvr.recv() {
            if let Some(app) = &mut self.apps[packet.port as usize] {
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
                packet_fifo_tx: pf_tx.clone(),
                packet_fifo_rx: pf_rx.clone(),
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
        fn rx_callback(&mut self, packet: MetroPacket) {
            self.packet_rx_count += 1;
            self.last_rx_packet = Some(packet.clone());
        }

        fn send(&self, packet: MetroPacket) {
            self.metro_inst.send(packet);
        }
    }

    #[test]
    fn test_send() {
        let packet_fifo_aobi: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());
        let packet_fifo_aibo: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());

        let tcvr_a = TestMetroTcvr::new(packet_fifo_aobi.clone(), packet_fifo_aibo.clone());
        let tcvr_b = TestMetroTcvr::new(packet_fifo_aibo.clone(), packet_fifo_aobi.clone());

        let mut metro_a = Metro::new(tcvr_a);
        let mut metro_b = Metro::new(tcvr_b);

        let mut test_metro_app_a = TestMetroApp::new(&metro_a);
        let mut test_metro_app_b = TestMetroApp::new(&metro_b);

        metro_a.bind(0, &mut test_metro_app_a);
        metro_b.bind(0, &mut test_metro_app_b);
    }
}
