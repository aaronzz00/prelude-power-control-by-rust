/// Simple: Power ON DUT1 and hold for 30 seconds so user can visually confirm
use libftd2xx::{BitMode, Ftdi, FtdiCommon};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

const POW1: u8 = 0x10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening Port A (Bit-Bang)...");
    let mut ft = Ftdi::with_description("FT4232H_Orka Prelude A")?;
    ft.set_usb_parameters(4096)?;
    ft.set_chars(0, false, 0, false)?;
    ft.set_timeouts(Duration::from_millis(5000), Duration::from_millis(5000))?;
    ft.set_latency_timer(Duration::from_millis(16))?;
    ft.set_flow_control_none()?;
    ft.set_baud_rate(62500)?;
    ft.set_bit_mode(0xFF, BitMode::AsyncBitbang)?;
    println!("Bit-Bang mode active.");

    // Read current state
    let mut rd = [0u8; 7];
    let _ = ft.read(&mut rd)?;
    println!("Current pin state: {:02X?}", rd);

    // Power ON
    let mut payload = [0u8; 7];
    payload[6] = POW1;
    ft.write_all(&payload)?;
    println!("\n*** DUT1 POWERED ON (POW1=0x10) ***");
    println!("*** Holding for 30 seconds — please check if DUT powers up ***\n");

    for i in (1..=30).rev() {
        print!("\r  {} seconds remaining...  ", i);
        std::io::stdout().flush().unwrap();
        sleep(Duration::from_secs(1));
    }

    // Power OFF
    println!("\n\n*** Powering OFF DUT1 ***");
    payload[6] = 0x00;
    ft.write_all(&payload)?;
    println!("Done.");

    Ok(())
}
