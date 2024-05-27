//#![cfg_attr(not(test), no_std)]

#[derive(Debug)]
struct MetroPacket {
    port: u8,
    data: [u8; 256],
    data_len: u8,
}

impl MetroPacket {
    fn new() -> MetroPacket {
        let data = [0; 256];
        let port = 0;
        let data_len = 0;
        MetroPacket {
            port,
            data,
            data_len,
        }
    }
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

trait MetroApp<T: MetroTcvr> {
    fn rx_callback(&mut self, packet: MetroPacket);
    fn send(&mut self, metro: &Metro<T>, packet: MetroPacket);
}

struct Metro<T: MetroTcvr> {
    tcvr: T,
}

impl<'a, T: MetroTcvr> Metro<T> {
    fn new(tcvr: T) -> Metro<T> {
        Metro { tcvr }
    }

    fn process(&self, apps: &mut [Option<&'a mut dyn MetroApp<T>>]) {
        if let Some(packet) = self.tcvr.recv() {
            println!("packet recieved!");
            if let Some(app) = &mut apps[packet.port as usize] {
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

    struct TestMetroApp {
        pub packet_rx_count: usize,
        pub packet_tx_count: usize,
        pub last_tx_packet: Option<MetroPacket>,
        pub last_rx_packet: Option<MetroPacket>,
    }

    impl<'a> TestMetroApp {
        fn new() -> TestMetroApp {
            TestMetroApp {
                packet_rx_count: 0,
                packet_tx_count: 0,
                last_tx_packet: None,
                last_rx_packet: None,
            }
        }
    }

    impl<'a> MetroApp<TestMetroTcvr> for TestMetroApp {
        fn rx_callback(&mut self, packet: MetroPacket) {
            self.packet_rx_count += 1;
            self.last_rx_packet = Some(packet.clone());
        }

        fn send(&mut self, metro: &Metro<TestMetroTcvr>, packet: MetroPacket) {
            self.packet_tx_count += 1;
            metro.send(packet);
        }
    }

    #[test]
    fn test_send() {
        let packet_fifo_aobi: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());
        let packet_fifo_aibo: RefCell<VecDeque<MetroPacket>> = RefCell::new(VecDeque::new());

        let tcvr_a = TestMetroTcvr::new(packet_fifo_aobi.clone(), packet_fifo_aibo.clone());
        let tcvr_b = TestMetroTcvr::new(packet_fifo_aibo.clone(), packet_fifo_aobi.clone());

        let metro_a = Metro::new(tcvr_a);
        let metro_b = Metro::new(tcvr_b);

        let mut test_metro_app_a = TestMetroApp::new();
        let mut test_metro_app_b = TestMetroApp::new();

        let mut apps_a: [Option<&mut dyn MetroApp<TestMetroTcvr>>; 256] =
            core::array::from_fn(|_| None);
        apps_a[0] = Some(&mut test_metro_app_a);

        let mut apps_b: [Option<&mut dyn MetroApp<TestMetroTcvr>>; 256] =
            core::array::from_fn(|_| None);
        apps_b[0] = Some(&mut test_metro_app_b);

        let mut test_packet = MetroPacket::new();

        test_packet.data[0] = 0xEF;
        test_packet.data[1] = 0xBE;
        test_packet.data[2] = 0xAD;
        test_packet.data[3] = 0xDE;

        test_packet.data_len = 4;

        test_metro_app_a.send(&metro_a, test_packet);
        dbg!(packet_fifo_aobi.borrow().len());

        assert_eq!(test_metro_app_a.packet_tx_count, 1);

        metro_b.process(&mut apps_b);

        assert_eq!(test_metro_app_b.packet_rx_count, 1);
    }
}
