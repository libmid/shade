use std::ffi::{c_char, c_int, c_ulong, c_ushort, c_void};

pub enum Display {}

#[repr(C)]
pub struct XRRScreenResources {
    timestamp: c_ulong,
    config_timestamp: c_ulong,
    pub ncrtc: c_int,
    pub crtcs: *mut c_ulong,
    noutputs: c_int,
    outputs: *mut c_ulong,
}

#[repr(C)]
pub struct XRRCrtcGamma {
    pub size: c_int,
    pub red: *mut c_ushort,
    pub green: *mut c_ushort,
    pub blue: *mut c_ushort,
}

#[link(name = "X11")]
unsafe extern "C" {
    pub fn XOpenDisplay(display_name: *mut c_char) -> *mut Display;
    pub fn XCloseDisplay(display: *mut Display) -> c_int;

    pub fn XDefaultScreen(dpy: *mut Display) -> c_int;

    pub fn XRootWindow(dpy: *mut Display, screen: c_int) -> c_ulong;

    pub fn XRRGetScreenResourcesCurrent(
        dpy: *const Display,
        root: c_ulong,
    ) -> *mut XRRScreenResources;
    pub fn XRRGetCrtcGammaSize(dpy: *mut Display, crtc: c_ulong) -> c_int;
    pub fn XRRAllocGamma(size: c_int) -> *mut XRRCrtcGamma;
    pub fn XRRSetCrtcGamma(dpy: *mut Display, crtc: c_ulong, gamma: *mut XRRCrtcGamma);
    pub fn XRRFreeGamma(gamma: *mut XRRCrtcGamma) -> c_int;

    pub fn XFree(obj: *mut c_void) -> c_int;
}
