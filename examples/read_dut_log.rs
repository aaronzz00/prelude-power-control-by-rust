use prelude_power_controller::{DeviceSide, PowerController, WireMode};
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port_name = "/dev/cu.usbserial-FT66ORKA0";
    println!("Connecting controller to {}...", port_name);

    // Connect using default mode (SingleWire - 9600 baud)
    let mut controller = PowerController::connect(port_name, WireMode::SingleWire)?;
    println!("Controller connected successfully! Setting timeout for 10ms for non-blocking read behavior...");

    // Set a very short timeout for reading so we don't block
    controller
        .port_mut()
        .set_timeout(Duration::from_millis(10))?;

    println!("--------------------------------");
    println!("Power ON Device 1 (5V)...");
    controller.power_on(DeviceSide::Device1)?;

    // Give DUT a moment to power on
    sleep(Duration::from_millis(1000));

    // Open the other FTDI ports for listening/writing at 9600 baud
    let mut log_port_1 = serialport::new("/dev/cu.usbserial-FT66ORKA1", 9600)
        .timeout(Duration::from_millis(10))
        .open()
        .ok();

    let mut log_port_2 = serialport::new("/dev/cu.usbserial-FT66ORKA2", 9600)
        .timeout(Duration::from_millis(10))
        .open()
        .ok();

    let mut log_port_3 = serialport::new("/dev/cu.usbserial-FT66ORKA3", 9600)
        .timeout(Duration::from_millis(10))
        .open()
        .ok();

    println!("Sending [init_status,] command to all available ports to probe...");
    let init_cmd = b"[init_status,]\r\n";
    if let Some(ref mut p) = log_port_1 {
        let _ = p.write_all(init_cmd);
        let _ = p.flush();
    }
    if let Some(ref mut p) = log_port_2 {
        let _ = p.write_all(init_cmd);
        let _ = p.flush();
    }
    if let Some(ref mut p) = log_port_3 {
        let _ = p.write_all(init_cmd);
        let _ = p.flush();
    }

    println!("Listening for DUT log (on interfaces 0, 1, 2, and 3 at 9600 baud) for 15 seconds...");

    let mut buffer = [0u8; 1024];
    let start = std::time::Instant::now();
    let duration = Duration::from_secs(15);

    while start.elapsed() < duration {
        // Read Port 1
        if let Some(ref mut p) = log_port_1 {
            match p.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    print!("[PORT 1] {}", String::from_utf8_lossy(&buffer[..n]));
                    io::stdout().flush().unwrap();
                }
                _ => {}
            }
        }

        // Read Port 2
        if let Some(ref mut p) = log_port_2 {
            match p.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    print!("[PORT 2] {}", String::from_utf8_lossy(&buffer[..n]));
                    io::stdout().flush().unwrap();
                }
                _ => {}
            }
        }

        // Read Port 3
        if let Some(ref mut p) = log_port_3 {
            match p.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    print!("[PORT 3] {}", String::from_utf8_lossy(&buffer[..n]));
                    io::stdout().flush().unwrap();
                }
                _ => {}
            }
        }

        // Read Port 0 (Controller)
        match controller.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                let text = String::from_utf8_lossy(&buffer[..bytes_read]);
                print!("[PORT 0] {}", text);
                io::stdout().flush().unwrap();
            }
            Ok(_) => {
                sleep(Duration::from_millis(10)); // Yield a bit
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("Error reading controller: {:?}", e);
                break;
            }
        }
    }

    println!("\n--------------------------------");
    println!("Sending [2700_shutdown,] command to gracefully shutdown DUT...");
    let shutdown_cmd = b"[2700_shutdown,]\r\n";
    if let Some(ref mut p) = log_port_1 {
        let _ = p.write_all(shutdown_cmd);
        let _ = p.flush();
    }
    if let Some(ref mut p) = log_port_2 {
        let _ = p.write_all(shutdown_cmd);
        let _ = p.flush();
    }
    if let Some(ref mut p) = log_port_3 {
        let _ = p.write_all(shutdown_cmd);
        let _ = p.flush();
    }

    sleep(Duration::from_secs(2)); // wait for shutdown sequence to complete

    println!("Power OFF Device 1 (5V)...");
    controller.power_off(DeviceSide::Device1)?;

    Ok(())
}
