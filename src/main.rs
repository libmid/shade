use std::f64::consts::E;
use std::ffi::*;
use std::process::exit;

use clap::Parser;

mod x11;

const TEMP_DAY: u32 = 6500;
const TEMP_NIGHT: u32 = 4500;
const TEMP_ZERO: u32 = 700;
const GAMMA_MULT: f64 = 65535.0;

const GAMMA_K0GR: f64 = -1.477_513_091_398_17;
const GAMMA_K1GR: f64 = 0.285_901_647_720_55;

// Blue color
const GAMMA_K0BR: f64 = -4.383_216_501_148_72;
const GAMMA_K1BR: f64 = 0.621_215_876_944_7;

// Blue range (T0 = TEMPERATURE_NORM - TEMPERATURE_ZERO)

// Red color
const GAMMA_K0RB: f64 = 1.753_902_040_390_18;
const GAMMA_K1RB: f64 = -0.115_080_567_148_2;

// Green color
const GAMMA_K0GB: f64 = 1.492_216_049_151_44;
const GAMMA_K1GB: f64 = -0.075_135_095_889_21;

#[derive(Debug, Parser)]
#[command(version)]
struct Shade {
    #[arg(short)]
    set_temp: Option<u32>,
}

fn set_temp(dpy: *mut x11::Display, screen: c_int, temp: u32) {
    let t = temp as f64;
    unsafe {
        let root = x11::XRootWindow(dpy, screen);
        let res = x11::XRRGetScreenResourcesCurrent(dpy, root);

        let gammar;
        let gammag;
        let gammab;

        if temp < TEMP_DAY {
            gammar = 1.0;
            if temp > TEMP_ZERO {
                let g = (t - TEMP_ZERO as f64).log(E);
                gammag = (GAMMA_K0GR + GAMMA_K1GR * g).clamp(0.0, 1.0);
                gammab = (GAMMA_K0BR + GAMMA_K1BR * g).clamp(0.0, 1.0);
            } else {
                gammag = 0.0;
                gammab = 0.0;
            }
        } else {
            let g = (t - (TEMP_DAY - TEMP_ZERO) as f64).log(E);
            gammar = (GAMMA_K0RB + GAMMA_K1RB * g).clamp(0.0, 1.0);
            gammag = (GAMMA_K0GB + GAMMA_K1GB * g).clamp(0.0, 1.0);
            gammab = 1.0;
        }

        for c in 0..(*res).ncrtc {
            let crtcxid = *(*res).crtcs.offset(c as isize);
            let size = x11::XRRGetCrtcGammaSize(dpy, crtcxid);

            let crtc_gamma = x11::XRRAllocGamma(size);

            for i in 0..size {
                let g = GAMMA_MULT * i as f64 / size as f64;
                *(*crtc_gamma).red.offset(i as isize) = (g * gammar + 0.5) as u16;
                *(*crtc_gamma).green.offset(i as isize) = (g * gammag + 0.5) as u16;
                *(*crtc_gamma).blue.offset(i as isize) = (g * gammab + 0.5) as u16;
            }

            x11::XRRSetCrtcGamma(dpy, crtcxid, crtc_gamma);
            x11::XRRFreeGamma(crtc_gamma);
        }

        x11::XFree(res as *mut c_void);
    }
}

fn main() {
    let shade = Shade::parse();

    if let Some(temp) = shade.set_temp {
        unsafe {
            let dpy = x11::XOpenDisplay(core::ptr::null_mut());
            if dpy.is_null() {
                eprintln!("XOpenDisplay(NULL) failed. Endure DISPLAY is set correctly!");
                exit(1);
            }
            let screen = x11::XDefaultScreen(dpy);

            set_temp(dpy, screen, temp);

            x11::XCloseDisplay(dpy);
        }
    }
}
