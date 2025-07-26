// src/main.rs

use rppal::gpio::{Gpio, InputPin, Level};

use evdev::{Key, InputEvent, EventType, AttributeSet}; // Add AttributeSet import
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use anyhow::{Result, Context};
use log::{info, debug};
use env_logger;

// Define GPIO pins for each encoder and the main button
const ENCODER_PINS: [(u8, u8); 5] = [
    (17, 27), // Encoder 1: Out A (CLK), Out B (DT)
    (23, 24), // Encoder 2
    (5, 6),   // Encoder 3
    (19, 26), // Encoder 4
    (20, 21), // Encoder 5
];

const MAIN_BUTTON_PIN: u8 = 10; // For the 2-pin button

// Structure to hold encoder state
struct Encoder {
    pin_a: InputPin,
    pin_b: InputPin,
    last_state_a: Level,
}

impl Encoder {
    fn new(gpio: &Gpio, pin_a_num: u8, pin_b_num: u8) -> Result<Self> {
        // Use into_input_pullup() directly on the Pin to set pull-up and convert to InputPin
        let pin_a = gpio.get(pin_a_num)?.into_input_pullup();
        let pin_b = gpio.get(pin_b_num)?.into_input_pullup();

        let initial_state_a = pin_a.read();

        Ok(Encoder {
            pin_a,
            pin_b,
            last_state_a: initial_state_a,
        })
    }

    fn update(&mut self) -> Option<i8> {
        let current_state_a = self.pin_a.read();
        let current_state_b = self.pin_b.read();

        if current_state_a != self.last_state_a {
            self.last_state_a = current_state_a;
            if current_state_a == Level::Low {
                if current_state_b == Level::Low {
                    return Some(1); // Clockwise
                } else {
                    return Some(-1); // Counter-clockwise
                }
            }
        }
        None
    }
}


fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Raspberry Pi Encoder Input service...");

    let gpio = Gpio::new().context("Failed to initialize GPIO")?;

    // --- Setup Encoders ---
    let mut encoders: Vec<Encoder> = Vec::with_capacity(ENCODER_PINS.len());
    for (i, &(pin_a_num, pin_b_num)) in ENCODER_PINS.iter().enumerate() {
        let encoder = Encoder::new(&gpio, pin_a_num, pin_b_num)
            .with_context(|| format!("Failed to set up encoder {}", i + 1))?;
        encoders.push(encoder);
        info!("Encoder {} (GPIO {}/{}) configured.", i + 1, pin_a_num, pin_b_num);
    }

    // --- Setup Main Button ---
    // Use into_input_pullup() directly on the Pin to set pull-up and convert to InputPin
    let main_button_pin = gpio.get(MAIN_BUTTON_PIN)?.into_input_pullup();
    info!("Main Button (GPIO {}) configured.", MAIN_BUTTON_PIN);

    // --- Setup evdev Virtual Device ---
    let uinput_device = Arc::new(Mutex::new(
        VirtualDeviceBuilder::new()?
            .name("EC12 Multi-Encoder Keyboard")
	    // Use AttributeSet::from_iter for with_keys in 0.12.2
            .with_keys(&AttributeSet::from_iter(vec![
                // Encoder 1: Left/Right
                Key::KEY_LEFT, Key::KEY_RIGHT,
                // Encoder 2: Up/Down
                Key::KEY_UP, Key::KEY_DOWN,
                // Encoder 3: A/D
                Key::KEY_A, Key::KEY_D,
                // Encoder 4: W/S
                Key::KEY_W, Key::KEY_S,
                // Encoder 5: KPPLUS/KPMINUS
                Key::KEY_KPPLUS, Key::KEY_KPMINUS,
                // Main Button: Enter
                Key::KEY_ENTER,
            ]))?
            .build()
            .context("Failed to build virtual device")?
    ));
    info!("Virtual input device created.");

    // --- Event loop for Encoders ---
    let mut handles = Vec::new();
    for (i, mut encoder) in encoders.into_iter().enumerate() {
        let uinput_device_clone: Arc<Mutex<VirtualDevice>> = Arc::clone(&uinput_device);
        let key_map = match i {
            0 => (Key::KEY_LEFT, Key::KEY_RIGHT), // Encoder 1: Left/Right
            1 => (Key::KEY_UP, Key::KEY_DOWN),    // Encoder 2: Up/Down
            2 => (Key::KEY_A, Key::KEY_D),        // Encoder 3: A/D
            3 => (Key::KEY_W, Key::KEY_S),        // Encoder 4: W/S
            4 => (Key::KEY_KPPLUS, Key::KEY_KPMINUS), // Encoder 5: KPPLUS/KPMINUS
            _ => unreachable!(),
        };

        let handle = thread::spawn(move || -> Result<()> {
            let mut last_detection_time = Instant::now();
            let debounce_delay = Duration::from_millis(5);

            loop {
                thread::sleep(Duration::from_micros(100));

                if last_detection_time.elapsed() < debounce_delay {
                    continue;
                }

                if let Some(direction) = encoder.update() {
                    let mut device = uinput_device_clone.lock().unwrap();
                    if direction == 1 {
                        debug!("Encoder {} Clockwise: {:?}", i + 1, key_map.0);
                        device.emit(&[
                            InputEvent::new(EventType::KEY, key_map.0.0, 1), // Pass EventType::KEY directly
                            InputEvent::new(EventType::KEY, key_map.0.0, 0), // Pass EventType::KEY directly
                        ])?;
                    } else {
                        debug!("Encoder {} Counter-Clockwise: {:?}", i + 1, key_map.1);
                        device.emit(&[
                            InputEvent::new(EventType::KEY, key_map.1.0, 1), // Pass EventType::KEY directly
                            InputEvent::new(EventType::KEY, key_map.1.0, 0), // Pass EventType::KEY directly
                        ])?;
                    }
                    last_detection_time = Instant::now();
                }
            }
        });
        handles.push(handle);
    }

    // --- Event loop for Main Button ---
    let uinput_device_clone: Arc<Mutex<VirtualDevice>> = Arc::clone(&uinput_device);
    let button_handle = thread::spawn(move || -> Result<()> {
        let mut last_state = main_button_pin.read();
        let mut last_press_time = Instant::now();
        let debounce_delay = Duration::from_millis(50);

        loop {
            thread::sleep(Duration::from_millis(10));
            let current_state = main_button_pin.read();

            if current_state != last_state {
                if last_press_time.elapsed() < debounce_delay {
                    continue;
                }

                last_state = current_state;
                if current_state == Level::Low { // Button pressed (pulled to ground)
                    info!("Main Button Pressed: KEY_ENTER!");
                    let mut device = uinput_device_clone.lock().unwrap();
                    device.emit(&[
                        InputEvent::new(EventType::KEY, Key::KEY_ENTER.0, 1), // Pass EventType::KEY directly
                        InputEvent::new(EventType::KEY, Key::KEY_ENTER.0, 0), // Pass EventType::KEY directly
                    ])?;
                }
                last_press_time = Instant::now();
            }
        }
    });
    handles.push(button_handle);


    // Wait for all threads to finish (they won't in this case, it's a daemon)
    for handle in handles {
        handle.join().expect("Thread panicked")?;
    }

    info!("Service stopped.");
    Ok(())
}
