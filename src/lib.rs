extern crate libc;
extern crate kernel32;
extern crate time;
extern crate jpeg_decoder as jpeg;

use std::ffi::CString;
use std::mem;
use std::slice;
use kernel32::GetLastError;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[repr(C)]
struct SimpleCapParams {
    buf: *mut libc::c_int,
    width: libc::c_uint,
    height: libc::c_uint,
    framerate: libc::c_float
}

#[no_mangle]
pub fn num_devices() -> usize {
    unsafe { countCaptureDevices() as usize }
}

#[no_mangle]
pub fn version() -> u32 {
    unsafe { ESCAPIVersion() }
}

#[no_mangle]
pub fn post_convert(out_buffer : *mut libc::c_void, in_buffer : *const libc::c_void, width : u32, height : u32, format : u32) {
    match format {
        5 => {
            //let decoder = jpeg::Decoder::new();
        }
        _ => {
            println!("doing memcopy");
            unsafe { libc::memcpy(out_buffer, in_buffer, (width * height * 4) as usize);}
        }
    }
}

#[no_mangle]
pub fn init
    (
        index : usize, 
        wdt : u32, 
        hgt : u32, 
        desired_fps: f32,
        device_buffer : *mut i32,
        device_ptr : *mut *mut Device,
        format_index : *mut u32
    ) -> usize 
{
    let init_result = init_rust(index, wdt, hgt, desired_fps, device_buffer, format_index);
    match init_result {
        Err (_) => {
            unsafe {
                *device_ptr = std::ptr::null_mut();
            }
            0
        },
        Ok (device) => {
            let heap_allocated_device : Box<Device> = Box::new(device);
            let ptr : *mut Device = Box::into_raw(heap_allocated_device);

            unsafe {
                  *device_ptr = ptr;
            }
            1
        }
    }
}

#[no_mangle]
pub fn free_string(ptr_to_str : *mut i8) {
    unsafe { CString::from_raw(ptr_to_str); }
}

#[no_mangle]
pub fn free_device(ptr_to_device : *mut Device) {
    unsafe { 
        let device =  Box::from_raw(ptr_to_device); 
        deinitCapture(device.device_idx);
    }
}


#[no_mangle]
pub fn allocate_buffer_i32(width : u32, height : u32, buffer_ptr : *mut *mut i32) {
    let mut data  : Vec<i32> = vec![0; (width * height) as usize];
    let ptr = data.as_mut_ptr();
    let len = data.len();
    unsafe { *buffer_ptr = ptr; }
    mem::forget(data);
}

#[no_mangle]
pub fn allocate_buffer_u8(size : u32, buffer_ptr : *mut *mut u8) {
    let mut data  : Vec<u8> = vec![0; size as usize];
    let ptr = data.as_mut_ptr();
    let len = data.len();
    unsafe { *buffer_ptr = ptr; }
    mem::forget(data);
}

#[no_mangle]
pub fn get_device_name(device_ptr : *const Device, str_ptr : *mut *mut i8 ) -> usize {
    let device : &Device = unsafe { &*device_ptr };
    let device_name : String = device.name();
    let c_str_result = CString::new(device_name);
    match c_str_result {
        Err (_) => 0,
        Ok (c_str) => {
            let length = c_str.as_bytes().len();
            let ptr_c_string = c_str.into_raw();
            unsafe { *str_ptr = ptr_c_string; }
            length
        }
    }
}

#[no_mangle]
pub fn get_capture_buffer(device_ptr : *const Device) -> usize {
    let before = time::now();
    let device : &Device = unsafe { &*device_ptr };

    let capture_result = device.capture();
    match capture_result {
        Err (_) => 0,
        Ok (_) => {
            let after = time::now();
            let diff = after - before;
            //println!("device.capture() took {} ms", diff.num_milliseconds());
            1
        }
    }
}

