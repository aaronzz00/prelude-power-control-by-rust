use crate::error::{PowerControllerError, Result};
use serialport::SerialPort;
use std::time::Duration;

// Hardware Constants derived from PreludeSettings.h
const VCHARGER1: u8 = 0x04;
const VCHARGER2: u8 = 0x08;
const POW1: u8 = 0x10;
const POW2: u8 = 0x20;
const RESET1: u8 = 0x01;
const RESET2: u8 = 0x02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceSide {
    Device1,
    Device2,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireMode {
    SingleWire, // Default, 9600 baud
    DoubleWire, // 192000 baud
}

impl WireMode {
    pub fn baud_rate(&self) -> u32 {
        match self {
            WireMode::SingleWire => 9600,
            WireMode::DoubleWire => 192000,
        }
    }
}

pub struct PowerController {
    port: Box<dyn SerialPort>,
    current_state: u8, // Tracks the byte status for data[6]
}

impl PowerController {
    /// Initializes and opens the serial port with the specified mode.
    /// By default, user requested SingleWire mode (9600).
    pub fn connect(port_name: &str, mode: WireMode) -> Result<Self> {
        let baud_rate = mode.baud_rate();

        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(5000))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None) // Assuming default 8N1
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(|e| PowerControllerError::PortOpenError(port_name.to_string(), e))?;

        // Initialize state to all power off and no reset
        let initial_state =
            0xFF & (!VCHARGER1) & (!VCHARGER2) & (!POW1) & (!POW2) & (!RESET1) & (!RESET2);

        let mut controller = Self {
            port,
            current_state: initial_state,
        };

        // Ensure starting with all configured power off
        controller.sync_state()?;

        Ok(controller)
    }

    /// Power ON the target device(s)
    pub fn power_on(&mut self, side: DeviceSide) -> Result<()> {
        match side {
            DeviceSide::Device1 => self.current_state |= POW1,
            DeviceSide::Device2 => self.current_state |= POW2,
            DeviceSide::Both => {
                self.current_state |= POW1;
                self.current_state |= POW2;
            }
        }
        self.sync_state()
    }

    /// Power OFF the target device(s)
    pub fn power_off(&mut self, side: DeviceSide) -> Result<()> {
        match side {
            DeviceSide::Device1 => self.current_state &= !POW1,
            DeviceSide::Device2 => self.current_state &= !POW2,
            DeviceSide::Both => {
                self.current_state &= !POW1;
                self.current_state &= !POW2;
            }
        }
        self.sync_state()
    }

    /// Enable VCHARGER for the target device(s)
    pub fn enable_vcharger(&mut self, side: DeviceSide) -> Result<()> {
        match side {
            DeviceSide::Device1 => self.current_state |= VCHARGER1,
            DeviceSide::Device2 => self.current_state |= VCHARGER2,
            DeviceSide::Both => {
                self.current_state |= VCHARGER1;
                self.current_state |= VCHARGER2;
            }
        }
        self.sync_state()
    }

    /// Disable VCHARGER for the target device(s)
    pub fn disable_vcharger(&mut self, side: DeviceSide) -> Result<()> {
        match side {
            DeviceSide::Device1 => self.current_state &= !VCHARGER1,
            DeviceSide::Device2 => self.current_state &= !VCHARGER2,
            DeviceSide::Both => {
                self.current_state &= !VCHARGER1;
                self.current_state &= !VCHARGER2;
            }
        }
        self.sync_state()
    }

    /// Execute a hardware RESET pulse for 100ms
    pub fn reset(&mut self, side: DeviceSide) -> Result<()> {
        // Assert RESET
        match side {
            DeviceSide::Device1 => self.current_state |= RESET1,
            DeviceSide::Device2 => self.current_state |= RESET2,
            DeviceSide::Both => {
                self.current_state |= RESET1;
                self.current_state |= RESET2;
            }
        }
        self.sync_state()?;

        std::thread::sleep(Duration::from_millis(100));

        // De-assert RESET
        match side {
            DeviceSide::Device1 => self.current_state &= !RESET1,
            DeviceSide::Device2 => self.current_state &= !RESET2,
            DeviceSide::Both => {
                self.current_state &= !RESET1;
                self.current_state &= !RESET2;
            }
        }
        self.sync_state()
    }

    /// Sync internal state to the hardware
    /// Creates a 7-byte command payload where index 6 contains the hardware masks.
    /// This abstracts the bit-bang data packet for an external UART adapter.
    fn sync_state(&mut self) -> Result<()> {
        // Construct the 7-byte payload as per original protocol
        // Original logic alternated 0x55/0xAA or read input; we zero pad until state byte
        let payload: [u8; 7] = [0x55, 0x55, 0x55, 0x55, 0x55, 0x55, self.current_state];

        self.port
            .write_all(&payload)
            .map_err(PowerControllerError::IoError)?;

        // Optional flush
        let _ = self.port.flush();

        Ok(())
    }

    /// Expose mutable reference to the underlying serial port for reading logs
    pub fn port_mut(&mut self) -> &mut Box<dyn SerialPort> {
        &mut self.port
    }
}

impl std::io::Read for PowerController {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.port.read(buf)
    }
}
