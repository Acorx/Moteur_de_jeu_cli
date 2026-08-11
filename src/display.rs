use crate::Result;
use crate::project::{CameraConfig, Project, RenderConfig};
#[cfg(all(feature = "display", target_os = "windows"))]
use crate::simulation::World;

pub const MIN_ZOOM: u32 = 1;
pub const MAX_ZOOM: u32 = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveCamera {
    pub x: i64,
    pub y: i64,
    pub zoom: u32,
}
impl InteractiveCamera {
    pub fn from_config(value: &CameraConfig) -> Self {
        Self {
            x: value.x,
            y: value.y,
            zoom: value.pixels_per_unit.clamp(MIN_ZOOM, MAX_ZOOM),
        }
    }
    pub fn pan(&mut self, x: i64, y: i64) {
        self.x = self.x.saturating_add(x);
        self.y = self.y.saturating_add(y);
    }
    pub fn zoom_by(&mut self, delta: i32) {
        self.zoom = if delta >= 0 {
            self.zoom.saturating_mul(2)
        } else {
            self.zoom / 2
        }
        .clamp(MIN_ZOOM, MAX_ZOOM);
    }
    pub fn recenter(&mut self) {
        self.x = 0;
        self.y = 0;
    }
    pub fn apply(&self, config: &mut RenderConfig) {
        config.camera = CameraConfig {
            x: self.x,
            y: self.y,
            pixels_per_unit: self.zoom,
        };
    }
}

#[cfg(all(feature = "display", target_os = "windows"))]
pub fn play(project: Project, max_ticks: Option<u64>) -> Result<()> {
    windows::play(project, max_ticks)
}
#[cfg(all(feature = "display", not(target_os = "windows")))]
pub fn play(_project: Project, _max_ticks: Option<u64>) -> Result<()> {
    Err("la feature display est actuellement implémentée pour Windows".into())
}
#[cfg(not(feature = "display"))]
pub fn play(_project: Project, _max_ticks: Option<u64>) -> Result<()> {
    Err("la commande play requiert une compilation avec --features display".into())
}

#[cfg(all(feature = "display", target_os = "windows"))]
mod windows {
    use super::*;
    use crate::render;
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr::{null, null_mut};
    use std::time::{Duration, Instant};

