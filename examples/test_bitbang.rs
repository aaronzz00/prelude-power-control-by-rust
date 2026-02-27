/// Bit-Bang mode test — faithfully reproduces the original C++ PreludeController logic.
///
/// Port A of FT4232H is opened in Asynchronous Bit-Bang mode (Mode 0x01).
/// In this mode, writing a byte directly sets the state of the 8 GPIO pins.
///
/// Pin mapping from PreludeSettings.h:
///   RESET1    = 0x01  (bit 0)
///   RESET2    = 0x02  (bit 1)
///   VCHARGER1 = 0x04  (bit 2)
///   VCHARGER2 = 0x08  (bit 3)
///   POW1      = 0x10  (bit 4)
///   POW2      = 0x20  (bit 5)
use libftd2xx::{BitMode, DeviceType, Ftdi, FtdiCommon};
use std::thread::sleep;
use std::time::Duration;

// Hardware constants from PreludeSettings.h
const RESET1: u8 = 0x01;
const RESET2: u8 = 0x02;
const VCHARGER1: u8 = 0x04;
const VCHARGER2: u8 = 0x08;
const POW1: u8 = 0x10;
const POW2: u8 = 0x20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---------------------------------------------------------------
    // Step 1: Open the FTDI device by description (Port A)
    // ---------------------------------------------------------------
    println!("Opening FTDI device 'FT4232H_Orka Prelude A'...");
    let mut ft = Ftdi::with_description("FT4232H_Orka Prelude A")?;

    let info = ft.device_info()?;
    println!("Device info: {:?}", info);

    // ---------------------------------------------------------------
    // Step 2: Configure exactly as in C++ source
    // ---------------------------------------------------------------
    ft.set_usb_parameters(4096)?; // USB transfer size
    ft.set_chars(0, false, 0, false)?; // Disable event chars
    ft.set_timeouts(Duration::from_millis(5000), Duration::from_millis(5000))?;
    ft.set_latency_timer(Duration::from_millis(16))?; // Latency timer 16ms
    ft.set_flow_control_none()?; // No flow control
    ft.set_baud_rate(62500)?; // bit rate x16 = 1M
    println!("Port configured (baud=62500, timeout=5s, latency=16ms).");

    // ---------------------------------------------------------------
    // Step 3: Enter Asynchronous Bit-Bang Mode (all pins output)
    // ---------------------------------------------------------------
    let mask: u8 = 0xFF; // All pins are outputs
    ft.set_bit_mode(mask, BitMode::AsyncBitbang)?;
    println!("Entered Asynchronous Bit-Bang mode (Mask=0xFF).");

    // ---------------------------------------------------------------
    // Step 4: Read current pin state
    // ---------------------------------------------------------------
    let mut data_read = [0u8; 7];
    let bytes_read = ft.read(&mut data_read)?;
    println!(
        "Read {} bytes from device. data_read = {:02X?}",
        bytes_read,
        &data_read[..bytes_read]
    );

    // Use what we read as the baseline for data[6]
    let mut state: u8 = if bytes_read >= 7 {
        data_read[6]
    } else if bytes_read > 0 {
        data_read[bytes_read - 1]
    } else {
        0x00
    };
    println!("Initial pin state byte: 0x{:02X}", state);

    // ---------------------------------------------------------------
    // Step 5: Power ON Device 1  (set POW1 bit)
    // ---------------------------------------------------------------
    println!("--------------------------------");
    println!("Power ON Device 1 (POW1)...");
    state |= POW1;
    let mut payload = [0u8; 7];
    payload[6] = state;
    ft.write_all(&payload)?;
    println!("Written payload: {:02X?}", payload);

    sleep(Duration::from_secs(3));

    // ---------------------------------------------------------------
    // Step 6: Read back to confirm state
    // ---------------------------------------------------------------
    let mut readback = [0u8; 7];
    let rb = ft.read(&mut readback)?;
    println!("Readback {} bytes: {:02X?}", rb, &readback[..rb]);

    // ---------------------------------------------------------------
    // Step 7: Enable VCHARGER1
    // ---------------------------------------------------------------
    println!("Enable VCHARGER1...");
    state |= VCHARGER1;
    payload[6] = state;
    ft.write_all(&payload)?;
    println!("Written payload: {:02X?}", payload);

    sleep(Duration::from_secs(2));

    // ---------------------------------------------------------------
    // Step 8: Disable VCHARGER1 & Power OFF Device 1
    // ---------------------------------------------------------------
    println!("Disable VCHARGER1 & Power OFF Device 1...");
    state &= !VCHARGER1;
    state &= !POW1;
    payload[6] = state;
    ft.write_all(&payload)?;
    println!("Written payload: {:02X?}", payload);

    sleep(Duration::from_secs(1));

    // ---------------------------------------------------------------
    // Step 9: Reset Device 1 (pulse RESET1 for 100ms)
    // ---------------------------------------------------------------
    println!("Reset Device 1 (pulse 100ms)...");
    state |= RESET1;
    payload[6] = state;
    ft.write_all(&payload)?;
    sleep(Duration::from_millis(100));
    state &= !RESET1;
    payload[6] = state;
    ft.write_all(&payload)?;
    println!("Reset pulse complete.");

    // ---------------------------------------------------------------
    // Cleanup
    // ---------------------------------------------------------------
    println!("All Bit-Bang tests completed.");
    // ft is dropped here, which calls FT_Close

    Ok(())
}