#[no_mangle]
pub fn try_decode_file() {

    let mut file = File::open("C:\\Users\\sesa455926\\Documents\\visual studio 2017\\Projects\\ConsoleApp3\\ConsoleApp3\\bin\\x64\\Debug\\data\\image3.jpg");
    match file {
        Err (_) => println!("cannot read file"),
        Ok (a) => {
            println!("can read file");
            
            let mut decoder = jpeg::Decoder::new(BufReader::new(a));
            let before = time::now();

            let pixels = decoder.decode();//.expect("failed to decode image");
            match pixels {
                Ok(_) => {
                    println!("can decode image");
                    let after = time::now();
                    let diff = after - before;
                    println!("DECODING JPG took {} ms", diff.num_milliseconds());
                },
                Err (a) => println!("Error is {}", a)
            }
            //let metadata = decoder.info().unwrap();
            ()
        }
    }

}

pub fn init_rust(index: usize, wdt: u32, hgt: u32, desired_fps: f32, buffer : *mut i32, format_index : *mut u32) -> Result<Device, Error> {
    let mut params = Box::new(SimpleCapParams {
        width: wdt,
        height: hgt,
        buf: buffer,
        framerate : desired_fps
    });

    let index = index as libc::c_uint;
    if unsafe { initCapture(index, &mut *params, format_index) } == 1 {
        assert!(unsafe { getCaptureErrorCode(index) } == 0);
        Ok(Device {
            device_idx: index,
            params: params,
            desired_fps: desired_fps as u64,
        })
    } else {
        Err(Error::CouldNotOpenDevice(unsafe { GetLastError() }))
    }
}

/// The device requests BGRA format, so the frames are in BGRA.
pub struct Device {
    device_idx: libc::c_uint,
    params: Box<SimpleCapParams>,
    desired_fps: u64,
}

impl Device {

    pub fn capture(&self) -> Result<(), Error> {
        unsafe { doCapture(self.device_idx) };

        const MAX_TRY_ATTEMPTS: usize = 1000;
        for nb_attemp in 0..MAX_TRY_ATTEMPTS {
            if unsafe { isCaptureDone(self.device_idx) } == 1 {
                //println!("nb_attemp is {}", nb_attemp);
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        Err(Error::CaptureTimeout)
    }

    pub fn name(&self) -> String {
        let mut v = vec![0u8; 100];
        unsafe { getCaptureDeviceName(self.device_idx, v.as_mut_ptr() as *mut i8, v.len() as i32) };
        let null = v.iter().position(|&c| c == 0).expect("null termination character");
        v.truncate(null);
        String::from_utf8(v).expect("device name contains invalid utf8 characters")
    }

    pub fn capture_width(&self) -> u32 {
        self.params.width
    }

    pub fn capture_height(&self) -> u32 {
        self.params.height
    }
}

#[derive(Debug)]
pub enum Error {
    CouldNotOpenDevice(libc::c_ulong),
    CaptureTimeout,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CouldNotOpenDevice(i) => write!(fmt, "could not open camera device, errorcode: {}", i),
            Error::CaptureTimeout => write!(fmt, "timeout during image capture"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::CouldNotOpenDevice(_) => "could not open camera device",
            Error::CaptureTimeout => "timeout during image capture",
        }
    }
}

extern "C" {
    fn countCaptureDevices() -> libc::c_int;
    fn initCapture(_: libc::c_uint, _: *mut SimpleCapParams, format_index : *mut u32) -> libc::c_int;
    fn deinitCapture(_: libc::c_uint);
    fn doCapture(_: libc::c_uint) -> libc::c_int;
    fn isCaptureDone(_: libc::c_uint) -> libc::c_int;
    fn getCaptureDeviceName(_: libc::c_uint, _: *mut libc::c_char, _: libc::c_int);
    fn ESCAPIVersion() -> libc::c_uint;
    fn getCapturePropertyValue(_: libc::c_uint, _: libc::c_int) -> libc::c_float;
    fn getCapturePropertyAuto(_: libc::c_uint, _: libc::c_int) -> libc::c_int;
    fn setCaptureProperty(_: libc::c_uint, _: libc::c_int, _: libc::c_float, _: libc::c_int);
    fn getCaptureErrorLine(_: libc::c_uint) -> libc::c_int;
    fn getCaptureErrorCode(_: libc::c_uint) -> libc::c_int;
}