    type Hwnd = *mut c_void;
    type Hinstance = *mut c_void;
    type Hdc = *mut c_void;
    type Hcursor = *mut c_void;
    type Lparam = isize;
    type Wparam = usize;
    type Lresult = isize;
    #[repr(C)]
    struct WndClassW {
        style: u32,
        proc: Option<unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult>,
        cls_extra: i32,
        wnd_extra: i32,
        instance: Hinstance,
        icon: *mut c_void,
        cursor: Hcursor,
        background: *mut c_void,
        menu: *const u16,
        class: *const u16,
    }
    #[repr(C)]
    struct Msg {
        hwnd: Hwnd,
        message: u32,
        wparam: Wparam,
        lparam: Lparam,
        time: u32,
        point: [i32; 2],
        private: u32,
    }
    #[repr(C)]
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        size_image: u32,
        x: i32,
        y: i32,
        used: u32,
        important: u32,
    }
    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u32; 1],
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn RegisterClassW(c: *const WndClassW) -> u16;
        fn CreateWindowExW(
            ex: u32,
            class: *const u16,
            title: *const u16,
            style: u32,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            parent: Hwnd,
            menu: *mut c_void,
            instance: Hinstance,
            param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(h: Hwnd, m: u32, w: Wparam, l: Lparam) -> Lresult;
        fn DestroyWindow(h: Hwnd) -> i32;
        fn PostQuitMessage(code: i32);
        fn PeekMessageW(m: *mut Msg, h: Hwnd, min: u32, max: u32, remove: u32) -> i32;
        fn TranslateMessage(m: *const Msg) -> i32;
        fn DispatchMessageW(m: *const Msg) -> Lresult;
        fn GetDC(h: Hwnd) -> Hdc;
        fn ReleaseDC(h: Hwnd, dc: Hdc) -> i32;
        fn GetAsyncKeyState(key: i32) -> i16;
        fn LoadCursorW(instance: Hinstance, name: *const u16) -> Hcursor;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn StretchDIBits(
            dc: Hdc,
            x: i32,
            y: i32,
            dw: i32,
            dh: i32,
            sx: i32,
            sy: i32,
            sw: i32,
            sh: i32,
            bits: *const c_void,
            info: *const BitmapInfo,
            usage: u32,
            rop: u32,
        ) -> i32;
    }
    const WM_DESTROY: u32 = 2;
    const WM_CLOSE: u32 = 16;
    const PM_REMOVE: u32 = 1;
    const WS_OVERLAPPEDWINDOW: u32 = 0x00cf0000;
    const WS_VISIBLE: u32 = 0x10000000;
    const SRCCOPY: u32 = 0x00cc0020;
    unsafe extern "system" fn proc(h: Hwnd, m: u32, w: Wparam, l: Lparam) -> Lresult {
        if m == WM_CLOSE {
            unsafe {
                DestroyWindow(h);
            }
            return 0;
        }
        if m == WM_DESTROY {
            unsafe {
                PostQuitMessage(0);
            }
            return 0;
        }
        unsafe { DefWindowProcW(h, m, w, l) }
    }
    fn down(key: i32) -> bool {
        unsafe { GetAsyncKeyState(key) < 0 }
    }
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
    pub fn play(project: Project, max_ticks: Option<u64>) -> Result<()> {
        let mut config = project.render.clone();
        let mut camera = InteractiveCamera::from_config(&config.camera);
        let mut world = World::from_project(project);
        let class = wide("AetherionWindow");
        let title = wide("Aetherion M3");
        let wc = WndClassW {
            style: 0,
            proc: Some(proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance: null_mut(),
            icon: null_mut(),
            cursor: unsafe { LoadCursorW(null_mut(), 32512usize as *const u16) },
            background: null_mut(),
            menu: null(),
            class: class.as_ptr(),
        };
        if unsafe { RegisterClassW(&wc) } == 0 {
            return Err("impossible d'enregistrer la classe de fenêtre Windows".into());
        }
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                100,
                100,
                config.width as i32 + 32,
                config.height as i32 + 64,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };
        if hwnd.is_null() {
            return Err("impossible de créer la fenêtre Windows".into());
        }
        let tick_duration = Duration::from_secs_f64(1.0 / f64::from(world.tick_rate));
        let mut next_tick = Instant::now() + tick_duration;
        let mut paused = false;
        let mut previous = [false; 256];
        'outer: loop {
            let mut msg = Msg {
                hwnd: null_mut(),
                message: 0,
                wparam: 0,
                lparam: 0,
                time: 0,
                point: [0; 2],
                private: 0,
            };
            while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if msg.message == 18 {
                    break 'outer;
                }
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            let keys = [0x1b, 0x20, 0x4e, 0x52, 0x25, 0x26, 0x27, 0x28, 0xBB, 0xBD];
            let mut pressed = [false; 256];
            for &k in &keys {
                pressed[k as usize] = down(k);
            }
            if pressed[0x1b] {
                break;
            }
            if pressed[0x20] && !previous[0x20] {
                paused = !paused;
            }
            if pressed[0x52] && !previous[0x52] {
                camera.recenter();
            }
            if pressed[0xBB] && !previous[0xBB] {
                camera.zoom_by(1);
            }
            if pressed[0xBD] && !previous[0xBD] {
                camera.zoom_by(-1);
            }
            let pan = (16 / i64::from(camera.zoom)).max(1);
            if pressed[0x25] {
                camera.pan(-pan, 0);
            }
            if pressed[0x27] {
                camera.pan(pan, 0);
            }
            if pressed[0x26] {
                camera.pan(0, pan);
            }
            if pressed[0x28] {
                camera.pan(0, -pan);
            }
            let step_once = paused && pressed[0x4e] && !previous[0x4e];
            let now = Instant::now();
            if step_once || (!paused && now >= next_tick) {
                world.step()?;
                next_tick = now + tick_duration;
            }
            if max_ticks.is_some_and(|limit| world.tick >= limit) {
                break;
            }
            previous = pressed;
            camera.apply(&mut config);
            let image = render::render(&world, &config)?.0;
            let mut bgra = Vec::with_capacity(image.pixels.len() / 3 * 4);
            for rgb in image.pixels.chunks_exact(3) {
                bgra.extend_from_slice(&[rgb[2], rgb[1], rgb[0], 0]);
            }
            let info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: size_of::<BitmapInfoHeader>() as u32,
                    width: image.width as i32,
                    height: -(image.height as i32),
                    planes: 1,
                    bit_count: 32,
                    compression: 0,
                    size_image: 0,
                    x: 0,
                    y: 0,
                    used: 0,
                    important: 0,
                },
                colors: [0],
            };
            let dc = unsafe { GetDC(hwnd) };
            unsafe {
                StretchDIBits(
                    dc,
                    0,
                    0,
                    image.width as i32,
                    image.height as i32,
                    0,
                    0,
                    image.width as i32,
                    image.height as i32,
                    bgra.as_ptr().cast(),
                    &info,
                    0,
                    SRCCOPY,
                );
                ReleaseDC(hwnd, dc);
            }
            std::thread::sleep(Duration::from_millis(4));
        }
        unsafe {
            DestroyWindow(hwnd);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_is_bounded_and_recentered() {
        let mut c = InteractiveCamera {
            x: 5,
            y: -2,
            zoom: 1,
        };
        c.zoom_by(-1);
        assert_eq!(c.zoom, MIN_ZOOM);
        for _ in 0..20 {
            c.zoom_by(1);
        }
        assert_eq!(c.zoom, MAX_ZOOM);
        c.pan(2, 3);
        c.recenter();
        assert_eq!((c.x, c.y), (0, 0));
    }
}
