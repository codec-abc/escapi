extern crate libc;
extern crate kernel32;

use std::ffi::CString;

use kernel32::GetLastError;

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
pub fn init(index : usize, wdt : u32, hgt : u32, desired_fps: f32, device_ptr : *mut *mut Device ) -> usize {
    let init_result = init_rust(index, wdt, hgt, desired_fps);
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
pub fn get_capture_buffer(device_ptr : *const Device, buffer : *mut *mut [i32], buffer_length : *mut usize ) -> usize {
    let device : &Device = unsafe { &*device_ptr };
    let capture_result = device.capture();
    match capture_result {
        Err (_) => 0,
        Ok (_) => {
            let length = device.buf.len();
            let raw_ptr = device.buf.clone();
            unsafe { *buffer = Box::into_raw(raw_ptr); }
            unsafe { *buffer_length = length; }
            1
        }
    }
}

pub fn init_rust(index: usize, wdt: u32, hgt: u32, desired_fps: f32) -> Result<Device, Error> {
    let mut data = vec![0; (wdt*hgt) as usize].into_boxed_slice();
    let mut params = Box::new(SimpleCapParams {
        width: wdt,
        height: hgt,
        buf: data.as_mut_ptr(),
        framerate : desired_fps
    });

    let index = index as libc::c_uint;
    if unsafe { initCapture(index, &mut *params) } == 1 {
        assert!(unsafe { getCaptureErrorCode(index) } == 0);
        Ok(Device {
            device_idx: index,
            buf: data,
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
    buf: Box<[i32]>,
    params: Box<SimpleCapParams>,
    desired_fps: u64,
}

impl Device {

    pub fn capture(&self) -> Result<&[u8], Error> {
        unsafe { doCapture(self.device_idx) };

        const MAX_TRY_ATTEMPTS: usize = 1000;
        for _ in 0..MAX_TRY_ATTEMPTS {
            if unsafe { isCaptureDone(self.device_idx) } == 1 {
                let data = &self.buf;
                return Ok(unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8,
                                                              data.len() * 4) });
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
    fn initCapture(_: libc::c_uint, _: *mut SimpleCapParams) -> libc::c_int;
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
