/// Boot log capture: Open listeners FIRST, then power on DUT1
///
/// Strategy: Open D2XX UART on B, C, D BEFORE powering on via Bit-Bang on A,
/// so we can capture the very first byte of boot log.
use libftd2xx::{BitMode, Ftdi, FtdiCommon};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

const POW1: u8 = 0x10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // =================================================================
    // Step 1: Open ALL listener ports FIRST (before power on)
    // =================================================================
    println!("=== Step 1: Opening listener ports B, C, D via D2XX UART (9600) ===");

    let descs = [
        ("B", "FT4232H_Orka Prelude B"),
        ("C", "FT4232H_Orka Prelude C"),
        ("D", "FT4232H_Orka Prelude D"),
    ];

    let mut listeners: Vec<(&str, Ftdi)> = Vec::new();

    for (label, desc) in &descs {
        print!("  Opening {} ('{}')... ", label, desc);
        match Ftdi::with_description(desc) {
            Ok(mut ft) => {
                ft.set_baud_rate(9600)?;
                ft.set_data_characteristics(
                    libftd2xx::BitsPerWord::Bits8,
                    libftd2xx::StopBits::Bits1,
                    libftd2xx::Parity::No,
                )?;
                ft.set_flow_control_none()?;
                ft.set_timeouts(Duration::from_millis(100), Duration::from_millis(100))?;
                ft.set_latency_timer(Duration::from_millis(2))?; // Low latency for fast capture
                ft.set_bit_mode(0x00, BitMode::Reset)?; // Ensure standard UART mode

                // Purge any stale data
                ft.purge_rx()?;
                ft.purge_tx()?;

                println!("OK");
                listeners.push((label, ft));
            }
            Err(e) => {
                println!("FAILED: {:?}", e);
            }
        }
    }

    println!("\n  {} listener ports ready.", listeners.len());

    // =================================================================
    // Step 2: Open Port A in Bit-Bang mode and power on DUT1
    // =================================================================
    println!("\n=== Step 2: Opening Port A for Bit-Bang power control ===");
    let mut ft_a = Ftdi::with_description("FT4232H_Orka Prelude A")?;
    ft_a.set_usb_parameters(4096)?;
    ft_a.set_chars(0, false, 0, false)?;
    ft_a.set_timeouts(Duration::from_millis(5000), Duration::from_millis(5000))?;
    ft_a.set_latency_timer(Duration::from_millis(16))?;
    ft_a.set_flow_control_none()?;
    ft_a.set_baud_rate(62500)?;
    ft_a.set_bit_mode(0xFF, BitMode::AsyncBitbang)?;

    let mut payload = [0u8; 7];
    println!("Powering ON DUT1 (POW1)...");
    payload[6] = POW1;
    ft_a.write_all(&payload)?;
    println!("DUT1 powered on! Listening for boot log...\n");

    // =================================================================
    // Step 3: Listen on all ports for 15 seconds
    // =================================================================
    println!("=== Step 3: Listening for boot log (15 seconds) ===");
    let mut buffer = [0u8; 4096];
    let start = std::time::Instant::now();
    let duration = Duration::from_secs(15);
    let mut port_bytes: Vec<usize> = vec![0; listeners.len()];

    while start.elapsed() < duration {
        for (i, (label, ft)) in listeners.iter_mut().enumerate() {
            match ft.queue_status() {
                Ok(rx_bytes) if rx_bytes > 0 => {
                    let to_read = std::cmp::min(rx_bytes, buffer.len());
                    match ft.read(&mut buffer[..to_read]) {
                        Ok(n) if n > 0 => {
                            port_bytes[i] += n;
                            let text = String::from_utf8_lossy(&buffer[..n]);
                            print!("[Port {}] {}", label, text);
                            std::io::stdout().flush().unwrap();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(5));
    }

    // =================================================================
    // Step 4: Summary
    // =================================================================
    println!("\n\n=== Summary ===");
    for (i, (label, _)) in listeners.iter().enumerate() {
        println!("  Port {}: {} bytes received", label, port_bytes[i]);
    }

    // =================================================================
    // Step 5: Power OFF DUT1
    // =================================================================
    println!("\n=== Powering OFF DUT1 ===");
    payload[6] = 0x00;
    ft_a.write_all(&payload)?;
    println!("DUT1 powered off.");

    Ok(())
}
