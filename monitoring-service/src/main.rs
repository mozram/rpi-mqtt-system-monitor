/* TODO:
    1. Monitor Temperature: vcgencmd measure_temp
    2. Monitor undervoltage: vcgencmd get_throttled -> https://www.raspberrypi.org/documentation/raspbian/applications/vcgencmd.md

    Topics:
    /rpi4/monitoring/state
    /rpi4/monitoring/temperature
    /rpi4/monitoring/throttled
*/
extern crate paho_mqtt as mqtt;

use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::process::Command;

const DELAY: u64 = 2;

fn main() {
    // Create a client & define connect options
    let cli = mqtt::AsyncClient::new("tcp://90smobsters.com:1883").unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptions::new();

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(conn_opts).wait() {
        println!("Unable to connect: {:?}", e);
        process::exit(1);
    }

    // Sig term variable and hook
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    println!("Waiting for Ctrl-C...");

    while running.load(Ordering::SeqCst) {
        // Measure temperature
        let get_temp = Command::new("vcgencmd")
                    .arg("measure_temp")
                    .output()
                    .expect("failed to execute process");
        let result = String::from_utf8(get_temp.stdout).unwrap();
        let filtered_result: String = result.chars().filter(|c| c.is_digit(10)).collect();  // Filter out non number char
        println!("{}", filtered_result);

        // Measure throttled
        let get_throttled = Command::new("vcgencmd")
                    .arg("get_throttled")
                    .output()
                    .expect("failed to execute process");
        let result_throttled = String::from_utf8(get_throttled.stdout).unwrap();
        let filtered_result_throttled: String = result_throttled.chars().filter(|c| c.is_digit(10)).collect();  // Filter out non number char
        println!("{}", filtered_result_throttled);

        // publish
        let temp_msg = mqtt::Message::new("/rpi4/monitoring/temperature", filtered_result, 0);
        let temp_tok = cli.publish(temp_msg);
        if let Err(e) = temp_tok.wait() {
            println!("Error sending message: {:?}", e);
        }

        let throttled_msg = mqtt::Message::new("/rpi4/monitoring/throttled", filtered_result_throttled, 0);
        let throttled_tok = cli.publish(throttled_msg);
        if let Err(e) = throttled_tok.wait() {
            println!("Error sending message: {:?}", e);
        }

        // Sleep for DELAY seconds
        sleep(Duration::from_secs(DELAY));
    }

    println!("Publishing exit state and closing broker...");

    // Create a message and publish it
    println!("Publishing a message on the 'test' topic");
    let msg = mqtt::Message::new("/rpi4/monitoring/state", "SHUTDOWN", 0);
    let tok = cli.publish(msg);

    if let Err(e) = tok.wait() {
        println!("Error sending message: {:?}", e);
    }

    // Disconnect from the broker
    let tok = cli.disconnect(None);
    tok.wait().unwrap();
}