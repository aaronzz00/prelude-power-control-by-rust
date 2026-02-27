/// Boot log capture v2: Power on FIRST via D2XX Bit-Bang, then listen via VCP
///
/// Fix: Previous attempt opened B/C/D via D2XX before A, which may have
/// caused chip-level conflicts. This time:
///   1. Open Port A via D2XX Bit-Bang → Power ON
///   2. Wait for DUT to start booting
///   3. Open B/C/D via standard serialport (VCP) for listening
use libftd2xx::{BitMode, Ftdi, FtdiCommon};
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::Duration;

const POW1: u8 = 0x10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // =================================================================
    // Step 1: Power ON via Bit-Bang on Port A (this worked before!)
    // =================================================================
    println!("=== Step 1: Opening Port A for Bit-Bang power control ===");
    let mut ft_a = Ftdi::with_description("FT4232H_Orka Prelude A")?;
    ft_a.set_usb_parameters(4096)?;
    ft_a.set_chars(0, false, 0, false)?;
    ft_a.set_timeouts(Duration::from_millis(5000), Duration::from_millis(5000))?;
    ft_a.set_latency_timer(Duration::from_millis(16))?;
    ft_a.set_flow_control_none()?;
    ft_a.set_baud_rate(62500)?;
    ft_a.set_bit_mode(0xFF, BitMode::AsyncBitbang)?;

    let mut payload = [0u8; 7];
    println!("Powering ON DUT1 (POW1=0x10)...");
    payload[6] = POW1;
    ft_a.write_all(&payload)?;
    println!("DUT1 5V ON! Waiting 1 second before opening listeners...");
    sleep(Duration::from_secs(1));

    // =================================================================
    // Step 2: Open listeners via standard serialport (VCP, not D2XX)
    // =================================================================
    println!("\n=== Step 2: Opening VCP listeners on B, C, D at 9600 baud ===");

    let vcp_ports = [
        ("B", "/dev/cu.usbserial-FT66ORKA1"),
        ("C", "/dev/cu.usbserial-FT66ORKA2"),
        ("D", "/dev/cu.usbserial-FT66ORKA3"),
    ];

    let mut listeners: Vec<(&str, Box<dyn serialport::SerialPort>)> = Vec::new();

    for (label, path) in &vcp_ports {
        print!("  Opening {} ({})... ", label, path);
        match serialport::new(*path, 9600)
            .timeout(Duration::from_millis(10))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .open()
        {
            Ok(port) => {
                println!("OK");
                listeners.push((label, port));
            }
            Err(e) => {
                println!("FAILED: {:?}", e);
            }
        }
    }

    // Also send [init_status,]\r\n on each to actively probe
    println!("\n  Sending [init_status,]\\r\\n on all listener ports...");
    let cmd = b"[init_status,]\r\n";
    for (label, port) in listeners.iter_mut() {
        let _ = port.write_all(cmd);
        let _ = port.flush();
        println!("  Sent to Port {}", label);
    }

    // =================================================================
    // Step 3: Listen for 15 seconds
    // =================================================================
    println!("\n=== Step 3: Listening for boot log (15 seconds) ===");
    let mut buffer = [0u8; 1024];
    let start = std::time::Instant::now();
    let duration = Duration::from_secs(15);
    let mut port_bytes: Vec<usize> = vec![0; listeners.len()];

    while start.elapsed() < duration {
        for (i, (label, port)) in listeners.iter_mut().enumerate() {
            match port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    port_bytes[i] += n;
                    let text = String::from_utf8_lossy(&buffer[..n]);
                    print!("[Port {}] {}", label, text);
                    io::stdout().flush().unwrap();
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(5));
    }

    // =================================================================
    // Summary
    // =================================================================
    println!("\n\n=== Summary ===");
    for (i, (label, _)) in listeners.iter().enumerate() {
        println!("  Port {}: {} bytes received", label, port_bytes[i]);
    }

    // =================================================================
    // Power OFF DUT1
    // =================================================================
    println!("\n=== Powering OFF DUT1 ===");
    payload[6] = 0x00;
    ft_a.write_all(&payload)?;
    println!("DUT1 powered off.");

    Ok(())
}
