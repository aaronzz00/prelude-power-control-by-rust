/// Comprehensive probe: Bit-Bang power on Port A + UART probe on B, C, D
///
/// Hardware mapping (FT4232H_Orka Prelude):
///   Port A (FT66ORKA0) = Bit-Bang power control (POW1/2, VCHARGER1/2, RESET1/2)
///   Port B (FT66ORKA1) = TBD (possibly 2nd power control or other)
///   Port C (FT66ORKA2) = DUT1 single-wire UART (9600 baud)
///   Port D (FT66ORKA3) = DUT2 single-wire UART (9600 baud)
use libftd2xx::{BitMode, Ftdi, FtdiCommon};
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::Duration;

const POW1: u8 = 0x10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // =================================================================
    // Step 1: Open Port A in Bit-Bang mode and power on DUT1
    // =================================================================
    println!("=== Step 1: Opening Port A for Bit-Bang power control ===");
    let mut ft = Ftdi::with_description("FT4232H_Orka Prelude A")?;
    ft.set_usb_parameters(4096)?;
    ft.set_chars(0, false, 0, false)?;
    ft.set_timeouts(Duration::from_millis(5000), Duration::from_millis(5000))?;
    ft.set_latency_timer(Duration::from_millis(16))?;
    ft.set_flow_control_none()?;
    ft.set_baud_rate(62500)?;
    ft.set_bit_mode(0xFF, BitMode::AsyncBitbang)?;
    println!("Port A opened in Bit-Bang mode.");

    // Read current state
    let mut data_read = [0u8; 7];
    let _ = ft.read(&mut data_read)?;
    let mut state: u8 = 0x00;

    // Power ON DUT1
    println!("Powering ON DUT1 (POW1 = 0x10)...");
    state |= POW1;
    let mut payload = [0u8; 7];
    payload[6] = state;
    ft.write_all(&payload)?;
    println!("DUT1 powered on. Waiting 3 seconds for boot...");
    sleep(Duration::from_secs(3));

    // =================================================================
    // Step 2: Try UART on ports B, C, D
    // =================================================================
    let ports_to_probe = [
        ("/dev/cu.usbserial-FT66ORKA1", "Port B"),
        ("/dev/cu.usbserial-FT66ORKA2", "Port C"),
        ("/dev/cu.usbserial-FT66ORKA3", "Port D"),
    ];

    let cmd = b"[init_status,]\r\n";

    for (port_path, label) in &ports_to_probe {
        println!("\n=== Probing {} ({}) at 9600 baud ===", label, port_path);

        match serialport::new(*port_path, 9600)
            .timeout(Duration::from_millis(100))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .open()
        {
            Ok(mut port) => {
                println!("  Opened successfully. Sending [init_status,]\\r\\n ...");
                let _ = port.write_all(cmd);
                let _ = port.flush();

                // Listen for 3 seconds
                let mut buffer = [0u8; 1024];
                let start = std::time::Instant::now();
                let listen_duration = Duration::from_secs(3);
                let mut total_bytes = 0usize;

                while start.elapsed() < listen_duration {
                    match port.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            total_bytes += n;
                            let text = String::from_utf8_lossy(&buffer[..n]);
                            print!("  [{}] RX: {}", label, text);
                            io::stdout().flush().unwrap();
                        }
                        Ok(_) => {
                            sleep(Duration::from_millis(10));
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                            // Expected
                        }
                        Err(e) => {
                            eprintln!("  [{}] Error: {:?}", label, e);
                            break;
                        }
                    }
                }
                if total_bytes == 0 {
                    println!("  [{}] No data received.", label);
                } else {
                    println!("\n  [{}] Total bytes received: {}", label, total_bytes);
                }
            }
            Err(e) => {
                println!("  Failed to open: {:?}", e);
            }
        }
    }

    // =================================================================
    // Step 3: Power OFF DUT1
    // =================================================================
    println!("\n=== Step 3: Powering OFF DUT1 ===");
    state &= !POW1;
    payload[6] = state;
    ft.write_all(&payload)?;
    println!("DUT1 powered off.");

    Ok(())
}
