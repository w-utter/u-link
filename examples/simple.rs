use u_link::{Monitor, NetworkItf, enumerator};

fn main() {
    let mut monitor = Monitor::new().unwrap().listen().unwrap();

    let mut buf = [0; 4096];

    println!("initial network interfaces:");
    for itf in enumerator::enumerate() {
        println!("\t{itf:?}");
    }

    println!("\nchanges:");

    loop {
        let len = monitor.recv(&mut buf).unwrap();
        let itf = NetworkItf::from_bytes(&buf[..len]).unwrap();
        println!("\t{itf:?}");
    }
}
