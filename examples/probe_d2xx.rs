/// Full D2XX probe: Bit-Bang on Port A + D2XX UART mode on B, C, D
///
/// Since standard VCP serialport didn't work, we try opening all ports
/// via the FTDI D2XX driver directly.
use libftd2xx::{BitMode, Ftdi, FtdiCommon};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

const POW1: u8 = 0x10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // =================================================================
    // Step 1: Open Port A in Bit-Bang mode and power on DUT1
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
    println!("Port A opened in Bit-Bang mode.");

    // Power ON DUT1
    let mut state: u8 = 0x00;
    state |= POW1;
    let mut payload = [0u8; 7];
    payload[6] = state;
    ft_a.write_all(&payload)?;
    println!("DUT1 powered on (POW1). Waiting 3 seconds for boot...");
    sleep(Duration::from_secs(3));

    // =================================================================
    // Step 2: Probe ports B, C, D via D2XX in standard UART mode
    // =================================================================
    let ports_to_probe = [
        "FT4232H_Orka Prelude B",
        "FT4232H_Orka Prelude C",
        "FT4232H_Orka Prelude D",
    ];

    let cmd = b"[init_status,]\r\n";

    for desc in &ports_to_probe {
        println!("\n=== Probing '{}' via D2XX UART at 9600 baud ===", desc);

        match Ftdi::with_description(desc) {
            Ok(mut ft) => {
                // Configure as standard UART
                ft.set_baud_rate(9600)?;
                ft.set_data_characteristics(
                    libftd2xx::BitsPerWord::Bits8,
                    libftd2xx::StopBits::Bits1,
                    libftd2xx::Parity::No,
                )?;
                ft.set_flow_control_none()?;
                ft.set_timeouts(Duration::from_millis(500), Duration::from_millis(500))?;
                ft.set_latency_timer(Duration::from_millis(16))?;

                // Make sure we're NOT in bit-bang mode (reset to normal)
                ft.set_bit_mode(0x00, BitMode::Reset)?;

                println!("  Opened and configured. Sending [init_status,]\\r\\n ...");
                ft.write_all(cmd)?;

                // Listen for 3 seconds
                let mut buffer = [0u8; 1024];
                let start = std::time::Instant::now();
                let listen_duration = Duration::from_secs(3);
                let mut total_bytes = 0usize;

                while start.elapsed() < listen_duration {
                    // Check queue status
                    let rx_bytes = ft.queue_status()?;
                    if rx_bytes > 0 {
                        let to_read = std::cmp::min(rx_bytes, buffer.len());
                        let n = ft.read(&mut buffer[..to_read])?;
                        total_bytes += n;
                        let text = String::from_utf8_lossy(&buffer[..n]);
                        print!("  [{}] RX: {}", desc, text);
                        std::io::stdout().flush().unwrap();
                    } else {
                        sleep(Duration::from_millis(10));
                    }
                }

                if total_bytes == 0 {
                    println!("  [{}] No data received.", desc);
                } else {
                    println!("\n  [{}] Total bytes received: {}", desc, total_bytes);
                }
            }
            Err(e) => {
                println!("  Failed to open '{}': {:?}", desc, e);
            }
        }
    }

    // =================================================================
    // Step 3: Power OFF DUT1
    // =================================================================
    println!("\n=== Step 3: Powering OFF DUT1 ===");
    state &= !POW1;
    payload[6] = state;
    ft_a.write_all(&payload)?;
    println!("DUT1 powered off.");

    Ok(())
}
