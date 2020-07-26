extern crate rtlsdr;

use std::os::raw::{c_void, c_uchar};
use std::ptr;
use rtlsdr::Error;
use std::time::Duration;
use std::thread;

#[allow(unused_mut, unused_assignments)]
fn sdr_config(dev: &rtlsdr::Device) -> Result<(), Error> {
    let info = dev.get_usb_strings()?;
    println!("Info: {:?}\n", info);

    // ---------- Get/Set/Get Hardware Info ----------
    println!("1. Getting hardware info...");
    let mut hw_info = dev.get_hw_info()?;

    println!("Vendor ID:             {:?}", hw_info.vendor_id);
    println!("Product ID:            {:?}", hw_info.product_id);
    println!("Manufacturer:          {:?}", hw_info.manufact);
    println!("Product:               {:?}", hw_info.product);
    println!("Serial number:         {:?}", hw_info.serial);
    println!("Serial number enabled: {:?}", hw_info.have_serial);
    println!("IR endpoint enabled:   {:?}", hw_info.enable_ir);
    println!("Remote wakeup enabled: {:?}", hw_info.remote_wakeup);
    println!("");

    println!("Writing hardware info...");
    dev.set_hw_info(&hw_info)?;

    println!("2. Getting hardware info...");
    let hw_info = dev.get_hw_info()?;

    println!("Vendor ID:             {:?}", hw_info.vendor_id);
    println!("Product ID:            {:?}", hw_info.product_id);
    println!("Manufacturer:          {:?}", hw_info.manufact);
    println!("Product:               {:?}", hw_info.product);
    println!("Serial number:         {:?}", hw_info.serial);
    println!("Serial number enabled: {:?}", hw_info.have_serial);
    println!("IR endpoint enabled:   {:?}", hw_info.enable_ir);
    println!("Remote wakeup enabled: {:?}", hw_info.remote_wakeup);
    println!("");

    // ---------- Get Tuner Gain ----------
    println!("get_tuner_type: {}", dev.get_tuner_type());
    dev.set_xtal_freq(28800000, 28800000)?;
    println!("set_xtal_freq - 28800000");
    println!("");

    // ---------- Set Tuner Gain ----------
    dev.set_tuner_gain_mode(true)?;
    println!("set_tuner_gain_mode successful...");

    let gains = dev.get_tuner_gains()?;
    println!("get_tuner_gains successful...");
    println!("\ntuner gains:  {:?}\n", gains);

    dev.set_tuner_gain(gains[2])?;
    println!("set_tuner_gain {:?} successful...", gains[2]);

    // ---------- Get/Set Sample Rate ----------
    let samplerate: i32 = 2083334;
    dev.set_sample_rate(samplerate)?;
    println!("set_sample_rate {} successful...", samplerate);

    println!("get_sample_rate {} successful...\n", dev.get_sample_rate());

    // ---------- Get/Set Xtal Freq ----------
    let (mut rtl_freq, mut tuner_freq) = dev.get_xtal_freq()?;
    println!("get_xtal_freq successful - rtl_freq: {}, tuner_freq: {}",
             rtl_freq,
             tuner_freq);

    rtl_freq = 28800000;
    tuner_freq = 28800000;

    dev.set_xtal_freq(rtl_freq, tuner_freq)?;
    println!("set_xtal_freq successful - rtl_freq: {}, tuner_freq: {}",
             rtl_freq,
             tuner_freq);
    println!("");

    // ---------- Get/Set Center Freq ----------
    dev.set_center_freq(978000000)?;
    println!("set_center_freq successful - 978000000");
    println!("get_center_freq: {}\n", dev.get_center_freq());

    // ---------- Set Tuner Bandwidth ----------
    let bw: i32 = 1000000;
    println!("Setting bandwidth: {}", bw);

    dev.set_tuner_bandwidth(bw)?;
    println!("set_tuner_bandwidth {} Successful", bw);

    // ---------- Buffer Reset ----------
    dev.reset_buffer()?;
    println!("reset_buffer successful...");

    // ---------- Get/Set Freq Correction ----------
    let mut freq_corr = dev.get_freq_correction();
    println!("get_freq_correction - {}", freq_corr);

    freq_corr += 1;
    dev.set_freq_correction(freq_corr)?;
    println!("set_freq_correction successful - {}", freq_corr);
    println!("");
    // ----------  ----------
    Ok(())
}

unsafe extern "C" fn read_async_callback(buf: *mut c_uchar, len: u32, ctx: *mut c_void) {
    let _ = ctx;
    let v = Vec::<u8>::from_raw_parts(buf, len as usize, len as usize);
    println!("----- read_async_callback buffer size - {}", len);
    println!("----- {} {} {} {} {} {}",
             v[0],
             v[1],
             v[2],
             v[3],
             v[4],
             v[5]);
}


fn main() -> Result<(), Error> {
    // ---------- Device Check ----------
    let count = rtlsdr::get_device_count();
    if count == 0 {
        println!("No devices found, exiting.");
        return Err(Error::NoDevice);
    }

    for i in 0..count {
        let info = rtlsdr::get_device_usb_strings(i)?;
        println!("UsbInfo: {} - {:?}", i, info);
    }

    let index = 0;
    println!("===== Device name, index {}: {} =====",
             index,
             rtlsdr::get_device_name(0));
    println!("===== Running tests using device indx: 0 =====\n");

    let dev = rtlsdr::open(index)?;

    sdr_config(&dev)?;

    println!("calling read_sync...");
    for i in 0..10 {
        let (_, read_count) = dev.read_sync(rtlsdr::DEFAULT_BUF_LENGTH)?;
        println!("----- read_sync requested iteration {} -----", i);
        println!("\tread_sync requested - {}", rtlsdr::DEFAULT_BUF_LENGTH);
        println!("\tread_sync received  - {}", read_count);
    }

    dev.reset_buffer()?;

    // read_async is a blocking call and doesn't return until
    // async_stop is explicitly called, so we spawn a thread
    // that sleeps for a bit while our async callback runs...
    let d = dev.clone();
    thread::spawn(move || {
        println!("async_stop thread sleeping for 5 seconds...");
        thread::sleep(Duration::from_millis(5000));
        println!("async_stop thread awake, canceling read async...");
        d.cancel_async().unwrap();
    });

    println!("calling read_async...");
    dev.read_async(Some(read_async_callback),
                   ptr::null_mut(),
                   rtlsdr::DEFAULT_ASYNC_BUF_NUMBER,
                   rtlsdr::DEFAULT_BUF_LENGTH)?;
    println!("read_async returned successfully...");

    dev.close()?;
    println!("device close successful...");

    Ok(())
}
