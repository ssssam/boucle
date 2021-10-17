pub mod boucle_organelle {
    use nannou_osc as osc;

    pub fn main() {
        let sender: osc::Sender::<osc::Connected>;

        let port = 4001;
        let target_addr = format!("{}:{}", "127.0.0.1", port);

        let sender = osc::sender()
            .expect("Could not bind to default socket")
            .connect(target_addr)
            .expect("Could not connect to socket at address");

        let osc_addr = "/oled/line/1".to_string();
        let args = vec![osc::Type::String("Hello from Boucle looper".to_string())];
        let packet = (osc_addr, args);
        sender.send(packet).ok();

        println!("Sent a hello packet.")
    }
}

fn main() {
    boucle_organelle::main();
}
