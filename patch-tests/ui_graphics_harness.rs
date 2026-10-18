// Standalone tests for the production ui_graphics.rs module.
// Core Graphics and Objective-C are test doubles, not the real renderer.
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, dead_code)]
use std::collections::HashMap;

#[macro_export]
macro_rules! export_c_func {
    ($name:ident($($arg:tt)*)) => { stringify!($name) };
}
#[macro_export]
macro_rules! msg_class {
    ($env:ident; UIImage imageWithCGImage:$image:ident) => {
        $env.make_ui_image($image)
    };
}
mod dyld {
    pub use crate::export_c_func;
    pub type FunctionExports = &'static [&'static str];
}
mod objc {
    pub type id = u32;
    pub const nil: id = 0;
    pub use crate::msg_class;
}
mod mem {
    pub type GuestUSize = u32;
    pub struct Ptr;
    impl Ptr {
        pub fn null() -> Self { Self }
    }
}
mod frameworks {
    pub mod core_graphics {
        #[derive(Clone, Copy)]
        pub struct CGSize { pub width: f32, pub height: f32 }
        pub mod cg_context {
            use crate::Environment;
            pub type CGContextRef = u32;
            pub fn CGContextRetain(env: &mut Environment, c: u32) -> u32 {
                env.objects.get_mut(&c).unwrap().refs += 1;
                c
            }
            pub fn CGContextRelease(env: &mut Environment, c: u32) {
                env.release(c);
            }
            pub fn CGContextTranslateCTM(env: &mut Environment, c: u32, x: f32, y: f32) {
                env.objects.get_mut(&c).unwrap().operations.push(("translate", x, y));
            }
            pub fn CGContextScaleCTM(env: &mut Environment, c: u32, x: f32, y: f32) {
                env.objects.get_mut(&c).unwrap().operations.push(("scale", x, y));
            }
        }
        pub mod cg_color_space {
            use crate::Environment;
            pub fn CGColorSpaceCreateDeviceRGB(env: &mut Environment) -> u32 {
                env.alloc("color_space")
            }
            pub fn CGColorSpaceRelease(env: &mut Environment, c: u32) { env.release(c); }
        }
        pub mod cg_bitmap_context {
            use crate::{Environment, mem::Ptr};
            pub fn CGBitmapContextCreate(
                env: &mut Environment, _: Ptr, w: u32, h: u32,
                bpc: u32, row: u32, cs: u32, info: u32,
            ) -> u32 {
                assert_eq!(env.objects[&cs].kind, "color_space");
                assert_eq!(bpc, 8);
                assert_eq!(row, w * 4);
                assert_eq!(info, 0x4001);
                let c = env.alloc("context");
                let object = env.objects.get_mut(&c).unwrap();
                object.dimensions = (w, h);
                object.pixels = vec![0; (row * h) as usize];
                c
            }
            pub fn CGBitmapContextCreateImage(env: &mut Environment, c: u32) -> u32 {
                let snapshot = env.objects[&c].clone();
                let image = env.alloc("cg_image");
                env.objects.get_mut(&image).unwrap().pixels = snapshot.pixels;
                env.objects.get_mut(&image).unwrap().dimensions = snapshot.dimensions;
                image
            }
        }
        pub mod cg_image {
            use crate::Environment;
            pub const kCGImageAlphaPremultipliedLast: u32 = 1;
            pub const kCGImageByteOrder32Big: u32 = 0x4000;
            pub fn CGImageRelease(env: &mut Environment, c: u32) { env.release(c); }
        }
    }
}
#[path = "../src/frameworks/uikit/ui_graphics.rs"]
mod ui_graphics;
#[derive(Default)]
struct FrameworkState { uikit: UIKit }
#[derive(Default)]
struct UIKit { ui_graphics: ui_graphics::State }
#[derive(Clone, Default)]
struct Object {
    refs: u32,
    kind: &'static str,
    dimensions: (u32, u32),
    pixels: Vec<u8>,
    operations: Vec<(&'static str, f32, f32)>,
}
#[derive(Default)]
pub struct Environment {
    framework_state: FrameworkState,
    objects: HashMap<u32, Object>,
    next_id: u32,
    ui_images: HashMap<u32, u32>,
}
impl Environment {
    fn alloc(&mut self, kind: &'static str) -> u32 {
        self.next_id += 1;
        self.objects.insert(self.next_id, Object { refs: 1, kind, ..Default::default() });
        self.next_id
    }
    fn release(&mut self, c: u32) {
        let obj = self.objects.get_mut(&c).expect("release of destroyed object");
        obj.refs -= 1;
        if obj.refs == 0 { self.objects.remove(&c); }
    }
    fn make_ui_image(&mut self, cg_image: u32) -> u32 {
        self.objects.get_mut(&cg_image).unwrap().refs += 1;
        let result = self.alloc("ui_image");
        self.ui_images.insert(result, cg_image);
        result
    }
    fn drain_ui_image(&mut self, ui_image: u32) {
        let cg_image = self.ui_images.remove(&ui_image).unwrap();
        self.release(cg_image);
        self.release(ui_image);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use frameworks::core_graphics::CGSize;
    use ui_graphics::*;

    #[test]
    fn exports_include_missing_api_and_companion_functions() {
        for name in ["UIGraphicsBeginImageContext", "UIGraphicsGetImageFromCurrentImageContext", "UIGraphicsEndImageContext"] {
            assert!(FUNCTIONS.contains(&name));
        }
    }
    #[test]
    fn bitmap_format_transform_and_balanced_ownership() {
        let mut env = Environment::default();
        UIGraphicsBeginImageContext(&mut env, CGSize { width: 2.25, height: 3.25 });
        let c = UIGraphicsGetCurrentContext(&mut env);
        let context = &env.objects[&c];
        assert_eq!(context.dimensions, (3, 4));
        assert_eq!(context.refs, 1);
        assert_eq!(context.pixels, vec![0; 48]);
        assert_eq!(context.operations, [("translate", 0.0, 4.0), ("scale", 1.0, -1.0)]);
        assert_eq!(env.objects.len(), 1, "temporary color space leaked");
        UIGraphicsEndImageContext(&mut env);
        assert_eq!(UIGraphicsGetCurrentContext(&mut env), 0);
        assert!(env.objects.is_empty());
    }
    #[test]
    fn nested_contexts_restore_previous_context() {
        let mut env = Environment::default();
        let ordinary = env.alloc("context");
        UIGraphicsPushContext(&mut env, ordinary);
        UIGraphicsBeginImageContext(&mut env, CGSize { width: 2.0, height: 2.0 });
        let outer = UIGraphicsGetCurrentContext(&mut env);
        UIGraphicsBeginImageContext(&mut env, CGSize { width: 1.0, height: 1.0 });
        let inner = UIGraphicsGetCurrentContext(&mut env);
        assert_ne!(outer, inner);
        UIGraphicsEndImageContext(&mut env);
        assert_eq!(UIGraphicsGetCurrentContext(&mut env), outer);
        assert!(!env.objects.contains_key(&inner));
        UIGraphicsEndImageContext(&mut env);
        assert_eq!(UIGraphicsGetCurrentContext(&mut env), ordinary);
        UIGraphicsEndImageContext(&mut env);
        assert_eq!(UIGraphicsGetCurrentContext(&mut env), ordinary);
        UIGraphicsPopContext(&mut env);
        env.release(ordinary);
        assert!(env.objects.is_empty());
    }
    #[test]
    fn no_image_context_returns_nil_without_popping() {
        let mut env = Environment::default();
        assert_eq!(UIGraphicsGetImageFromCurrentImageContext(&mut env), 0);
        UIGraphicsEndImageContext(&mut env);
        let ordinary = env.alloc("context");
        UIGraphicsPushContext(&mut env, ordinary);
        assert_eq!(UIGraphicsGetImageFromCurrentImageContext(&mut env), 0);
        UIGraphicsEndImageContext(&mut env);
        assert_eq!(UIGraphicsGetCurrentContext(&mut env), ordinary);
    }
    #[test]
    fn snapshots_survive_end_and_do_not_alias_bitmap() {
        let mut env = Environment::default();
        UIGraphicsBeginImageContext(&mut env, CGSize { width: 1.0, height: 1.0 });
        let c = UIGraphicsGetCurrentContext(&mut env);
        env.objects.get_mut(&c).unwrap().pixels = vec![10, 20, 30, 40];
        let first = UIGraphicsGetImageFromCurrentImageContext(&mut env);
        env.objects.get_mut(&c).unwrap().pixels = vec![50, 60, 70, 80];
        let second = UIGraphicsGetImageFromCurrentImageContext(&mut env);
        let cg_first = env.ui_images[&first];
        let cg_second = env.ui_images[&second];
        assert_ne!(cg_first, cg_second);
        UIGraphicsEndImageContext(&mut env);
        assert_eq!(env.objects[&cg_first].pixels, [10, 20, 30, 40]);
        assert_eq!(env.objects[&cg_second].pixels, [50, 60, 70, 80]);
        assert_eq!(env.objects[&cg_first].refs, 1);
        env.drain_ui_image(first);
        env.drain_ui_image(second);
        assert!(env.objects.is_empty());
    }
    #[test]
    fn temporary_push_preserves_image_context() {
        let mut env = Environment::default();
        UIGraphicsBeginImageContext(&mut env, CGSize { width: 1.0, height: 1.0 });
        let image_context = UIGraphicsGetCurrentContext(&mut env);
        let ordinary = env.alloc("context");
        UIGraphicsPushContext(&mut env, ordinary);
        assert_eq!(UIGraphicsGetImageFromCurrentImageContext(&mut env), 0);
        UIGraphicsPopContext(&mut env);
        assert_eq!(UIGraphicsGetCurrentContext(&mut env), image_context);
        UIGraphicsEndImageContext(&mut env);
        env.release(ordinary);
        assert!(env.objects.is_empty());
    }
}
