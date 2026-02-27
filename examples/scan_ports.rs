use serialport::available_ports;

fn main() {
    println!("Scanning for available serial ports...");
    match available_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                println!("No serial ports found!");
            } else {
                for p in ports {
                    println!("Found port: {}", p.port_name);
                    match p.port_type {
                        serialport::SerialPortType::UsbPort(info) => {
                            println!("  Type: USB");
                            println!("  VID: {:04x}", info.vid);
                            println!("  PID: {:04x}", info.pid);
                            println!("  Serial Number: {:?}", info.serial_number);
                            println!("  Manufacturer: {:?}", info.manufacturer);
                            println!("  Product: {:?}", info.product);
                        }
                        _ => println!("  Type: Other"),
                    }
                    println!("--------------------------------");
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing serial ports: {:?}", e);
        }
    }
}
