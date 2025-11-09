use std::f32::consts::PI;
use std::f64::consts::E;
use std::ffi::*;
use std::process::exit;

use clap::Parser;
use iced::widget::{button, column, container, row, slider, slider::HandleShape, text};
use iced::{
    application, border::Radius, gradient::Linear, Background, Color, Element, Gradient, Task,
};

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
    unsafe {
        let dpy = x11::XOpenDisplay(core::ptr::null_mut());
        if dpy.is_null() {
            eprintln!("XOpenDisplay(NULL) failed. Endure DISPLAY is set correctly!");
            exit(1);
        }
        let screen = x11::XDefaultScreen(dpy);
        if let Some(temp) = shade.set_temp {
            set_temp(dpy, screen, temp);
        } else {
            // Run GUI here
            application("Shade", update, view)
                .theme(|_| iced::Theme::Ferra)
                .window_size((500.0, 300.0))
                .run_with(move || {
                    (
                        ShadeState {
                            temp: 6500,
                            dpy,
                            screen,
                        },
                        Task::none(),
                    )
                })
                .unwrap();
        }
        x11::XCloseDisplay(dpy);
    }
}

struct ShadeState {
    temp: u32,
    dpy: *mut x11::Display,
    screen: i32,
}

#[derive(Clone, Debug)]
enum Message {
    ChangeTemp(u32),
    SetDay,
    SetNight,
    SetAstro,
    Reset,
}

fn update(state: &mut ShadeState, message: Message) {
    match message {
        Message::ChangeTemp(temp) => {
            set_temp(state.dpy, state.screen, temp);
            state.temp = temp;
        }

        Message::SetDay => {
            set_temp(state.dpy, state.screen, TEMP_DAY);
            state.temp = TEMP_DAY;
        }
        Message::SetNight => {
            set_temp(state.dpy, state.screen, TEMP_NIGHT);
            state.temp = TEMP_NIGHT;
        }
        Message::SetAstro => {
            set_temp(state.dpy, state.screen, 700);
            state.temp = 700;
        }
        Message::Reset => {
            set_temp(state.dpy, state.screen, TEMP_DAY);
            state.temp = TEMP_DAY;
        }
    }
}

fn view(state: &ShadeState) -> Element<Message> {
    column![
        column![text("Set Screen Temperature")].padding(10),
        slider(700..=10_000, state.temp, Message::ChangeTemp).style(|theme, status| {
            let mut style = slider::default(theme, status);
            style.rail.width = 60.0;
            style.handle.shape = HandleShape::Rectangle {
                width: 5,
                border_radius: Radius::new(10),
            };
            style.handle.background = Background::Color(Color::BLACK);

            // Temp Gradient
            let grad1 = Linear::new(PI / 2.0)
                .add_stop(0.0, kelvin_to_rgb(700.0))
                .add_stop(1.0, kelvin_to_rgb(state.temp as f64));
            let grad2 = Linear::new(PI / 2.0)
                .add_stop(0.0, kelvin_to_rgb(state.temp as f64))
                .add_stop(1.0, kelvin_to_rgb(10_000.0));

            style.rail.backgrounds = (
                Background::Gradient(Gradient::Linear(grad1)),
                Background::Gradient(Gradient::Linear(grad2)),
            );
            style
        })
    ]
    .into()
}

pub fn kelvin_to_rgb(temp_k: f64) -> Color {
    let temp = temp_k / 100.0;

    // Red
    let red = if temp <= 66.0 {
        255.0
    } else {
        let r = 329.698727446 * (temp - 60.0).powf(-0.1332047592);
        r.clamp(0.0, 255.0)
    };

    // Green
    let green = if temp <= 66.0 {
        let g = 99.4708025861 * (temp.ln()) - 161.1195681661;
        g.clamp(0.0, 255.0)
    } else {
        let g = 288.1221695283 * (temp - 60.0).powf(-0.0755148492);
        g.clamp(0.0, 255.0)
    };

    // Blue
    let blue = if temp >= 66.0 {
        255.0
    } else if temp <= 19.0 {
        0.0
    } else {
        let b = 138.5177312231 * (temp - 10.0).ln() - 305.0447927307;
        b.clamp(0.0, 255.0)
    };

    Color::from_rgb8(red.round() as u8, green.round() as u8, blue.round() as u8)
}
